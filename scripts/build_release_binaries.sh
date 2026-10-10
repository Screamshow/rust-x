#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."
rustup target add x86_64-unknown-linux-musl aarch64-unknown-linux-musl
export CC_x86_64_unknown_linux_musl="${CC_x86_64_unknown_linux_musl:-musl-gcc}"
for target in x86_64-unknown-linux-musl aarch64-unknown-linux-musl; do
    cargo build --locked -p core-cli --no-default-features --features rust-x --release --target "$target"
done
bash scripts/pack_all_architectures.sh "${1:-}"
