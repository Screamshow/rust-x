//! Live registration probe. Auth key is read from stdin, never argv or logs.
use core_runtime::outbound::tailscale::Tailscale;
use std::io::{self, BufRead};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let state = std::env::args()
        .nth(1)
        .ok_or("state directory argument required")?;
    let mut key = String::new();
    io::stdin().lock().read_line(&mut key)?;
    let mut config = serde_json::json!({
        "type": "tailscale", "tag": "ts-probe", "hostname": "rust-x-native-test",
        "state_directory": state, "auth_key": key.trim(), "ephemeral": false
    });
    if key.trim().is_empty() {
        config.as_object_mut().unwrap().remove("auth_key");
    }
    let endpoint = Tailscale::build(config.as_object().unwrap(), "tailscale")?;
    let response = endpoint.register().await?;
    println!(
        "{}",
        serde_json::json!({
            "authorized": response.machine_authorized,
            "requires_browser_login": response.auth_url.as_ref().is_some_and(|u| !u.is_empty()),
            "addresses": response.node.as_ref().map(|n| &n.addresses),
            "node_id": response.node.as_ref().map(|n| n.id)
        })
    );
    Ok(())
}
