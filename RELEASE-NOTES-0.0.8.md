Rust X 0.0.8

- Fix XHTTP auto and packet-up transfers, asynchronous uploads, TLS response flushing and leftover-byte buffering.
- Preserve local DNS access and skip interface binding and routing marks for loopback destinations.
- Accept AnyTLS inbound passwords from the users list.
- Authenticate HTTPS URLTest certificates and handshake signatures; exclude TLS setup from the reported HTTP response delay while retaining the overall timeout.
- Release drained TCP relay buffers while waiting for data or flush, preserving backpressure and half-close behavior.
- Release UDP stream bridge sockets after EOF or errors and expire abandoned tunnels using shared activity from either direction; preserve per-flow udp_timeout overrides.
- Improve UDP sniffing, route resolution and cryptographic safety checks adapted from upstream.
- Preserve APK signatures when release signing is retried.

Verified with focused tests and existing OpenWrt VMware diagnostics. UDP cleanup fixes two confirmed socket-retention mechanisms; it does not establish that all long-running RSS growth reported in upstream issue 21 is resolved. No claim of full real-subscription gRPC or latest post-quantum REALITY compatibility is made.
