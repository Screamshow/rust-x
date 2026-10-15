# Native Tailscale: implementation progress

Target: remote access to the router's LuCI through Rust X, without installing
`tailscaled`. The first user interface will be a Tailscale tab in Forkop.

## Step 1: persistent identity

The old implementation saved public keys and subsequently interpreted them as
private keys. It cannot recover the original identity from that file.

New `tailscale.state` files use `Version: 1` and store the private machine and
node keys. Unix files are created with mode 0600. Initialization stages the
complete state in a create-new file, syncs it, renames it, and on Unix syncs the
parent directory. Errors propagate to endpoint initialization instead of
silently starting with a different identity.

Existing malformed, unversioned or unsupported-version files are preserved and
rejected. Resetting an identity must be an explicit operator/user action.
An interrupted write may leave `tailscale.state.new`; initialization reports an
error rather than automatically removing a file that another initializer could
be using. Ephemeral nodes do not read or write persistent state.

Focused verification:

```sh
cargo test --locked --offline -p core-runtime --no-default-features \
  --features rust-x --lib outbound::tailscale::control::tests
```

2026-10-14: all seven focused tests passed on Ubuntu WSL, including restart,
0600 permissions, malformed/legacy state, write failure and ephemeral identity.

The wider `outbound::tailscale` filter also passed all 10 tests (Noise IK
roundtrip, DERP framing, endpoint initialization and the control tests).
These are local component tests, not live control-plane interoperability.
Read-only VM25 diagnostics confirmed VMware OpenWrt 25.12.5 with a running
Rust X binary reporting `v0.0.14-tachyon.0`. No VM packages, services or
configuration were changed. The supplied test auth key was not used: current
endpoint initialization constructs a registration request but never sends it.

## Remaining work after step 1

- Real control-key discovery and authenticated control connection; registration,
  browser login URL and authorization polling.
- Assigned IPv4/IPv6 addresses and live network map updates.
- WireGuard packet transport, peer discovery and DERP fallback.
- Incoming connections to router services with tailnet policy enforcement.
- Endpoint startup independent of outbound traffic, shutdown and reconnect.
- Status/authorization API for the Forkop tab.

The current capability tag and accepted configuration are not evidence of a
working Tailscale connection. No live tailnet registration or remote LuCI access
has been verified by this first increment.

## Step 2: native TS2021 registration (2026-10-14)

Implemented verified HTTPS control-key discovery, the HTTP/1.1 `/ts2021`
upgrade with the framed Noise IK initiation, authenticated record transport,
optional early-payload consumption, and HTTP/2 `/machine/register`.
Transport counters use big-endian encoding as required by controlbase;
handshake nonces retain Noise encoding. Low-order DH responses are rejected.
Connections, registration and initial map requests have timeouts and response
size limits; task ownership cancels the transport when a session is dropped.
Server error bodies are not logged or surfaced because they may echo secrets.

`Tailscale::register()` returns authorization state and AuthURL for a future GUI.
`register_followup()` accepts that URL with persistent identity for browser
authorization; real interactive browser completion remains unverified.
An actual new-identity request without an auth key on VM25 returned
`MachineAuthorized: false` and a nonempty AuthURL. The probe deliberately does
not print the authorization URL. Both live state files have mode 0600.
Authorized nodes fetch an initial uncompressed, length-prefixed network map.
Endpoint initialization now uses its assigned IP instead of a hardcoded VIP.
There is no ongoing map stream or working peer packet transport in this increment.

Live probe: `crates/core-runtime/examples/tailscale_register.rs`, statically
compiled for x86_64 musl. It reads the auth key from stdin and prints only
authorization state, assigned addresses, node ID and whether browser login is
required. An empty line omits the auth key for identity-reuse testing.

VM25 verified actual authorization using the supplied test key, followed by a
new probe process with **no auth key** and the same state directory. The latter
was authorized and received IPv4 `100.106.165.40/32` and IPv6
`fd7a:115c:a1e0::c2a:a529/128`; node ID `8476890468342356`.
Only standalone probe files and key state in `/tmp` were created. The installed
Rust X, separate Tailscale package, Forkop configuration, services and firewall
were not changed. No auth key was written to disk.

