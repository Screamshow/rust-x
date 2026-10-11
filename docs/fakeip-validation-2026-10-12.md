# FakeIP regression validation — 2026-10-12

Added the LRU eviction regression from upstream [PR #23](https://github.com/Dushnilin/tachyon-core/pull/23) to Rust X. With persistent storage disabled, allocating beyond the 16,384-entry capacity must remove the evicted domain's forward and reverse mappings for both IPv4 and IPv6, while preserving the newest mapping.

Validation on the existing Rust X main checkout:

```sh
cargo +stable fmt --all -- --check
cargo +stable test --locked --offline --target x86_64-unknown-linux-musl \
  -p core-runtime --no-default-features --features rust-x --test fakeip_persistence
```

Formatting passed. All four integration tests passed, with zero failures or ignored tests (1.25 seconds): binary-key persistence and cursor recovery, Go-style metadata and pool wrap, idle/shutdown flush and unknown FakeIP rejection, and LRU eviction. The reduced Rust X feature profile emitted 14 existing unused/dead-code warnings; this was a focused test run, not a strict Clippy run.

For comparison, upstream PR #23 passed all seven CI jobs on its first submitted commit: Linux, Windows, differential, and x86_64/ARM64/ARMv7/MIPS musl builds. Those are upstream results, separate from the Rust X checks above.

This follow-up changes only tests and this validation record. Runtime fixes remain those released in Rust X 0.0.9; no new release is required.
