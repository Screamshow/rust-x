//! Run an isolated native LuCI endpoint using existing local node identity.
use core_runtime::outbound::tailscale::Tailscale;
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    core_runtime::logging::init(None, Some("debug"), false);
    let state = std::env::args().nth(1).ok_or("state directory required")?;
    let config = serde_json::json!({"type":"tailscale","tag":"web-probe",
        "hostname":"rust-x-native-test","state_directory":state,
        "listen_port":41642,"luci_access":true});
    let endpoint = Tailscale::build(config.as_object().unwrap(), "tailscale")?;
    let _shared = endpoint.shared().await?;
    println!("native endpoint initialized");
    if std::env::args().nth(2).as_deref() == Some("deny-after") {
        let tx = _shared.cmd_tx.clone();
        tokio::spawn(async move {
            tokio::time::sleep(std::time::Duration::from_secs(20)).await;
            let _ = tx
                .send(
                    core_runtime::outbound::tailscale::engine::Cmd::UpdateAccess { rules: vec![] },
                )
                .await;
            println!("test: LuCI grants revoked locally");
        });
    }
    tokio::time::sleep(std::time::Duration::from_secs(180)).await;
    Ok(())
}
