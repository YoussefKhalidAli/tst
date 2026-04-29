use crate::REGISTRY_URL;
use anyhow::Ok;
use k8s_openapi::api::batch::v1::Job;
use kube::{Api, Client};
use serde_json::json;
use uuid::Uuid;

use crate::BUILD_NAMESPACE;

pub async fn create_build_job(
    client: &Client,
    registry: &str,
    app_name: &str,
    repo_url: &str,
    git_ref: &str,
) -> anyhow::Result<String> {
    let job_name = format!("{}-{}", app_name, &Uuid::new_v4().to_string()[..8]);
    let latest_dest = format!("{registry}/{app_name}:latest");
    let ref_dest = format!("{registry}/{app_name}:{git_ref}");
    let git_url = repo_url.replace("https://", "git://");

    let job: Job = serde_json::from_value(json!({
    "apiVersion": "batch/v1",
    "kind": "Job",
    "metadata": {
        "name": job_name,
        "namespace": BUILD_NAMESPACE
    },
    "spec": {
        "ttlSecondsAfterFinished": 120,
        "template": {
            "spec": {
                "restartPolicy": "Never",
                "containers": [{
                    "name": "kaniko",
                    "image": "martizih/kaniko:latest",
                    "args": [
                        format!("--context={}", git_url),
                        format!("--destination={}", ref_dest),
                        format!("--destination={}", latest_dest),
                        "--insecure",
                        "--skip-tls-verify",
                        "--insecure-pull",
                        "--cache=true",
                        format!("--cache-repo={REGISTRY_URL}/cache"),
                        format!("--registry-mirror={REGISTRY_URL}")
                    ],
                    "volumeMounts": [{
                        "name": "kaniko-secret",
                        "mountPath": "/kaniko/.docker"
                    }]
                }],
                "volumes": [{
                    "name": "kaniko-secret",
                    "secret": {
                        "secretName": "registry-creds",
                        "items": [{"key": "config.json", "path": "config.json"}]
                    }
                }]
            }
        }
    }}))?;

    let jobs: Api<Job> = Api::namespaced(client.clone(), BUILD_NAMESPACE);
    jobs.create(&Default::default(), &job).await?;

    Ok(job_name)
}
