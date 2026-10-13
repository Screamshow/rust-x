# Discord UDP rule-set routing fix

Input diagnosis: parent workspace docs/router90-discord-rust-x-diagnostic-2026-10-13.md.
Observed tuple: 192.168.90.201 -> 104.29.155.207:19314/UDP; generated rule
contains network=udp, ip_cidr=104.16.0.0/12, port_range=19294:19344.

## Changes

- Shared rule-set range parsing accepts colon syntax and retains historical
  hyphen syntax. Colon open endpoints use 0/65535, as ordinary route-rule
  parsing does. Reversed, malformed and overflowing ranges do not match.
  Historical malformed empty hyphen endpoints remain rejected.
- Interpreted, indexed, logical fallback, source-port and TSRS destination-port
  parsing use the same helper. Existing network/IP/port conjunctions are retained.
- route.final explicitly records final plus the final outbound tag, replacing
  a previously recorded nonterminal sniff/resolve action. Terminal matches
  still retain their own rule metadata.
- UDP trackers count successfully sent payload bytes on initial delivery,
  fast path and backlog drain. Reverse payload bytes update download counters.
  Framing bytes and queued-but-unsent/dropped datagrams are not upload traffic.

## Tests

New cases cover the observed Discord tuple, both endpoints of the range,
outside ports, outside IP, wrong network, logical fallback, both delimiters,
source-port/open ranges and malformed values. A router-level datagram case
selects dis_youtube-out for 19314 and final/direct-out for 19345 after sniff.
TSRS tests check colon and legacy port-range boundary matching. Local UDP
socket tests check first, fast, queued and reverse byte accounting.

Validation passed locally in WSL/Linux: strict workspace clippy with
legacy-protocols/all-targets; 13 compiled rule-set tests, 9 hot-reload tests,
10 UDP unit tests, and 14 interpreted/remote rule-set unit tests (46 total).
Rust X workspace cargo check, formatting check and git diff --check passed.
The reduced Rust X profile retains its pre-existing dead-code warnings.
Log: parent workspace tmp/discord-ranges-tests.log.

This is a source fix; no package was installed or service changed on the operator router.
Discord voice quality and live-router recovery require a subsequent deployment
and retest, and are not inferred from local tests.
