# Post-handshake delay validation

Manual Clash API and background URLTest now report the HTTP response interval
after destination TLS completes. The complete operation still uses the original
timeout, including dial/TLS failures. API formats, retry/status rules, URLTest
tolerance and Forkop priority selection rules are unchanged. This is HTTP
response latency, not ICMP RTT. Plain HTTP behavior is retained.

Validation on 2026-10-11:

- Six focused tests passed: Clash API, Clash views and URLTest timing.
- The native static integration test passed on both existing OpenWrt VMware
  VMs (25 and 24): delayed TLS was excluded in manual, group and background
  tests; response delay remained; a TLS deadline returned HTTP 504.
- Real VLESS/REALITY nodes were downloaded with Forkop's normal HWID headers
  and normalized by Forkop. Manual tests, provider URLTest, positive history,
  manual selector updates and failed-node errors passed on both VMs.
- The real Forkop priority worker passed fallback, recovery to a higher-priority
  node and failover after that node failed, with within-level fastest selection
  enabled, on both VMs.
- VM24 baseline manual samples: 276/577/150 ms; candidate: 138/147/138 ms.
  VM25 baseline: 239/280/158 ms; candidate: 146/461/150 ms. Network variability
  remains; excluding TLS cannot remove response-time spikes.

An initial test configuration routed DNS recursively through the tested
selector and overflowed the baseline runtime stack. Explicit direct DNS fixed
that fixture; it was not a timing regression. One VM25 candidate group test
failed on its first network run and the complete candidate run passed on retry.

Candidates ran from `/tmp` on loopback ports without package installation;
normal routing remained active during successful network checks. Installed
package inventories were unchanged and normal Forkop remained healthy.
Subscription credentials, URLs, HWIDs and private logs are excluded here.
No new package version or release was published.
