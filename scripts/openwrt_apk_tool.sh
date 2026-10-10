#!/bin/bash
set -euo pipefail
mkdir -p work/sdk
base=https://downloads.openwrt.org/releases/25.12.3/targets/x86/64
sdk=openwrt-sdk-25.12.3-x86-64_gcc-14.3.0_musl.Linux-x86_64.tar.zst
curl -fL --retry 3 "$base/$sdk" -o work/sdk.tar.zst
curl -fL --retry 3 "$base/sha256sums" -o work/sdk.sha256sums
expected=$(awk -v name="$sdk" '{sub(/^\*/, "", $2)} $2 == name {print $1}' work/sdk.sha256sums)
test -n "$expected"
echo "$expected  work/sdk.tar.zst" | sha256sum -c -
tar --zstd -xf work/sdk.tar.zst --strip-components=1 -C work/sdk
echo "APK_TOOL=$PWD/work/sdk/staging_dir/host/bin/apk" >> "$GITHUB_ENV"