Focused musl tests: 18 passed, including real discovery response shape, map
framing, big-endian record nonce, fragmented early payload and registration
AuthURL response through an in-memory HTTP/2 server and rejection of a
low-order responder ephemeral key.

Next: ongoing map updates, WireGuard/DERP packet transport, inbound service
access and lifecycle/status integration. Successful registration does not prove
remote LuCI, SSH, peer reachability, or tailnet policy enforcement.

## Step 3: direct WireGuard data plane (2026-10-14)

`mesh.rs` connects the existing WireGuard primitives to a bounded UDP peer mesh.
Only authenticated peers from the initial map are accepted, and inner packet
sources must match that peer's assigned node addresses. Routed LAN prefixes are
not accepted. Packet destination must be a local node address. Handshake
timestamps and transport replay counters prevent repeated messages; source
endpoint roaming is learned only after successful authentication. Pending
queues, peer/endpoints counts and packet MTU are bounded. Outer UDP currently
uses IPv4; inner packets can be IPv4 or IPv6.

Handshake retries, session expiry, rekey and responder key confirmation are
implemented for this first direct transport. Simultaneous native initiations
use a deterministic key ordering. Cookie replies update the handshake cookie.
This is not a claim of complete WireGuard timer or Tailscale interoperability.

Endpoint initialization retains the same registered node private key and peers
and starts the mesh socket. The userspace stack now sends encrypted IP packets
and consumes authenticated replies. UDP uses actual smoltcp sockets with bounded
buffers, source/port checks, idle expiry and cleanup. TCP backpressure keeps
unaccepted bytes in the socket receive buffer. Stack task exits when its command
senders have closed.

The shared WireGuard primitive now authenticates transport packets **before**
advancing the replay window, so forged high counters cannot poison reception.
Its responder initiation result exposes the authenticated timestamp to the mesh.

Verification: 23 Tailscale tests passed locally and on existing VMware VM25
(OpenWrt 25.12.5), including a complete TCP and UDP echo through smoltcp,
WireGuard handshake, encrypted UDP transport and decryption. Spoofed source IP,
removed peer and malformed IP checks passed. Seven existing WireGuard crypto
tests passed on VM25, including independent reference response vectors and the
forged-counter regression added to responder roundtrip.

These traffic tests use two native Rust endpoints over loopback UDP with test
keys, not a phone or official Tailscale peer in a real tailnet. Installed core,
packages, configuration, firewall and services were preserved. Only standalone
test files in `/tmp` were changed; old registration probe executables were
removed to recover temporary RAM storage, while registered key state was kept.

Remaining before user-facing remote access: publish/discover reachable
endpoints, maintain the live map with reconnect and peer revocation, DERP
authentication/relay, enforce tailnet filters, and implement incoming access to
selected router services. No LuCI or SSH listener is exposed by this increment.

## Step 4: authenticated DERP relay (2026-10-14)

Corrected the placeholder frame IDs and server magic to the actual DERP v2
protocol. Added HTTPS upgrade with certificate verification, server greeting,
nonce-prefixed NaCl client authentication and authenticated ServerInfo, ping/pong,
read/write deadlines and bounded packet queues. `crypto_box` 0.9.1 uses pure Rust
XSalsa20-Poly1305; the lockfile adds only that crate, crypto_secretbox and salsa20.

RelayManager maintains at most eight region connections, reconnects with bounded
backoff, tries up to three nodes per region, and owns/cancels its background
tasks. Connection setup runs outside the network stack task. Reader and writer
failure propagation prevents a dead write side from leaving a nominally live
connection. Congestion drops packets instead of forcing reconnects.

The mesh supports both UDP and DERP paths for the same WireGuard handshake and
transport state. Relay source keys must match the authenticated peer. Modern
HomeDERP and legacy DERP region representations are recognized. Initialization
starts a preferred home relay from the map, with a deterministic first-region
fallback; nearest-region selection is not implemented. Published DERPPort is
recognized, including the zero/default-443 representation.

