//! Optional Linux TUN for approved subnet forwarding; absent in LuCI-only mode.
use super::access::{Rule, allows_protocol};
use smoltcp::wire::IpCidr;
use std::{io, net::IpAddr};
pub fn permitted(packet: &[u8], routes: &[IpCidr], rules: &[Rule]) -> bool {
    let Some((src, dst, len)) = super::mesh::packet_ips(packet) else {
        return false;
    };
    if !routes.iter().any(|r| r.contains_addr(&dst.into())) {
        return false;
    }
    if !matches!(dst, IpAddr::V4(_))
        || packet[0] != 0x45
        || len < 20
        || packet[6] & 0x3f != 0
        || packet[7] != 0
    {
        return false;
    }
    let proto = packet[9] as u16;
    let port = match proto {
        6 if len >= 40 => u16::from_be_bytes([packet[22], packet[23]]),
        17 if len >= 28 => u16::from_be_bytes([packet[22], packet[23]]),
        1 if len >= 28 => 0,
        _ => return false,
    };
    allows_protocol(rules, src, dst, port, proto)
}
#[cfg(target_os = "linux")]
pub struct LanTun {
    fd: std::os::fd::OwnedFd,
}

#[cfg(target_os = "linux")]
impl LanTun {
    pub fn open() -> io::Result<Self> {
        use std::os::fd::{AsRawFd, FromRawFd};
        let fd = unsafe {
            libc::open(
                c"/dev/net/tun".as_ptr(),
                libc::O_RDWR | libc::O_NONBLOCK | libc::O_CLOEXEC,
            )
        };
        if fd < 0 {
            return Err(io::Error::last_os_error());
        }
        let fd = unsafe { std::os::fd::OwnedFd::from_raw_fd(fd) };
        let mut req: libc::ifreq = unsafe { std::mem::zeroed() };
        for (i, b) in b"forkop-tslan".iter().enumerate() {
            req.ifr_name[i] = *b as libc::c_char;
        }
        req.ifr_ifru.ifru_flags = (libc::IFF_TUN | libc::IFF_NO_PI) as libc::c_short;
        if unsafe { libc::ioctl(fd.as_raw_fd(), libc::TUNSETIFF as _, &req) } < 0 {
            return Err(io::Error::last_os_error());
        }
        Ok(Self { fd })
    }
    pub fn write(&self, bytes: &[u8]) -> io::Result<()> {
        use std::os::fd::AsRawFd;
        let n = unsafe { libc::write(self.fd.as_raw_fd(), bytes.as_ptr().cast(), bytes.len()) };
        if n < 0 {
            return Err(io::Error::last_os_error());
        }
        if n as usize != bytes.len() {
            return Err(io::Error::other("short TUN write"));
        }
        Ok(())
    }
    pub fn read(&self) -> io::Result<Option<Vec<u8>>> {
        use std::os::fd::AsRawFd;
        let mut bytes = vec![0; 1280];
        let n = unsafe { libc::read(self.fd.as_raw_fd(), bytes.as_mut_ptr().cast(), bytes.len()) };
        if n < 0 {
            let e = io::Error::last_os_error();
            return if e.kind() == io::ErrorKind::WouldBlock {
                Ok(None)
            } else {
                Err(e)
            };
        }
        bytes.truncate(n as usize);
        Ok(Some(bytes))
    }
}
#[cfg(not(target_os = "linux"))]
pub struct LanTun;
#[cfg(not(target_os = "linux"))]
impl LanTun {
    pub fn open() -> io::Result<Self> {
        Err(io::Error::other("LAN forwarding requires Linux"))
    }
    pub fn write(&self, _: &[u8]) -> io::Result<()> {
        Ok(())
    }
    pub fn read(&self) -> io::Result<Option<Vec<u8>>> {
        Ok(None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn subnet_acl_rejects_other_destinations_protocols_and_fragments() {
        let rules: Vec<Rule> = serde_json::from_value(serde_json::json!([{"SrcIPs":["100.64.0.0/10"],"IPProto":[17],"DstPorts":[{"IP":"192.168.1.0/24","Ports":{"First":53,"Last":53}}]}])).unwrap();
        let routes = vec!["192.168.1.0/24".parse().unwrap()];
        let mut packet = vec![0; 28];
        packet[0] = 0x45;
        packet[2..4].copy_from_slice(&28u16.to_be_bytes());
        packet[9] = 17;
        packet[12..16].copy_from_slice(&[100, 64, 0, 2]);
        packet[16..20].copy_from_slice(&[192, 168, 1, 2]);
        packet[22..24].copy_from_slice(&53u16.to_be_bytes());
        assert!(permitted(&packet, &routes, &rules));
        assert!(!permitted(&packet, &routes, &[]));
        packet[22..24].copy_from_slice(&54u16.to_be_bytes());
        assert!(!permitted(&packet, &routes, &rules));
        packet[22..24].copy_from_slice(&53u16.to_be_bytes());
        packet[16] = 8;
        assert!(!permitted(&packet, &routes, &rules));
        packet[16] = 192;
        packet[6] = 0x20;
        assert!(!permitted(&packet, &routes, &rules));
    }
}
