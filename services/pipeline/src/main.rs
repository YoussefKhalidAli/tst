use crate::router::create_router;
use tracing::info;
use tracing_subscriber::{EnvFilter, fmt};

mod pipeline;
mod router;

// Also set in Dockerfile
const PORT: u16 = 6969;
pub const REGISTRY_URL: &str = "zot.registry.svc.cluster.local:5000";
pub const BUILD_NAMESPACE: &str = "build-pipeline";
// Subject to change (using namespace for each app)
pub const APPS_NAMESPACE: &str = "apps";

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .with_target(true)
        .init();

    info!(port = PORT, "starting pipeline service");

    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{PORT}")).await?;
    info!(addr = %listener.local_addr()?, "tcp listener bound");

    let app = create_router();
    info!("router initialized; serving requests");

    axum::serve(listener, app).await?;

    Ok(())
}
