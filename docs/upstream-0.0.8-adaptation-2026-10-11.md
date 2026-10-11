# Upstream 0.0.8 adaptation

Selected upstream fixes from Dushnilin/tachyon-core:

- `898a11c`: bypass interface binding/mark and foreign source binding for loopback; local DNS without an explicit detour.
- `921672d`: AnyTLS inbound accepts passwords from `users`.
- `5c5284e`: HTTP inbound transport, XHTTP leftover buffering and dependent TLS/server/config support.
- `107bc0e`: XHTTP auto resolves to packet-up, asynchronous packet upload and inbound flushes, with multi-write tests.

Upstream authorship and GPL-3.0-only are preserved. Benchmark rewrites,
desync, version changes and architecture expansion were not imported.
Existing generic x86-64 baseline, authenticated URLTest, relay memory release
and explicit rule-set soft-start policy are retained.

## UDP socket retention / upstream issue 21

Two new regression tests reproduce a concrete defect: after remote TCP EOF,
the packet bridge (used by VLESS/Trojan) and UOT bridge leave their UDP receive
direction waiting indefinitely and retain a connected loopback socket.
Both tests failed before the fix. The bridges now run both directions in one
task with `select!`, dropping the other direction when either ends.
Tests repeatedly close bridges and verify that their bound UDP ports become
available again (32 UOT cycles, 32 cycles for each packet encoding).

This proves the EOF retention defect, not the full cause of issue 21's
seven-hour RSS growth. A dropped local UDP endpoint does not itself send EOF
to the remote TCP stream; idle-only lifecycle and other outbound bridges are
outside this focused correction. Do not describe issue 21 as fully resolved.
Installed router packages and production configuration are not changed.

## Verification

- Runtime library: 697 passed, 9 ignored; transport integration: 48 passed.
- Authenticated URLTest regression passed. The manual relay-memory benchmark
  is ignored by the standard test invocation, not counted as executed.
- Both EOF socket-release regressions failed before the correction and passed
  afterward, including the existing OpenWrt 25 VMware VM.
- VM package/service baseline was captured first. Only a standalone test
  executable in `/tmp` ran on loopback; no package/service/config changes.
  Initial debug upload exhausted tmpfs (the partial file was removed); the
  repeat used a stripped executable, verified SHA-256
  `f3e712e64f2068c2b373b16094f2e2da5a8aaece3e7034fc4487c947523b9864`.
  The repeat passed, the file was removed, and Forkop remained running.
- Evidence: operator workspace `tmp/upstream008-{red,tests}.log`,
  `tmp/upstream008-vm25-{before,tests}.log` (baseline ends in `.txt`).
