#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    core_runtime::outbound::tailscale::service::run().await
}
