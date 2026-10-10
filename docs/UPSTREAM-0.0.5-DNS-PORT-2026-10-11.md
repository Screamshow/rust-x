# Selected upstream 0.0.5 DNS fixes

Adapted from Dushnilin/tachyon-core commit
03f0db726aa808f7bff44a4a1a9ec0a47b51a96d (v0.0.5), authored by Dushnilin.
GPL-3.0-only and upstream authorship are retained.

Rust X now keeps successful domain_resolver transport-address results for
60 seconds. The cache stores the selected IP; each caller applies its own
port. Concurrent resolutions share a lock per (resolver tag, host), rather
than upstream's global lock. Unrelated hosts/resolvers remain independent.
Expired entries and dead weak lock entries are pruned on insertion.
Recursive resolver chains skip locking and retain the existing depth guard
to avoid waiting on an ancestor's lock.

lookup_via collects A and AAAA independently. A successful family survives
failure of the other; an empty result with an error returns that error.
Failed results are not cached. Configured domain_resolver failures still
propagate: no automatic fallback to system DNS was added. Existing system
resolution for servers without domain_resolver is unchanged.

New regression tests cover 32 concurrent resolutions sharing one request,
cache reuse/expiry, distinct caller ports, unrelated host progress while
another key is locked, resolver cycles, either family failing, and failed
lookup retry without cache/fallback. All 57 focused DNS tests passed.

Static x86_64-musl build passed. On the existing OpenWrt 25 VM at
192.168.1.1 an isolated candidate served 32 simultaneous DNS queries plus
one cached follow-up. Only one bootstrap A and one AAAA request were sent;
AAAA deliberately returned malformed data, while all queries succeeded via
the A address. Production UCI hashes and installed packages were preserved.
The first fixture attempt omitted the required route.default_domain_resolver
and was rejected before startup; the corrected fixture passed.

No Mieru, protocol-profile changes, version bump or release publication.
Evidence in Forkop workspace: tmp/upstream005-dns-tests.log,
tmp/upstream005-build.log and tmp/upstream005-bootstrap-vm-results.json.
