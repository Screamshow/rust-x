- Preserve logical route conditions and sniffed host in connection monitoring.
- Restore upstream protocol defaults; keep the explicit Rust X release profile.
- Rust X remains experimental.

Thanks to Dushnilin/tachyon-core. GPL-3.0-only.

- Port upstream 0.0.6 hot reload, DNS UDP retransmission and process-stable random uTLS profiles.
- Preserve active TCP sessions, selector/delay state and DNS/FakeIP caches on SIGHUP.
- Stage remote rule-set downloads before switching the live router; failed preparation preserves the running configuration.
- Preserve upstream Trojan fixes in the full protocol profile.
