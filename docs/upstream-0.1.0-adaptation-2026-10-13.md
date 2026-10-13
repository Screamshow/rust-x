# Rust X: selective tachyon-core 0.1.0 adaptation

Base: Rust X main `99abc54`. Upstream sources are Dushnilin's commits
`a572c60`, `d75ed1a`, `48eec78`, `1dbda76` and `6a3dc7d` from
https://github.com/Dushnilin/tachyon-core/releases/tag/v0.1.0.
Changes are adapted rather than imported as a full merge. GPL-3.0-only and
upstream authorship are retained. No version bump or release is included.

## Runtime changes

- H2 receive-window capacity returns when the caller consumes bytes, not
  merely when they arrive. Header/trailer consumption in gRPC and unread
  bytes on stream Drop repay the shared window. Existing window sizes remain.
- CacheFile avoids dirty/pending updates for identical values and missing-key
  deletion. Clash Mode startup does not rewrite the same value. Remote 304
  timestamps use the normal flush policy. Identical TSRS content is compared
  with 8 KiB scratch space rather than another complete ruleset allocation.
  Inline/local compiled TSRS use atomic replacement, keeping old mappings
  valid during Router::prepare. Temporary names are unique for those writes.
- Clash /traffic reports bytes per actual elapsed second, handles counter
  reset, and avoids burst samples after delayed ticks. /connections retains
  cumulative totals and reports Linux RSS using the system page size.
- DNS UDP has at most two retransmissions in addition to its initial query,
  still inside the configured transport timeout. Bootstrap resolves the
  transport endpoint once, then queries both address families concurrently.
  Successful responses get 50 ms grace for the other family. JoinSet owns
  cancellation of unfinished queries. Failed refresh preserves a known
  transport-server address; this does not enable stale DNS-answer serving.
- DoH H2 is reused within one ServerConfig and resolved endpoint. Changed
  configuration, same-tag replacement, and different endpoints do not share
  a global tag cache. Driver tasks have an aborting owner, including failed
  setup and concurrent first-query races.
- FakeIP restores RAM maps, repairs missing reverse records, removes
  conflicting/invalid/orphan records and uses binary-key iteration. Paired
  LRU eviction and IP reuse cleanup are applied through CacheFile batches.
  Startup validates address family as well as pool range; reverse cleanup
  inspects the repaired maps, not the still-unmodified disk view. Existing
  cache migration and binary metadata compatibility remain.
- WireGuard key lifetime follows session start. Poll scheduling observes
  smoltcp's deadline. Caller data enters the TCP socket before stack polling.
  EOF sends FIN after queued application data is accepted by the socket.
- TUN preserves the remainder of partial/failed writes, reserves channel
  capacity before draining received TCP bytes, bounds packet queues and
  stores one cancellable send reservation instead of spawning wait tasks on
  every Full poll. Existing per-socket buffers remain 64 KiB per direction.

## BLAKE3 and exclusions

The upstream handwritten BLAKE3 is excluded. The original blake3 1.8.7
library is vendored with only byte-context API wrappers in lib.rs/hazmat.rs.
The hash/compression implementation and optimizations are unchanged; original
authors and licenses are preserved. VLESS encryption no longer constructs an
invalid str with from_utf8_unchecked. See vendor/blake3/RUST-X-PATCH.md.

Proxy detour implementation, Vision UUID removal, allocator replacement,
smoltcp buffer growth, new fastest-IP/stale-answer/DoH-interception features,
pin-only TLS trust policy, upstream CI reduction and release metadata are
excluded. Existing acknowledged reload, Router::prepare and TCP/UDP route
revalidation are retained. Mux/duration work discussed before this release
is not part of this adaptation.

## Focused regression coverage

New checks cover unread H2 backpressure and reopening the window; elapsed
traffic rates and RSS; no-op cache dirty/pending state and file mtime; TSRS
no-op and validity of an old mapping after replacement; fast-family recovery
and cancellation; failed bootstrap refresh; DoH connection reuse and same-tag
replacement; missing FakeIP reverse records, corrupt keys and family-local
paired eviction; full TUN channels and FIFO byte order after partial/error
writes; and a binary BLAKE3 derive-key vector plus UTF-8 parity.

Local WSL/Linux verification:

- Workspace clippy with legacy-protocols/all-targets and `-D warnings`: passed.
- Runtime library: 717 passed, 9 pre-existing ignored microbenchmarks/network
  tests. The subsequently added failed-bootstrap-refresh test passed separately.
- CacheFile: 10; cache interoperability: 1; DNS resolver: 18; FakeIP
  persistence: 6; hot reload/connection revalidation: 9 — all passed.
- Rust X configuration profile: all 3 passed.
- Transports and WireGuard interoperability passed with the existing local
  sing-box-lx 1.14.2-lx.11 reference. This was an actual protocol run; an earlier
  transports invocation without SING_BOX_REF returned early and is not counted
  as interoperability validation.
- aarch64-unknown-linux-musl Rust X core-runtime `cargo check`: passed with
  the repository's existing target crypto flags. This is a compile check,
  not an ARM runtime or complete package/linking test.
- Comparing the vendored BLAKE3 with the original registry source confirms
  only lib.rs and hazmat.rs differ; compression, SIMD/portable code, build
  script and license files are unchanged.

- Full workspace Rust X `cargo check`, x86_64-unknown-linux-musl workspace
  `cargo check`, `cargo fmt --all -- --check` and `git diff --check`: passed.

Logs are in the parent workspace's tmp/upstream010-tests.log,
tmp/upstream010-final.log and tmp/upstream010-arm.log.
No VM/router installation, signed release,
performance measurements, privileged OpenWrt lifecycle run or GitHub CI run is
included. Reduced Rust X feature checks retain existing dead-code warnings for
unused protocol helpers; strict legacy-profile clippy has no warnings.
