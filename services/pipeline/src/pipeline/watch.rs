use crate::BUILD_NAMESPACE;
use futures::StreamExt;
use k8s_openapi::api::batch::v1::Job;
use kube::runtime::watcher::{self, Config};
use kube::{Api, Client};
use tracing::{debug, error, info, warn};

pub(crate) async fn wait_for_job(client: Client, job_name: String) -> bool {
    let api = Api::<Job>::namespaced(client, BUILD_NAMESPACE);
    let cfg = Config::default().fields(&format!("metadata.name={job_name}"));

    info!(%job_name, namespace = BUILD_NAMESPACE, "starting job watcher");
    let mut stream = watcher::watcher(api, cfg).boxed();

    while let Some(event) = stream.next().await {
        match event {
            Ok(watcher::Event::Apply(job)) => {
                let name = job.metadata.name.unwrap_or_else(|| job_name.clone());
                if let Some(status) = &job.status {
                    let succeeded = status.succeeded.unwrap_or(0);
                    let failed = status.failed.unwrap_or(0);

                    debug!(job = %name, succeeded, failed, "received apply event with job status");

                    if succeeded > 0 {
                        info!(job = %name, "job completed successfully");
                        return true;
                    }
                    if failed > 0 {
                        warn!(job = %name, "job failed");
                        return false;
                    }
                } else {
                    debug!(job = %name, "received apply event without status");
                }
            }
            Ok(watcher::Event::Delete(job)) => {
                let name = job.metadata.name.unwrap_or_else(|| job_name.clone());
                warn!(job = %name, "job resource deleted while watching");
            }
            Ok(watcher::Event::Init) => {
                debug!(%job_name, "watcher init event");
            }
            Ok(watcher::Event::InitApply(job)) => {
                let name = job.metadata.name.unwrap_or_else(|| job_name.clone());
                debug!(job = %name, "watcher init-apply event");
            }
            Ok(watcher::Event::InitDone) => {
                debug!(%job_name, "watcher init done");
            }
            Err(e) => {
                error!(%job_name, error = %e, "watcher stream error");
            }
        }
    }

    warn!(%job_name, "watcher stream ended before terminal job state");
    false
}
