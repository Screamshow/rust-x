Rust X 0.0.12

- Add authenticated connection revalidation after a successful routing apply, closing affected TCP and UDP sessions when their destination or outbound changes or their destination rule no longer matches.
- Support removed inline domains and IP/CIDRs, removed rule sets, and changed entries inside a rule set without changing its tag. Preserve sessions still served by an equivalent active route.
- Isolate tracked sessions by router instance; evaluate a coherent routing snapshot with bounded concurrency and timeouts, preserving uncertain sessions and rejecting superseded evaluations.
- Track UDP flows with close signals so the next datagram opens a new session under the applied policy. Do not retain sniffed payload buffers in the connection registry.
- Advertise connection-revalidation capability alongside the existing authenticated PID/generation/configuration-hash acknowledgment.

Verified with 9 hot-reload tests, 9 UDP lifecycle tests, authenticated Clash API tests and strict core-runtime Clippy; x86_64 and aarch64 musl release builds; and real TCP/UDP routing-change tests on the OpenWrt 25 GL-MT6000 router. Forkop X 2.2.9-canary.8 integrates the new API. Real TikTok TLS moved from the Germany section to Poland in one apply, closing the old session and preserving independent TCP. Kernel-forwarded direct connections that bypass Rust X are intentionally outside revalidation. Listener changes still require restart; Brutal/WireGuard and a full subscription-protocol or long-running memory matrix are outside this increment.
