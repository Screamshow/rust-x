# BLAKE3 library API extension

Vendored blake3 1.8.7 from the crates.io source already used by Rust X.
Original authorship and licenses are preserved. Two byte-context entry points
are added: `derive_key_bytes` and `hazmat::hash_derive_key_context_bytes`.
The existing string entry points delegate to them. Compression, SIMD/portable hashing, tree handling,
and key-material derivation remain the original library implementation.
This avoids constructing an invalid Rust str from binary VLESS context bytes.
Keep this small API change when updating the library; do not replace the hash
implementation with tachyon-core's handwritten module.
