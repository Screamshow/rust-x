# Selective upstream 0.0.2 port

Date: 2026-10-10. Rust X retains its current version and reduced protocol profile.

Sources:

- Dushnilin/tachyon-core `7db0fdaae8029c257d236f709947d239078a02ba`: DNS RCODE names, connected UDP exchange, and DNS rule-set miss handling.
- Dushnilin/tachyon-core `0e39528c7d48e33b33fab601ae0d44b4772822c5`: independent XHTTP compatibility test.

The RCODE parser is adapted from upstream, with checked numeric conversion to
prevent values above 255 from wrapping. UDP exchange uses `ErrorKind::NotConnected`
instead of hardcoded platform error numbers. Direct sockets are connected before
exchange; already connected router/relay sockets retain their actual peer.
DNS rule-set misses reject the rule instead of falling through to a match, and
matching sets no longer bypass the remaining conditions.

Added regression coverage for RCODE names/numbers and invalid values, rule-set
hits/misses/inversion/additional conditions, and local UDP exchanges with and
without a router.

Intentionally excluded: automatic system DNS fallback, incomplete
evaluate/match_response implementation, upstream version/packaging/workflow
changes. Existing Vision and TLS implementations already match the corresponding
upstream code and were not replaced.

Validation on x86_64 GNU/Linux, stable Rust:

- Three reduced-profile configuration tests and one XHTTP compatibility test passed.
- Eight DNS rule unit tests passed on the reduced profile.
- All 16 DNS resolver integration tests passed with legacy-protocols enabled,
  including UDP, TCP, DoT, DoH, cache, FakeIP, and router wiring.
- Formatting and strict core-runtime/core-config library Clippy checks passed.

No VM installation, package update, release, or cross-architecture validation was
performed. Existing reduced-profile dead-code warnings remain.
