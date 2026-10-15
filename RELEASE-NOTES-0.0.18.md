Rust X 0.0.18

- Reduce Clash API `/traffic` CPU and allocation overhead by reading byte counters directly instead of collecting connection metadata on every update.
- Reuse the decrypted TLS handshake record buffer, removing one allocation per record.
