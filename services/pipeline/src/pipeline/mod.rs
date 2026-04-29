use crate::pipeline::deploy::{create_deployment, create_http_route};
use crate::pipeline::watch::wait_for_job;
use crate::router::{AppState, BuildStatus, DeployRequest, StatusStore};
use crate::{APPS_NAMESPACE, REGISTRY_URL};
use kube::Client;
use tracing::{error, info, info_span, instrument};

pub use build::create_build_job;

mod build;
mod deploy;
mod watch;

#[instrument(
    name = "pipeline.watch_and_deploy",
    skip(client, store, req),
    fields(job_name = %job_name)
)]
pub async fn watch_and_deploy(
    client: Client,
    job_name: String,
    req: DeployRequest,
    store: StatusStore,
) {
    let orchestration_span = info_span!(
        "pipeline.orchestration",
        app_name = %req.app_name,
        git_ref = %req.git_ref,
        app_port = req.app_port
    );
    let _enter = orchestration_span.enter();

    info!("waiting for build job completion");
    let success = wait_for_job(client.clone(), job_name.clone()).await;
    info!(build_succeeded = success, "build job watcher completed");

    if !success {
        error!("kaniko build failed, marking job as failed");
        store.insert(
            job_name.clone(),
            AppState {
                status: BuildStatus::Failed {
                    reason: "Kaniko build failed".into(),
                },
            },
        );
        return;
    }

    info!("build succeeded, transitioning status to deploying");
    store.insert(
        job_name.clone(),
        AppState {
            status: BuildStatus::Deploying,
        },
    );

    let image = format!("{REGISTRY_URL}/{}:{}", req.app_name, req.git_ref);
    info!(%image, namespace = APPS_NAMESPACE, "starting deployment apply");

    let http_route_url =
        match create_deployment(&client, &req.app_name, req.app_port, &image, APPS_NAMESPACE).await
        {
            Ok(_) => {
                info!("deployment applied successfully, creating httproute");
                match create_http_route(&client, &req.app_name, APPS_NAMESPACE).await {
                    Ok(url) => url,
                    Err(e) => {
                        error!(error = %e, "httproute creation failed, marking job as failed");
                        store.insert(
                            job_name.clone(),
                            AppState {
                                status: BuildStatus::Failed {
                                    reason: e.to_string(),
                                },
                            },
                        );
                        return;
                    }
                }
            }
            Err(e) => {
                error!(error = %e, "deployment failed, marking job as failed");
                store.insert(
                    job_name.clone(),
                    AppState {
                        status: BuildStatus::Failed {
                            reason: e.to_string(),
                        },
                    },
                );
                return;
            }
        };

    info!(%http_route_url, "httproute created successfully, transitioning to running");
    store.insert(
        job_name.clone(),
        AppState {
            status: BuildStatus::Running {
                url: http_route_url,
            },
        },
    );
}
