Rust X 0.0.16

- Add experimental native Tailscale remote access for Forkop 2.3.4 and newer, without a separate tailscaled package. Includes persistent private identity, authenticated control sessions, direct WireGuard and DERP connectivity, LuCI access and optional IPv4 LAN forwarding.
- Enforce Tailscale access policies and approved subnet routes, validate peer packet sources and keep authentication keys out of persistent configuration and logs.
- Stop native remote access and remove its owned network rules when Forkop or the selected core becomes incompatible. Preserve saved identity for subsequent compatible upgrades.
- Authenticate WireGuard transport packets before advancing the replay window, preventing forged packet counters from invalidating legitimate traffic.
- Add OpenWrt package lifecycle hooks for the native service and controlled Forkop restart during core updates.

Verification: 31 focused native Tailscale tests and 7 WireGuard Noise tests passed. OpenWrt 25 VM checks covered LuCI/LAN access, separate Lite support coexistence, reconnect, reboot and Forkop version downgrade cleanup. VLESS, xHTTP, gRPC and Hysteria implementation files are unchanged from 0.0.15; this release does not claim new interoperability coverage for them.
