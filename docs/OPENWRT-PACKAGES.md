# Rust X OpenWrt packages

Rust X 0.0.1 has APK `0.0.1-r1` for OpenWrt 25 and IPK `0.0.1-1`
for OpenWrt 24. Architectures: `aarch64_cortex-a53` (ARM64) and `x86_64`.
The payload uses the existing release binaries without UPX compression.

The package is `rust-x`, provides `sing-box` and conflicts with other
sing-box package variants. Dependencies: `ca-bundle`, `kmod-tun`.
The native executable lives at `/usr/lib/rust-x/sing-box`; the wrapper
at `/usr/bin/sing-box` supplies the existing Forkop configuration capability
baseline while reporting Rust X identity separately. This baseline does not
claim every Sing-Box feature or protocol. Existing configuration is retained;
the included procd service is disabled by default on fresh installations.

APK assets are unsigned, intended for explicit local installation with
`--allow-untrusted`. They are not a signed mirror feed. Existing Forkop
defaults, component selectors and mirror update policy are unchanged.

`scripts/package_openwrt.py` validates ELF machine identity and produces
both formats, hashes and `openwrt-manifest.json`. The separate packaging
workflow can attach packages to an existing binary release; future core
releases package both formats automatically. The original 0.0.1 core tag
remains unchanged; `openwrt-packaging-source.tar.gz` supplies the added
packaging source from main.

On 2026-10-10, real x86_64 APK/IPK installations on the existing OpenWrt 25/24
VMs preserved `/etc/config/sing-box`, passed native configuration checks,
started the normal sing-box service through Forkop and reached watchdog ready.
The APK installation passed all 42 subscription group-entry HTTPS requests.
ARM64 package architecture and metadata were checked; ARM64 execution was
not tested because both available VMs are x86_64.
