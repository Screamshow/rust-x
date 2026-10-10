- Fixed sequential DNS evaluate, response IP rule-set matching and respond.
- Preserved evaluated responses across nested logical rules and client bypass.
- Shared remote DNS rule sets with the runtime router, including downloaded SRS.
- Reload configuration on SIGHUP with listener cleanup and recovery after failed startup.
- Added focused DNS, remote rule-set and reload regression tests.

Rust X is an independent GPL-3.0-only fork for Forkop, based on
Dushnilin/tachyon-core. This release retains the experimental integration
and existing reduced protocol profile.
