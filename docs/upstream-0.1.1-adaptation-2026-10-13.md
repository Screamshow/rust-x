# Rust X: safe selective adaptation of tachyon-core 0.1.1

Base: Rust X main b8c35b4. Sources: Dushnilin/tachyon-core v0.1.1 d9979e1,
particularly b72a7c8 (HTTP delay), bcca432 (xHTTP lifecycle) and d87c1d9
(UDP lifecycle). Upstream authorship and GPL-3.0-only are preserved.

## Changes

- Manual Clash delay honors an explicit http:// URL and defaults to port 80;
  HTTPS still defaults to port 443. Automatic URLTest and TLS verification
  remain unchanged. The API's existing zero-ms failure contract remains.
- Split-mode xHTTP sends/flushed GET headers, then reads the response in its
  owned download task. Upload can start before a GET response, avoiding a
  deadlock against servers that await the first POST/VLESS payload. HTTP
  header/body failures reach the reader; dropping the stream aborts its pump.
  The download queue remains four messages, not the upstream increase to 16.
- Each xHTTP upload has an xmux Lease. It releases usage synchronously on
  success, error or task cancellation, and retires a connection whose response
  could not be completed. No-IO idle clients are removed by pool acquisition.
  A lease that failed to take another request's socket does not close that
  socket. Completed keep-alive bodies can still return their IO to the pool.
- No automatic replay of an ambiguous POST failure: a server may already
  have accepted the sequence/payload. The error is surfaced and the pooled
  connection is retired; a subsequent attempt opens a fresh connection.
- Dedicated UDP hubs in stream datagram relay, inbound mux and TUN get an
  aborting StopHub owner. Existing shared direct/tproxy listener guards remain.
- UDP backlog sends are bounded by the flow timeout and interrupted by the
  tracker close signal. A failed send clears the remaining backlog and closes
  its tracker. An ownership-aware guard resets writing on cancellation without
  clearing a flag reclaimed by another writer. Upload counters still increase
  only for successfully sent bytes.
- Hysteria2 and MASQUE UDP bridge activity now considers both directions, so
  reverse traffic keeps a receive-only flow alive. Existing configured/default
  timeout values and Hysteria2 BBR selection are retained.

## Deliberate exclusions

xHTTP auto remains packet-up. Explicit transport modes, TLS/REALITY stack
policy, H2 windows/send buffers, relay pools, allocator, library BLAKE3,
Vision UUID, h2mux defaults/concurrency algorithm, protocol profile, package
versions and release workflow remain unchanged. No upstream TLS downgrade
cache or Brutal controller is added. Router::prepare, acknowledged reload,
route revalidation, Discord colon ranges and UDP metrics are preserved.

## Regression coverage

New cases cover a GET response blocked until upload arrives, rejected download
headers, failed POST without replay followed by a fresh connection, cancelled
xmux usage/IO, a dedicated UDP relay releasing its socket after EOF, and actual
HTTP HEAD to the URL requested through Clash API. Existing keep-alive xHTTP,
Mux header/short-write, UDP accounting/activity, reload, rule-set and URLTest
TLS tests remain in the verification set.

Local WSL/Linux verification passed:

- Strict workspace clippy, all targets with legacy-protocols, `-D warnings`.
- Runtime library: 724 passed, 9 existing ignored microbenchmark/network cases.
- Clash API: 5; hot reload: 9; compiled rule-set: 13; URLTest TLS integration:
  1; Rust X configuration profile: 3 — all passed.
- Existing transports interop and Hysteria2 TCP roundtrip passed against the
  local sing-box-lx 1.14.2-lx.11 reference. TUIC was intentionally filtered out.
- Full workspace Rust X cargo check and aarch64 musl Rust X core-runtime
  cargo check passed. Reduced feature checks retain existing dead-code warnings.
- Formatting and diff whitespace checks passed.

Logs: parent workspace tmp/upstream011-tests.log and tmp/upstream011-interop.log.
These checks cover hub/socket cleanup and transport interoperability, but do
not represent a long-duration receive-only Hysteria2/MASQUE UDP voice test,
an ARM runtime test, a complete signed package build or GitHub CI result.
No VM/router package installation, GitHub release or live voice-quality
validation is performed by this task.
