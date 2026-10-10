# Selected upstream 0.0.4 fixes for Rust X

Source: Dushnilin/tachyon-core v0.0.4, commit 8eb4de4. Selected work by
Dushnilin (d042b39, eb9315c, b84e3b7) and MaksSt (94efaf3 / PR #9).
GPL-3.0-only and upstream authorship are retained.

Included:

- Compound headless rule-set AND semantics and indexed/interpreted parity,
  including catch-all rules and domain/IP groups.
- Proxy dial defaults (routing mark and interface), respecting explicit
  outbound settings and Rust X legacy-protocols feature gates.
- XHTTP HTTP/1.1 empty response completion and pool reuse/open-usage fixes.
- DoH request Content-Length, with a local HTTP/2 transport regression test.
- XUDP address/port wire ordering and VLESS Mux request header; server XUDP
  support, repeated datagrams and session cleanup.
- UDP weak socket ownership, flow expiry/cleanup and TProxy reply sockets
  using the original destination address, including receive-only media.
- Reusable TCP/UDP binds and supported listener error recovery.

Our DNS evaluate/respond, remote rule-set registry, host-address handling,
CLI rule-set match fix and runtime reload implementation are preserved.
No QUIC/Hysteria2 changes, bootstrap fallback, new platforms, version bump
or release publication are included. Existing optional Shadowsocks and other
legacy transports retain their feature gates; shared UDP types required
updating the optional Shadowsocks inbound too.

Validation:

- Focused default-profile lib tests: rule-set 13, DNS 54, XHTTP 15, xmux 6,
  packet framing 9, UDP hub 8, route defaults 2; all passed.
- VLESS secure tests 6; inbounds 12; compiled rule-set 10; VLESS server
  integration 16; direct UDP reload 1; all passed.
- Transparent UDP/TProxy tests repeated as root: 2 passed without skipping.
  Socket SO_MARK getsockopt regression repeated as root: passed.
- Optional legacy-protocols cargo check passed.
- Static x86_64-musl and aarch64-musl release builds checked.
- Existing OpenWrt 25 VM 192.168.1.1: seven DNS scenarios (two query IDs),
  seven reload gates and remote JSON/SRS rule-set scenarios passed against
  an isolated candidate. Installed packages and production UCI preserved.

An initial unfiltered default-profile test attempt was not a passing gate:
it exposed legacy-protocol-dependent fixtures, unprivileged SO_MARK and
the old XUDP fixture, then was stopped. The XUDP code/fixture pair is now
ported and passes; SO_MARK passes as root. No claim of a complete all-profile
compatibility audit is made.

Evidence in the Forkop workspace: tmp/upstream004-focused.log,
tmp/upstream004-secure.log, tmp/upstream004-integration.log,
tmp/upstream004-vless.log, tmp/upstream004-udp-integration.log,
tmp/upstream004-legacy-check.log, tmp/upstream004-build.log,
tmp/upstream004-arm-build.log and tmp/upstream004-vm25-*-results.json.
