# Conservative outgoing Vision Direct

The client replies with PaddingDirect only after its peer has already entered Direct and a complete inner TLS application-data record is available. The final padded frame is fully written through outer TLS before raw writes begin. Pending ciphertext prevents the switch. Connections without peer confirmation retain PaddingEnd. This does not implement independently initiating Direct or expand server TLS wrapper support.

Focused verification: 15 Vision tests, two TLS buffer/transition tests and three relay-buffer tests passed. The TLS test checks pending ciphertext refusal, original plaintext byte accounting and raw bytes after the switch. Standalone release CLI builds successfully. Full CI and ARM runtime verification were not performed locally.

Local x86_64 OpenWrt 24/25 VMware comparison, 2026-10-16: four concurrent verified TLS1.3 HTTPS uploads over VLESS TCP REALITY Vision to Sing-Box X1.0.4; local HTTPS and local REALITY disguise target. Two trials per variant per VM in before/after/after/before order, 2 GiB per trial, 16 GiB total. Same release toolchain/profile, allocator, buffer sizes and one worker pinned to guest CPU0; no concurrent compilation. Throughput 999.73–999.85 Mbit/s without transfer errors.

| Client VM | Process CPU before / after, % of one vCPU | Whole VM busy before / after, % of two vCPUs |
| --- | ---: | ---: |
| OpenWrt24 | 81.74 /43.21 | 53.02 /29.55 |
| OpenWrt25 | 81.00 /41.52 | 52.45 /29.28 |

This is a 47–49% relative process CPU reduction for this eligible upload workload, not a general protocol/download/ARM claim. Only two observations per variant; host physical-core affinity not fixed. RSS differences were small and do not establish a memory improvement. TLS1.2 fallback and simultaneous upload/download smoke checks passed on both VMs. No production package was replaced for these measurements.

A separate EOF investigation found outer TLS close_notify emitted by the reference REALITY server after switching to raw Direct when the inner HTTPS origin closes without close_notify. Its ciphertext authenticated with outer rather than inner TLS keys. The issue reproduces before this change and with a Sing-Box client. A correctly shutting-down rustls HTTPS origin passes strict EOF checks. This patch does not filter alerts or hide that server shutdown issue; no test traffic secrets or configurations are included here.
