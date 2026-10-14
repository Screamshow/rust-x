//! Tailscale endpoint outbound implementation.
//!
//! Native TS2021 registration and initial network map over authenticated Noise.
//! Direct WireGuard UDP and DERP are connected to the stack. Discovery and incoming
//! router-service access remain unfinished.

pub mod access;
pub mod config;
pub mod control;
pub mod derp;
pub mod derp_client;
pub mod engine;
pub mod mesh;
pub mod noise;
pub mod session;
pub mod web;
pub mod lan;
pub mod service;

use std::io;
use std::net::IpAddr;
use std::sync::Arc;
use std::sync::atomic::Ordering;
use std::time::Duration;

use serde_json::{Map, Value};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::UdpSocket;
use tokio::sync::{Mutex as AsyncMutex, mpsc, oneshot};

use crate::inbound::{AnyStream, Target};

use crate::outbound::quic::udp_loopback_pair;
use crate::outbound::tailscale::config::TailscaleConfig;
use crate::outbound::tailscale::control::{
    Hostinfo, RegisterRequest, RegisterResponse, load_or_create_keys, node_key_str,
};
use crate::outbound::tailscale::engine::{Cmd, TailscaleShared, resolve_target};
use crate::outbound::tailscale::session::{CAPABILITY_VERSION, ControlSession};

const CALLER_BUF: usize = 65_536;
const REPLY_QUEUE: usize = 32;
const SCRATCH_BUF: usize = 16_384;
const CONNECT_TIMEOUT: Duration = Duration::from_secs(30);

/// Tailscale endpoint outbound.
#[derive(Clone)]
pub struct Tailscale {
    cfg: TailscaleConfig,
    shared: Arc<AsyncMutex<Option<Arc<TailscaleShared>>>>,
}

impl std::fmt::Debug for Tailscale {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Tailscale")
            .field("tag", &self.cfg.tag)
            .field("control_url", &self.cfg.control_url)
            .finish()
    }
}

impl Tailscale {
    pub fn build(obj: &Map<String, Value>, at: &str) -> anyhow::Result<Self> {
        let cfg = TailscaleConfig::parse(obj, at)?;
        Ok(Self {
            cfg,
            shared: Arc::new(AsyncMutex::new(None)),
        })
    }

    /// Register this identity. AuthURL is returned to the caller for browser login;
    /// an authorized registration is not evidence that the data plane is ready.
    pub async fn register(&self) -> io::Result<RegisterResponse> {
        self.register_followup(None).await
    }

    /// Complete browser authorization with the previously returned AuthURL.
    /// Persistent state is required so follow-up uses the same node identity.
    pub async fn register_followup(&self, followup: Option<&str>) -> io::Result<RegisterResponse> {
        self.register_context(followup)
            .await
            .map(|(response, _, _, _)| response)
    }

