# Rust X: selective tachyon-core 0.1.2 adaptation

Date: 2026-10-14. Base Rust X main: 0beb310. Upstream source: v0.1.2 / 7376350003d5ef4ff29e3bf32844f345192fd930, compared with v0.1.1 / d9979e1. Upstream authorship and GPL-3.0-only are preserved. Existing selective 0.1.1 fixes from 0353cbb remain.

Changes:

- Ordinary rustls TCP and QUIC now enforce configured certificate and public-key SHA-256 pins in addition to CA validation. Both pin lists participate in the existing bounded TLS config cache key. Explicit insecure behavior remains unchanged.
- Clash API classifies raw URL paths and decodes proxy/group names after segmentation. Encoded slash names support lookup, selection and delay. Static UI paths still reject decoded traversal.
- Unix sockets are nonblocking before connect and Linux sockets have CLOEXEC. Ordinary TCP/UDP dial avoids a blocking worker; namespace switching retains its worker. Protection failures close the fd. Windows TCP connect receives the configured timeout.
- REALITY accepts the authenticated Ed25519 X.509 v1 leaf shape; CertificateVerify remains cryptographically checked. Pin verifier also supports the rustls ServerCertVerifier interface. Negative signature/auth-key and malformed certificate coverage remains.
- xHTTP packet-up and stream-up can multiplex GET/upload streams over a pooled HTTP/2 connection. HTTP/1 fallback reuses the negotiated connection. Auto remains packet-up; explicit modes are preserved.
- H2 pool registers Notify before checking state; dial reservations and stream counters have Drop cleanup on cancellation/error. Pool metadata uses a short synchronous mutex, never held over await, to allow synchronous cleanup. Opening a failed session is not automatically replayed. HTTP/1 xmux leases and the no-ambiguous-POST-replay policy remain.
- H2 upload uses available send capacity before waiting for more. Download queues remain four entries. Shutdown flushes packet uploads and preserves the download/ack tasks until normal completion or connection Drop.

Excluded: hardware profiles, enlarged memory pools/windows, automatic TLS 1.2 downgrade, Brutal, fixed 15-second half-close timeout, protocol profile expansion, release/package/version changes, deployment. Forkop UCI, applied-config generation acknowledgment and route revalidation remain unchanged. No packages or services on VMs/routers are changed.

Regression coverage includes CA-trusted wrong/matching pins over real local TCP TLS and QUIC handshakes, TLS cache separation, encoded slash REST lookup/selection/delay, raw socket flags, bounded dial completion, REALITY v1 with wrong signature/auth key, cancellation/wakeup of H2 reservations, and H2 packet-up/stream-up wire round trips including half-close followed by download. Existing HTTP/1 POST-no-replay and cancellation tests are preserved.

Local WSL/Linux verification:

- core-tls: 88 tests passed, including X.509 v1 and negative signature/auth-key checks.
- Runtime library: 741 passed, 9 existing ignored microbenchmark/network cases. Includes real TCP/QUIC pin handshakes, H2 packet-up/stream-up, dial flags/time limit and cancellation tests.
- Clash API REST: 6; hot reload: 9; relay splice: 5; relay tracking: 4; URLTest TLS: 1; Rust X profile: 3 -- all passed.
- Reference sing-box-lx 1.14.2-lx.11 transports interoperability and Hysteria2 TCP roundtrip passed. TUIC was filtered out.
- Rust X workspace cargo check and aarch64-unknown-linux-musl core-runtime check passed; the reduced profile retains 14 existing dead-code warnings.
- Formatting and diff whitespace checks passed.

The first new slash-name REST test incorrectly expected a delay field for a refused destination. It was corrected to assert the existing 503 error contract; the final REST suite passed. Product behavior was not changed to accommodate that expectation.

Logs: parent workspace tmp/upstream012-final-tests.log (core TLS/runtime), tmp/upstream012-remaining-tests.log (remaining checks), tmp/upstream012-interop.log (reference/ARM). H2 split-mode wire tests use in-process peers; live Xray/REALITY subscription and post-quantum compatibility, Windows execution, VM installation and GitHub CI are not claimed.

Strict workspace clippy with all targets, legacy-protocols and -D warnings passed.
