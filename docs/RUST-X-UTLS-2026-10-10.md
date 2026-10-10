# Rust X uTLS refresh

Scope: Firefox 148, Safari 26.3, and the current Chrome profile used for XHTTP,
measured from our Sing-Box X 1.0.3 build. Existing Chrome aliases use the updated
Chrome table. Other browser and random profiles were not expanded in this work.
See the capture directory's README and provenance.json for donor credits,
reference package/binary hashes and fixture hashes.

Firefox preserves reuse of the hybrid X25519 key in its classical share. Chrome
and Safari preserve independent keys. All three keep X25519MLKEM768 in supported
groups and key shares; REALITY already advertised client version 26.3.27.

The native TLS client now accepts RFC 8879 compressed certificates (zlib,
Brotli and Zstd), using pure-Rust decoders, only for advertised algorithms.
It checks compressed/decompressed lengths and caps decompressed certificates at
1 MiB. Certificate validation and Finished verification remain enabled, and the
transcript hashes the compressed handshake message as required by RFC 8879.

Validation:

- Four fingerprint tests, including reconstruction of captured hellos, hybrid
  key/share consistency, Firefox key reuse and configured ALPN, passed.
- All 47 TLS library unit tests and eight independent rustls handshake tests
  passed, including all three certificate compression algorithms and rejection
  of invalid certificates/names.
- Strict Clippy passed for all core-tls targets on x86_64 musl; formatting and
  git diff whitespace checks passed.
- The updated primary VM25 core passed all 42 existing subscription group entry
  requests (the selector includes an automatic group entry), each HTTP 200 with
  577 bytes from example.com.
- Explicit profile checks on the primary VM25 service passed: Firefox and Safari
  on TCP/REALITY and gRPC; Chrome on both XHTTP nodes. Six HTTPS requests returned
  HTTP 200 and 577 bytes.

The profile check temporarily used cloned outbound definitions in the VM's
generated configuration. Forkop regenerated its regular configuration afterward.
No subscription credentials were exported to the source repository.
These checks do not establish compatibility with every Xray version/target,
TLS 1.2, real ECH, session resumption or long-running stability.


Final reduced x86_64 musl binary: 11,185,488 bytes (160,256 bytes above the
previous 11,025,232-byte build, mainly certificate decoders). SHA-256:
`bf90f14c6f5afb43baa6095348dd6b3d426699b7f4cc5ff6a42570b9640a6032`.
All 42 group-entry requests and all six explicit profile combinations were
repeated successfully on this final binary. Temporary mixed proxy settings
were removed and Forkop returned to its regular configuration.
