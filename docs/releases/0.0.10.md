Rust X 0.0.10

- Apply compatible configuration changes in place, preserving listeners and established connections; reject changes to immutable startup settings before changing runtime state.
- Expose authenticated applied-config acknowledgments with PID, generation and SHA-256, and a read-only reload-plan CLI for Forkop's guarded reload transaction.
- Retain DNS response and upstream-address caches only when DNS configuration is unchanged, so new resolvers and hosts entries take effect after reload.
- Correct Mux request headers across partial writes and Pending, accept compound URLTest duration units, and reuse bounded PacketAddr framing buffers.

Verified with focused reload, DNS, FakeIP, Clash API, Mux, duration and PacketAddr tests; strict workspace Clippy; x86_64 and aarch64 musl release builds; and real Forkop/OpenWrt 25 reload, list add/remove, rollback and subscription HTTPS tests. Forkop X 2.2.9-canary.1 integrates the acknowledgment protocol. Listener changes require restart. Brutal/WireGuard and full padded Mux interoperability are outside this increment.
