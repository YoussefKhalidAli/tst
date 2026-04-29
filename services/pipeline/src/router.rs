use crate::{
    REGISTRY_URL,
    pipeline::{create_build_job, watch_and_deploy},
};
use axum::{
    Json, Router,
    extract::{Path, State},
    response,
    routing::{get, post},
};
use dashmap::DashMap;
use kube::Client;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::{error, info, warn};

#[derive(Serialize, Clone)]
#[serde(rename_all = "lowercase")]
pub enum BuildStatus {
    Building,
    Deploying,
    Running { url: String },
    Failed { reason: String },
}

pub struct AppState {
    pub status: BuildStatus,
}

pub type StatusStore = Arc<DashMap<String, AppState>>;

pub fn create_router() -> Router {
    let state: StatusStore = Arc::new(DashMap::new());

    Router::new()
        .route("/deploy", post(deploy_handler))
        .route("/status/{job_name}", get(status_handler))
        .with_state(state)
}

#[derive(Deserialize)]
pub struct DeployRequest {
    pub git_url: String,
    pub git_ref: String,
    pub app_name: String,
    pub app_port: u16,
}

#[derive(Serialize)]
struct DeployResponse {
    job_name: String,
    status: BuildStatus,
}

async fn deploy_handler(
    State(store): State<StatusStore>,
    Json(req): Json<DeployRequest>,
) -> response::Result<Json<DeployResponse>> {
    info!(
        app_name = %req.app_name,
        git_ref = %req.git_ref,
        git_url = %req.git_url,
        app_port = req.app_port,
        "received deploy request"
    );

    let client = Client::try_default().await.map_err(|e| {
        error!(error = %e, "failed to initialize kube client");
        "Couldn't initialize Kube client"
    })?;

    let job_name = create_build_job(
        &client,
        REGISTRY_URL,
        &req.app_name,
        &req.git_url,
        &req.git_ref,
    )
    .await
    .map_err(|e| {
        error!(
            error = %e,
            app_name = %req.app_name,
            git_ref = %req.git_ref,
            "failed to create kaniko build job"
        );
        "Failed to run Kaniko job"
    })?;

    info!(
        job_name = %job_name,
        app_name = %req.app_name,
        "build job created; setting status=building"
    );

    store.insert(
        job_name.clone(),
        AppState {
            status: BuildStatus::Building,
        },
    );

    let job_name_clone = job_name.clone();
    tokio::spawn(async move {
        info!(job_name = %job_name_clone, "spawned background build/deploy watcher");
        watch_and_deploy(client, job_name_clone, req, store).await
    });

    info!(job_name = %job_name, "responding to deploy request");

    response::Result::Ok(Json(DeployResponse {
        job_name,
        status: BuildStatus::Building,
    }))
}

async fn status_handler(
    State(store): State<StatusStore>,
    Path(job_name): Path<String>,
) -> Json<Option<BuildStatus>> {
    let status = store.get(&job_name).map(|s| s.status.clone());

    match &status {
        Some(BuildStatus::Building) => {
            info!(job_name = %job_name, status = "building", "status lookup")
        }
        Some(BuildStatus::Deploying) => {
            info!(job_name = %job_name, status = "deploying", "status lookup")
        }
        Some(BuildStatus::Running { url }) => {
            info!(job_name = %job_name, status = "running", url = %url, "status lookup")
        }
        Some(BuildStatus::Failed { reason }) => {
            warn!(job_name = %job_name, status = "failed", reason = %reason, "status lookup")
        }
        None => warn!(job_name = %job_name, "status lookup for unknown job"),
    }

    Json(status)
}
