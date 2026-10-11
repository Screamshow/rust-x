# UDP bridge idle cleanup

Follow-up to upstream issue 21 and the EOF cleanup in `f350b11`.
The UOT and VLESS/Trojan packet bridges previously retained their loopback
socket and two stream halves when the caller's UDP endpoint disappeared but
the remote TCP tunnel remained open. Connected UDP receives no EOF when its
peer is closed. Stopping UdpHub's reverse reader alone does not stop these
independent outbound tasks.

Both bridges now use a shared sliding idle deadline. Default: the existing
120-second UDP policy. Packet arrival in either direction refreshes it;
expiration cancels both pumps and drops their socket/stream state. UdpHub
scopes outbound setup with the selected per-flow timeout, preserving inbound
and route-options `udp_timeout` overrides. Task-local state is read before
spawning the bridge; unrelated setup uses the default.

Tests cover 16 abandoned UOT tunnels and 16 tunnels of each packet encoding,
with remote stream halves deliberately kept open; repeated receive-only and
send-only traffic across multiple idle windows; and the scoped timeout.
Short test deadlines avoid waiting 120 seconds per cycle. EOF regressions
remain. No allocator changes or installed package replacements are involved.

This correction covers these two bridges, not every UDP-capable outbound.
It is a concrete match for stale loopback sockets but does not establish
that it alone explains issue 21's reported seven-hour RSS slope. Long-running
production verification remains necessary before claiming the issue resolved.

Verification on the final source:

- Runtime library: 702 passed, 9 ignored; server transports: 48 passed.
- Strict core-runtime Clippy with all targets and warnings denied passed;
  formatting and diff whitespace checks passed.
- Authenticated URLTest and five TLS server handshake tests passed.
- Existing OpenWrt 25 VMware VM: all 12 packet and 6 UOT tests passed.
  Package/service state was captured before copying the test into `/tmp`.
  SHA-256 matched locally and on the VM:
  `c2668d79e41447addbfb9c73488d7eaefc10e184fae1c3cc840a0ee08c54f9d4`.
  Test binary was removed; Forkop remained running. No installed packages,
  UCI, firewall, service lifecycle or production router were changed.
- Evidence: operator workspace `tmp/udp-idle-final.log`,
  `tmp/udp-idle-tests.log`, `tmp/udp-idle-vm25-before.txt`,
  `tmp/udp-idle-vm25-tests.log`.