Live verification on existing VMware VM25 used `tailscale_derp_probe` and the
previously registered node state without an auth key. It obtained the current
DERP map, authenticated two clients to official DERP region 1 and verified
bidirectional relay. A second stage disabled all direct endpoint addresses and
verified bidirectional **WireGuard-encrypted synthetic IP packets** through that
public relay. The second client used a temporary in-memory key; it was not a
new registered tailnet member. This validates DERP protocol compatibility and
mesh relay integration, not tailnet policies or access from a phone.

Focused tests cover NaCl authentication/tampering, invalid greeting and low-order
keys, ping/pong and send/receive framing, as well as existing direct TCP/UDP.
All 26 Tailscale tests passed locally and on VM25. Temporary probe/test binaries
were removed afterward; `/tmp` returned to 71.5 MiB used. The installed core kept
the same PID throughout verification.
No separate Tailscale daemon was started or package installed. The installed
Rust X/Forkop, configuration and firewall were preserved.

Remaining: publish the selected home DERP, maintain/reconnect the live network
map and revoke removed peers, enforce packet filters, and provide incoming access
to selected router services. The current relay test does not prove remote LuCI.

## Step 5: periodic map refresh and preferred relay announcement

The endpoint announces Hostinfo.NetInfo.PreferredDERP on complete map requests.
It refreshes a full snapshot every 15 seconds, reconnecting the control session
with bounded retry after errors. This is polling, not the incremental streaming
map protocol. The updater holds only a weak reference to the endpoint between
requests and stops when its owner disappears.

Every successful snapshot replaces the peer set; removed peers lose their
WireGuard state. A failed request clears all peers until a new snapshot succeeds.
The request itself can take up to 30 seconds before that failure is detected.
If the local assigned addresses change, the engine replaces its userspace IP
addresses and mesh destination checks, drops sessions using the previous IP,
and recreates LuCI/SSH listeners with the current access rules. Status reports
the new address only after the engine acknowledges the change. Address ordering
alone does not reset sessions. Invalid assigned addresses fail closed.
DERP catalog updates cancel connections whose server definitions
changed or vanished and permit fresh connections using the new definitions.

27 focused musl tests passed locally, including an HTTP/2 test verifying the
PreferredDERP announcement and full empty-peer snapshot. Live control-plane
publication/refresh and outage/revocation integration were not verified in this
increment. No VM mutation was needed. Incoming router access and tailnet packet
filter enforcement are still pending.

## Step 6: initial LuCI bridge and network grant evaluation

Added opt-in `luci_access` endpoint configuration (default false), userspace
TCP listeners on node ports 80/443, and forwarding only to the same loopback
ports. No SSH, other ports, or LAN forwarding is enabled. Connection count,
buffers, connect timeout and inactivity timeout are bounded. Loopback targets
are fixed; custom uhttpd ports and binding verification remain integration work.

The bridge checks source/destination TCP network grants from PacketFilter and
PacketFilters on every poll, including existing connections. Empty/missing
filters deny access; control errors clear grants. Supported selectors are
IP/CIDR/range/wildcard and legacy prefix bits. Capability-based source selectors
fail closed. This is a limited network-grant evaluator, not full Tailscale ACL
semantics. HTTP/2 map tests, grant tests and compilation do not establish live
LuCI access. A real official-client/VM25 integration test is still required.

### Official-client integration attempt

On VM25 an isolated official Tailscale 1.98.3 userspace client was started with
in-memory state, loopback SOCKS port 11080, DNS/routes disabled, and a distinct
socket. Authorization succeeded using the test auth key through stdin. The
native standalone web probe used existing registered state and port 41642.
No installed core or primary Tailscale service settings were changed.

The official client listed rust-x-native-test as offline and SOCKS connection
failed before HTTP. LuCI access is therefore **not verified**. Native periodic
non-stream map requests do not establish the persistent online presence needed
for this flow. The official probe also showed offline and its control state
needs diagnosis when repeating the test. Next required change is persistent
streaming map handling, including incremental peer/filter updates.

Logout was attempted but did not complete promptly; both probe processes stopped
and the official client's in-memory credentials were discarded. The temporary
tailnet node may remain visible until removed or expired. The
temporary binaries, socket and logs were removed. Existing native identity was
preserved. Local uhttpd returned LuCI HTML with HTTP 403, which must also be
distinguished from transport failure in the eventual HTTP check.

