Rust X 0.0.9

- Reject unknown FakeIP destinations before TCP or UDP routing can send a synthetic address to the network.
- Persist and restore FakeIP mappings and allocation cursors using sing-box-compatible cache records and metadata for IPv4 and IPv6.
- Preserve packed binary address keys when reading and rewriting cache databases; update paired mappings and cursors together.
- Flush pending cache writes during idle periods and clean shutdown, and release reverse-map entries evicted from the bounded in-memory cache.
- Set the packaged OpenWrt sing-box service's open-file limit to 65535 to avoid inheriting the default 4096 ceiling.

Verified with focused FakeIP, cache, DNS and routing tests, a release build, and the existing OpenWrt 25.12.5 VMware VM. VM checks cover cache restart, missing-record rejection, allocation cursor metadata and the procd open-file limit. The complementary own-traffic bypass fix for combined mwan3 marks is released in Forkop X 2.2.8-canary.1. Cache-format fixtures do not establish full live cross-variant migration or complete protocol compatibility.
