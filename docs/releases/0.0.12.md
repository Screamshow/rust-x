Rust X 0.0.12

- Fix Discord UDP routing for rule sets using colon-separated port ranges; apply the same range parser to compiled and indexed rules, expose final-route metadata and count successfully transferred UDP payloads.
- Return HTTP/2 receive capacity as bytes are consumed, preserve unread stream accounting and avoid unnecessary cache or rule-set writes. Replace compiled rule sets atomically so prepared routing snapshots retain valid mappings.
- Improve DNS retry and bootstrap cancellation, reuse DoH connections within their configuration and endpoint, and repair persisted FakeIP forward/reverse records without changing cache compatibility.
- Preserve queued TCP bytes across TUN backpressure and partial writes; improve WireGuard polling, session key lifetime and EOF handling.
- Honor explicit HTTP URLs in manual Clash delay checks. Allow split-mode xHTTP upload before the download response, release pooled upload connections on failure or cancellation, and avoid replaying ambiguous POST requests.
- Stop dedicated UDP hubs with their owning session, bound backlog sends and retain receive-only Hysteria2/MASQUE flows while reverse traffic remains active.

Verified with GitHub CI, strict workspace Clippy, Rust X configuration-profile tests, hot-reload and rule-set regression tests, and x86_64/aarch64 musl builds. Forkop X 2.3.1-canary.1 retains authenticated PID/generation/configuration-hash acknowledgment and TCP/UDP connection revalidation. Explicit transport modes, xHTTP auto policy, TLS/REALITY policy, protocol profile and memory budgets remain unchanged. A full real-subscription matrix, ARM runtime validation and long-duration UDP voice tests are outside this release verification.