## Working official-client LuCI integration (2026-10-14)

The native endpoint now maintains a persistent uncompressed TS2021 map stream,
handling fragmented/coalesced frames, full peer lists, PeersChanged and
PeersRemoved, retaining filters on updates that omit them. Address invalidation
is retained across subsequent map messages. Empty/failed control updates remain
fail closed. It separately announces Hostinfo/NetInfo and its home DERP before
opening the stream, using the official Ts-Lb routing header.

Added a per-instance discovery key and authenticated discovery ping/pong against
known peer discovery keys. Direct and relay requests verify the mapped sender;
responses contain the observed endpoint. Unsupported discovery types are ignored.
The mesh answers incoming WireGuard handshakes without proactively initiating
all idle peers. Outstanding outbound initiations still retry. This avoids the
simultaneous-handshake conflict encountered during official-client testing.

On existing VM25, stock OpenWrt Tailscale 1.98.3 was used as an isolated userspace
**test client**, with SOCKS bound to loopback, DNS/routes disabled and independent
temporary socket/state. The deployed feature uses only Rust X; no package was
installed. Official-client startup with mem: repeatedly failed to reach Running;
temporary persistent state and daemon restart reached Running. HTTPS control was
forced for that test client after observed control port-80 resets.

Verified through official-client SOCKS and public DERP relay nyc:

- Native `http://100.106.165.40/cgi-bin/luci/`: HTTP 403, 2801-byte LuCI login form.
- Native HTTPS LuCI: identical response and HTML.
- Native HTTP root and luci-static/resources/luci.js: HTTP 200.
- Native HTTPS luci-static/resources/luci.js: HTTP 200.
- Remote/local login HTML SHA-256 both
  `76b9a7f146190aeb695ced2b03faf872f73be249817115c40cf9070a6134c12a`.
- Port 22 connection rejected; no SSH banner received.
- Probe restarts re-established access with the saved identity.

HTTP 403 here is the local unauthenticated LuCI login response, not a relay or
bridge error. Login with a router password was not attempted. This is official
client interoperability on VM25, not a physical phone/mobile-network test.

30 focused musl tests passed locally. A local diagnostic empty-grant injection
was restored by subsequent valid control grants, so it is **not** evidence of
server-side ACL revocation. Real tailnet-policy revocation, long-run stability,
modern PeersChangedPatch handling, custom uhttpd ports and GUI remain work.

Forkop was briefly stopped to isolate network interference; this broke the VM's
configured DNS. A temporary controlplane/login hosts entry was used; Forkop was
restarted and hosts restored byte-for-byte afterward. Forkop UCI SHA-256 remained
`aeb9e1e593c3cd6d46c79e7759a65fa459305e397b3ba797b91ecb936f1b5468`.
Temporary test client credentials and executables were removed after testing;
the native node identity in /tmp/rust-x-ts-native-state remains for further tests.

## Native client metadata and local IP reassignment

Hostinfo is consistent across registration, initial map, announcement and streaming requests. IPNVersion carries a numeric interoperability baseline plus the explicit rustx suffix and actual Rust X build version; it does not identify this independent implementation as the official Tailscale daemon. The negotiated capability level remains separate.

For OpenWrt, preserve the linux OS family of existing node identities. Set distribution/release fields from os-release/openwrt_release and include OpenWrt with its release in the displayed OSVersion. The hosted console ignored the proper distribution fields alone. Changing OS to OpenWrt/openwrt on the existing test identity resulted in no assigned addresses, including after reauthorization; do not silently perform that identity change in an upgrade.

A health map may legitimately contain Node.Addresses=null. Parse nullable node arrays as empty and report that the server did not assign an address rather than treating the response as a malformed map and indefinitely waiting for internet connectivity. Diagnostic status exposes only fixed local error reasons. On a map parsing failure, a root-only volatile type/structure snapshot omits string values.

Actual address changes update the userspace interface and mesh destination checks, close old sessions, and recreate incoming listeners with current access rules. Address ordering is normalized. Live x86_64 VM testing verified hosted-console IPv4 change and reversal with LuCI access and unchanged identity, followed by a real reboot. This does not establish all upstream Tailscale features or approved-LAN forwarding compatibility.