    async fn register_context(
        &self,
        followup: Option<&str>,
    ) -> io::Result<(
        RegisterResponse,
        Option<super::tailscale::control::MapResponse>,
        x25519_dalek::StaticSecret,
        x25519_dalek::StaticSecret,
    )> {
        if followup.is_some() && (self.cfg.ephemeral || self.cfg.state_directory.is_none()) {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "tailscale: browser follow-up requires persistent identity",
            ));
        }
        let (machine, node) =
            load_or_create_keys(self.cfg.state_directory.as_deref(), self.cfg.ephemeral)?;
        let mut session = ControlSession::connect(&self.cfg.control_url, machine.clone()).await?;
        let node_key = node_key_str(&x25519_dalek::PublicKey::from(&node));
        let mut response =
            session
                .register(&RegisterRequest {
                    version: CAPABILITY_VERSION,
                    node_key: node_key.clone(),
                    auth: self.cfg.auth_key.as_ref().map(|key| {
                        super::tailscale::control::AuthObj {
                            auth_key: key.clone(),
                        }
                    }),
                    hostinfo: Hostinfo {
                        os: "linux".to_owned(),
                        hostname: self
                            .cfg
                            .hostname
                            .clone()
                            .unwrap_or_else(|| "rust-x".to_owned()),
                        routable_ips: self
                            .cfg
                            .advertise_routes
                            .iter()
                            .map(ToString::to_string)
                            .collect(),
                    },
                    ephemeral: self.cfg.ephemeral,
                    followup: followup.map(ToString::to_string),
                })
                .await?;
        let map = if response.machine_authorized == Some(true) {
            let map = session
                .initial_map(&node_key, self.cfg.hostname.as_deref().unwrap_or("rust-x"))
                .await?;
            response.node = map.node.clone();
            Some(map)
        } else {
            None
        };
        Ok((response, map, node, machine))
    }

    pub async fn shared(&self) -> io::Result<Arc<TailscaleShared>> {
        let mut guard = self.shared.lock().await;
        if let Some(shared) = &*guard {
            return Ok(Arc::clone(shared));
        }

        let (response, map, node_key, machine) = self.register_context(None).await?;
        if response.machine_authorized != Some(true) {
            return Err(io::Error::new(
                io::ErrorKind::PermissionDenied,
                "tailscale: authorization required; retrieve AuthURL through register()",
            ));
        }
        let local_vip = response
            .node
            .as_ref()
            .and_then(|node| {
                node.addresses
                    .iter()
                    .find_map(|address| address.split('/').next()?.parse::<IpAddr>().ok())
            })
            .ok_or_else(|| io::Error::other("tailscale: registration has no assigned address"))?;
        let (_cmd_tx, cmd_rx) = mpsc::channel(128);
        let local_ips = response
            .node
            .as_ref()
            .unwrap()
            .addresses
            .iter()
            .filter_map(|a| a.split('/').next()?.parse::<IpAddr>().ok())
            .collect();
        let mut mesh =
            mesh::DirectMesh::bind(node_key.clone(), local_ips, self.cfg.listen_port).await?;
        let mut home = None;
        if let Some(derp_map) = map.as_ref().and_then(|m| m.derp_map.clone()) {
            home = response
                .node
                .as_ref()
                .and_then(|n| n.home_derp)
                .filter(|id| derp_map.regions.contains_key(&id.to_string()))
                .or_else(|| derp_map.regions.values().map(|r| r.region_id).min());
            mesh.set_relay(derp_client::RelayManager::new(
                node_key.clone(),
                derp_map,
                home,
            ));
        }
        mesh.update_peers(map.as_ref().and_then(|m| m.peers.as_deref()).unwrap_or(&[]));
        crate::logging::debug(format_args!("tailscale: selected home relay {:?}", home));
        let disco = mesh.disco_public();
        let tun = if self.cfg.system_interface && !self.cfg.advertise_routes.is_empty() { Some(lan::LanTun::open()?) } else { None };
        if tun.is_some() { mesh.set_local_routes(self.cfg.advertise_routes.clone()); }
        let shared = engine::spawn_tailscale_engine_with_tun(
            self.cfg.clone(),
            local_vip,
            cmd_rx,
            Some(mesh),
            tun,
        );
        let weak = Arc::downgrade(&shared);
        let cfg = self.cfg.clone();
        let node = node_key_str(&x25519_dalek::PublicKey::from(&node_key));
        let expected_addresses = response.node.as_ref().unwrap().addresses.clone();
        tokio::spawn(async move {
            let mut session = None;
            let mut delay = 1;
            let mut stream = None;
            let mut peers = std::collections::HashMap::new();
            let mut rules = vec![];
            let mut named_rules = std::collections::HashMap::new();
            let mut addresses_valid = true;
            loop {
                if weak.strong_count() == 0 {
                    break;
                }
                let result = async {
                    if session.is_none() {
                        session =
                            Some(ControlSession::connect(&cfg.control_url, machine.clone()).await?);
                    }
                    if stream.is_none() {
                        stream = Some(
                            session
                                .as_mut()
                                .unwrap()
                                .map_stream_with_routes(
                                    &node,
                                    cfg.hostname.as_deref().unwrap_or("rust-x"),
                                    home,
                                    disco,
                                    &cfg.advertise_routes.iter().map(ToString::to_string).collect::<Vec<_>>(),
                                )
                                .await?,
                        );
                    }
                    stream.as_mut().unwrap().next().await
                }
                .await;
                let Some(shared) = weak.upgrade() else {
                    break;
                };
                match result {
                    Ok(map) => {
                        if let Some(filter) = map.packet_filter.clone() {
                            named_rules.insert("base".to_string(), filter);
                        }
                        if let Some(filters) = &map.packet_filters {
                            if filters.get("*").is_some_and(Option::is_none) { named_rules.clear(); }
                            for (name, value) in filters {
                                if let Some(value) = value { named_rules.insert(name.clone(), value.clone()); }
                                else { named_rules.remove(name); }
                            }
                        }
                        rules = named_rules.values().flatten().cloned().collect();
                        if let Some(full) = &map.peers {
                            peers.clear();
                            for peer in full {
                                peers.insert(peer.id, peer.clone());
                            }
                        }
                        if let Some(changed) = &map.peers_changed {
                            for peer in changed {
                                peers.insert(peer.id, peer.clone());
                            }
                        }
                        if let Some(removed) = &map.peers_removed {
                            for id in removed {
                                peers.remove(id);
                            }
                        }
                        if shared
                            .cmd_tx
                            .send(Cmd::UpdateAccess {
                                rules: rules.clone(),
                            })
                            .await
                            .is_err()
                        {
                            break;
                        }
                        // Changing local identity/address requires a fresh endpoint,
                        // never continue accepting traffic with stale local addresses.
                        if let Some(local) = &map.node {
                            addresses_valid = local.addresses == expected_addresses;
                            shared.routes_approved.store(!cfg.advertise_routes.is_empty() && cfg.advertise_routes.iter().all(|route| local.allowed_ips.iter().any(|a| a.parse::<smoltcp::wire::IpCidr>().ok().as_ref() == Some(route))), Ordering::Relaxed);
                        }
                        let peers = if addresses_valid {
                            peers.values().cloned().collect()
                        } else {
                            vec![]
                        };
                        let _ = shared.cmd_tx.send(Cmd::UpdateLanApproval { approved: addresses_valid && shared.routes_approved.load(Ordering::Relaxed) }).await;
                        if shared
                            .cmd_tx
                            .send(Cmd::UpdatePeers {
                                peers,
                                derp_map: map.derp_map,
                            })
                            .await
                            .is_err()
                        {
                            break;
                        }
                        delay = 0;
                        shared.control_connected.store(addresses_valid, std::sync::atomic::Ordering::Relaxed);
                    }
                    Err(error) => {
                        shared.routes_approved.store(false, Ordering::Relaxed);
                        let _ = shared.cmd_tx.send(Cmd::UpdateLanApproval { approved: false }).await;
                        shared.control_connected.store(false, std::sync::atomic::Ordering::Relaxed);
                        crate::logging::warn(format_args!(
                            "tailscale: map synchronization failed: {}",
                            error
                        ));
                        if shared
                            .cmd_tx
                            .send(Cmd::UpdateAccess { rules: vec![] })
                            .await
                            .is_err()
                        {
                            break;
                        }
                        session = None;
                        stream = None;
                        peers.clear();
                        rules.clear();
                        named_rules.clear();
                        // Fail closed during control outages; snapshots restore peers.
                        if shared
                            .cmd_tx
                            .send(Cmd::UpdatePeers {
                                peers: vec![],
                                derp_map: None,
                            })
                            .await
                            .is_err()
                        {
                            break;
                        }
                        delay = (delay * 2).min(30);
                    }
                }
                drop(shared);
                tokio::time::sleep(Duration::from_secs(delay)).await;
            }
        });
        *guard = Some(Arc::clone(&shared));
        Ok(shared)
    }

    pub async fn connect(&self, target: &Target) -> io::Result<AnyStream> {
        let dst = resolve_target(target).await?;
        let shared = self.shared().await?;
        let id = shared.next_id.fetch_add(1, Ordering::Relaxed);
        let (local, remote) = tokio::io::duplex(CALLER_BUF);
        let (caller_read, caller_write) = tokio::io::split(local);
        let (to_caller_tx, mut to_caller_rx) = mpsc::channel(REPLY_QUEUE);
        let (resp_tx, resp_rx) = oneshot::channel();

        shared
            .cmd_tx
            .send(Cmd::TcpConnect {
                id,
                dst,
                to_caller: to_caller_tx,
                resp: resp_tx,
            })
            .await
            .map_err(|_| io::Error::other("tailscale: engine task stopped"))?;

        // caller → stack
        let cmd_tx = shared.cmd_tx.clone();
        tokio::spawn(async move {
            let mut read = caller_read;
            let mut buf = vec![0u8; SCRATCH_BUF];
            loop {
                match read.read(&mut buf).await {
                    Ok(0) => {
                        let _ = cmd_tx.send(Cmd::TcpEof { id }).await;
                        break;
                    }
                    Ok(n) => {
                        if cmd_tx
                            .send(Cmd::TcpData {
                                id,
                                bytes: buf[..n].to_vec(),
                            })
                            .await
                            .is_err()
                        {
                            break;
                        }
                    }
                    Err(_) => {
                        let _ = cmd_tx.send(Cmd::TcpClose { id }).await;
                        break;
                    }
                }
            }
        });

        // stack → caller
        tokio::spawn(async move {
            let mut write = caller_write;
            while let Some(bytes) = to_caller_rx.recv().await {
                if write.write_all(&bytes).await.is_err() {
                    break;
                }
                let _ = write.flush().await;
            }
        });

        match tokio::time::timeout(CONNECT_TIMEOUT, resp_rx).await {
            Ok(Ok(Ok(()))) => Ok(Box::new(remote)),
            Ok(Ok(Err(e))) => Err(io::Error::other(format!("tailscale: {e}"))),
            Ok(Err(_)) => Err(io::Error::other("tailscale: engine task stopped")),
            Err(_) => Err(io::Error::new(
                io::ErrorKind::TimedOut,
                "tailscale: connect timeout",
            )),
        }
    }

    pub async fn connect_udp(&self, target: &Target) -> io::Result<UdpSocket> {
        let dst = resolve_target(target).await?;
        let shared = self.shared().await?;
        let (local, peer) = udp_loopback_pair().await?;
        let (resp_tx, resp_rx) = oneshot::channel();

        shared
            .cmd_tx
            .send(Cmd::UdpOpen {
                target: dst,
                resp: resp_tx,
            })
            .await
            .map_err(|_| io::Error::other("tailscale: engine task stopped"))?;

        let (port, mut replies) = match tokio::time::timeout(Duration::from_secs(10), resp_rx).await
        {
            Ok(Ok(Ok(v))) => v,
            Ok(Ok(Err(e))) => return Err(io::Error::other(format!("tailscale: {e}"))),
            Ok(Err(_)) => return Err(io::Error::other("tailscale: engine task stopped")),
            Err(_) => {
                return Err(io::Error::new(
                    io::ErrorKind::TimedOut,
                    "tailscale: udp session timeout",
                ));
            }
        };

        let peer = Arc::new(peer);
        let cmd_tx = shared.cmd_tx.clone();
        let udp_timeout = self.cfg.udp_timeout;

        // replies: engine → loopback peer socket
        {
            let peer = Arc::clone(&peer);
            tokio::spawn(async move {
                while let Some(bytes) = replies.recv().await {
                    if peer.send(&bytes).await.is_err() {
                        break;
                    }
                }
            });
        }

        // sends: loopback peer socket → engine
        tokio::spawn(async move {
            let mut buf = vec![0u8; 65_535];
            while let Ok(Ok(n)) = tokio::time::timeout(udp_timeout, peer.recv(&mut buf)).await {
                if cmd_tx
                    .send(Cmd::UdpSend {
                        port,
                        bytes: buf[..n].to_vec(),
                    })
                    .await
                    .is_err()
                {
                    break;
                }
            }
            let _ = cmd_tx.send(Cmd::UdpClose { port }).await;
        });

        Ok(local)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_tailscale_build_and_engine_start() {
        let json_cfg = serde_json::json!({
            "type": "tailscale",
            "tag": "ts-out",
            "auth_key": "tskey-auth-test-key",
            "control_url": "https://controlplane.tailscale.com",
            "ephemeral": true,
            "hostname": "test-box",
            "listen_port": 41641
        });

        let map = json_cfg.as_object().unwrap();
        let ts = Tailscale::build(map, "endpoints[0]").unwrap();
        assert_eq!(ts.cfg.tag, "ts-out");
        assert_eq!(ts.cfg.hostname.as_deref(), Some("test-box"));
        assert!(ts.cfg.ephemeral);

        // Test engine startup
        let (_tx, rx) = mpsc::channel(128);
        let shared =
            engine::spawn_tailscale_engine(ts.cfg.clone(), "100.64.0.1".parse().unwrap(), rx);
        assert_eq!(shared.next_id.load(Ordering::Relaxed), 1);
    }
}
