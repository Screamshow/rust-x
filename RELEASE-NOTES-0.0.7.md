- Fix Forkop connection monitoring: capture the exact matched domain, suffix, keyword, regex or subnet in Clash API rulePayload.
- Preserve exact matches inside logical rules with device exclusions and long domain lists.
- Only report evidence from the selected OR branch; inverted conditions keep an honest fallback.
- Verified real Clash API output on OpenWrt 25 for chatgpt.com and 2ip.io.

Rust X remains experimental.
Thanks to Dushnilin/tachyon-core. GPL-3.0-only.
