Rust X 0.0.15

- Fix DoH HTTP/2 recovery after DNS timeouts: evict stalled connections on errors or cancellation so subsequent queries can reconnect without a reload, while preserving healthy connection reuse.
- Enforce configured certificate and public-key SHA-256 pins for ordinary rustls TCP and QUIC connections, and isolate pinned configurations in the bounded TLS cache.
- Fix Clash API lookup, selection and delay requests for proxy and group names containing encoded slashes.
- Make Unix TCP/UDP sockets nonblocking before connect, close sockets on protection failures, and apply the configured TCP connect timeout on Windows.
- Support authenticated Ed25519 REALITY X.509 v1 certificates while retaining handshake signature and authentication checks.
- Add pooled HTTP/2 multiplexing for xHTTP packet-up and stream-up, fix pool wakeups and cancellation cleanup, and improve upload flow control and shutdown completion. Preserve auto mode, bounded download queues and the existing policy against replaying ambiguous failed POST requests.
