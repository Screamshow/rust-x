# DoH HTTP/2 timeout recovery

A silent HTTP/2 stall survived DNS query deadlines because the outer timeout
dropped the exchange future before the explicit-error branch removed the pooled
connection. Every later query could reuse that same stalled connection. Reload
rebuilt the DNS service and happened to clear it.

The fix wraps both reused and newly published HTTP/2 exchanges in a Drop guard.
Successful requests disarm it. Errors and cancellation evict only the captured
connection ID, preserving any newer replacement installed by another request.
Normal connection reuse is retained. The expired query still fails at its
deadline; subsequent requests can establish a healthy connection.

Verification with the Rust X profile:

- `cargo test --locked --offline -p core-runtime --no-default-features --features rust-x --lib doh_h2_ -- --nocapture`: 3 passed.
- `cargo test --locked --offline -p core-runtime --no-default-features --features rust-x --test dns_resolver doh_ -- --nocapture`: 3 passed. The recovery test exercises both a fresh and a warmed connection and verifies reuse of the replacement.
- `cargo fmt --all -- --check` and `git diff --check`: passed.
- `cargo build --locked -p core-cli --no-default-features --features rust-x --release --target x86_64-unknown-linux-musl`: passed.
- Existing OpenWrt 25.12.5 and 24.10.8 VMware VMs: published 0.0.14 reproduced three consecutive timeouts until reload; the isolated patched binary recovered on the next query after one timeout, without reload, on both VMs.

The VM test used a local controlled TLS HTTP/2 server and a trusted test
certificate. It does not identify the original cause of the user's network
interruption. No release or operator-router installation was performed.
