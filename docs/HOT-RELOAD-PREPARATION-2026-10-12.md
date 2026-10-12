# Rust X: targeted fixes and hot reload preparation

Implemented on the existing main checkout, following review of upstream
tachyon-core 0.0.9. No version bump, release publication or Forkop integration
is part of this change.

## Changes

- MuxStream's initial header/payload write now counts only accepted payload
  bytes. Partial header writes and Pending retain header progress; a
  header-only write continues without returning a false WriteZero. Tests
  cover short header, short payload, Pending, empty writes and real zero writes.
  This does not establish complete padded Mux interoperability.
- Duration parsing accepts compound Go units (`1m30s`, `2s500ms`, microseconds
  and nanoseconds), retaining the existing numeric nanosecond and bare-number
  string handling. Invalid floating-point values return errors rather than
  panicking. Negative unit durations saturate to zero as in upstream.
- Packet framing writes directly into reusable scratch storage, with an
  explicit bound on the complete packetaddr body. Scratch grows on demand;
  it is not preallocated to the maximum datagram size for every idle bridge.
  Existing EOF and idle cancellation remain unchanged. No RSS claim is made.
- In-place reload rejects changes to inbounds, Clash API, cache-file settings,
  NTP and Linux namespace configuration before preparation. The CLI can use
  its existing restart path for such changes.
- Remote rulesets and endpoints are still prepared before the live router is
  replaced. Invalid configs/downloads/local lists preserve the previous router
  and applied-config acknowledgment.
- DNS answer and resolver-address caches migrate only when the DNS section
  fingerprint is unchanged. Resolver tags alone are not identities: replacing
  a server behind the same tag previously reused its cached answer. FakeIP
  pools and valid reverse mappings migrate separately as before.

## Applied-config acknowledgment

`GET /rust-x/applied-config` is a read-only extension to the existing Clash
controller. It uses the controller's existing authentication and CORS policy.
Example response:

```json
{
  "schema": 1,
  "pid": 1234,
  "generation": 2,
  "config_sha256": "<64 hexadecimal characters>",
  "ready": true,
  "hot_reload": true
}
```

Generation 1 is published only after startup completes. Each successful
in-place reload increments it, even when JSON remains identical and a local
ruleset file changed. Failed preparation leaves the acknowledgment unchanged.
Generation resets on a new runtime instance; it is not a persistent counter.
Forkop must retain its process identity checks and restrict this contract to
in-place transitions. A stopped/restarted instance is a different transition.

The fingerprint is SHA-256 of `serde_json::to_vec` of the parsed/merged config:
compact JSON, sorted object keys in the current build, preserved array order.
It is not the hash of the original file bytes, and it excludes contents of
external ruleset files. Consumers should compare the generation as well as
the fingerprint. The endpoint returns neither config contents nor errors
containing credentials. `ready` confirms completed apply, not upstream DNS or
proxy connectivity; normal runtime probes remain required.

The endpoint does not request reload and does not replace SIGHUP. Future
Forkop integration must separately coordinate nft/config generations and
rollback. Kernel WG, Brutal and automatic changes to Forkop are excluded.

## Verification

- General legacy runtime library: 707 passed, 9 ignored.
- Rust X focused Mux: 8 passed; duration: 4 passed; packet: 13 passed.
- Final Rust X Hot Reload integration: 8 passed, including changed local
  rules applying to new connections while an established TCP flow survives,
  failed remote download, broken local list, invalid DNS, selector/history/
  FakeIP preservation, restart-only rejection and acknowledgment behavior.
- Clash API: 2 passed, including authentication and acknowledgment response.
- FakeIP persistence: 4 passed; URLTest timing and TLS: 1 each passed.
- Strict workspace/all-targets legacy Clippy, Rust X workspace check and
  rustfmt passed. An initial unfiltered Rust X library run hit two inherited
  tests that require excluded Shadowsocks/SOCKS; those are not Rust X passes.
- A regression test against real loopback UDP resolvers failed before DNS
  migration correction (`203.0.113.7` instead of `.8`) and passed afterward.
- Standalone musl binary on existing OpenWrt 25.12.5 VMware VM: startup/API,
  real SIGHUP DNS change, unchanged PID, surviving TCP echo, rejected malformed
  config, same-config generation advance and listener release on TERM passed.
  Test files and process were removed. Installed Forkop/sing-box packages,
  sing-box PID 22834 and config SHA-256 remained unchanged.

VM binary SHA-256:
`8a9dcb631ea1e3bb6c3f5edd25c007918406a62c0786813e303e03ad17ed14d4`.
The standalone VM check is not an installed-package upgrade, firewall
transaction test, real subscription compatibility test or performance result.

Operator evidence under the parent workspace `tmp/`: `dns-reload-red.log`,
`rust-reload-check.log`, `rust-reload-final-check.log`,
`rust-reload-vm-results.json`, `rust-reload-vm-run.log`, and
`rust-reload-vm-{before,after}.txt`.
