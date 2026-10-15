Rust X 0.0.19

- Fix native Tailscale IP changes without manual reconnection, updating remote access and closing connections bound to the previous address.
- Report the native Tailscale client version and OpenWrt release consistently, enabling IP changes in the admin console while preserving existing device identities.
- Handle server responses without an assigned address and show an authorization or compatibility error instead of endlessly waiting for connectivity.
