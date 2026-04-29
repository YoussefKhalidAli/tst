use k8s_openapi::api::{apps::v1::Deployment, core::v1::Service};
use kube::{Api, Client};
use serde_json::json;

pub async fn create_deployment(
    client: &Client,
    app_name: &str,
    port: u16,
    image: &str,
    namespace: &str,
) -> anyhow::Result<String> {
    let deployment: Deployment = serde_json::from_value(json!({
        "apiVersion": "apps/v1",
        "kind": "Deployment",
        "metadata": {
            "name": app_name,
            "namespace": namespace
        },
        "spec": {
            "replicas": 1,
            "selector": {
                "matchLabels": { "app": app_name }
            },
            "template": {
                "metadata": {
                    "labels": { "app": app_name }
                },
                "spec": {
                    "containers": [{
                        "name": app_name,
                        "image": image,
                        "ports": [{ "containerPort": port }],
                        // pull from local registry, no auth needed
                        "imagePullPolicy": "Always",
                        "env": [{"name": "PORT", "value": port.to_string()}]
                    }]
                }
            }
        }
    }))?;

    let deployments: Api<Deployment> = Api::namespaced(client.clone(), namespace);

    // use patch to handle both create and update (idempotent)
    deployments
        .patch(
            app_name,
            &kube::api::PatchParams::apply("pipeline-service"),
            &kube::api::Patch::Apply(deployment),
        )
        .await?;

    let service: Service = serde_json::from_value(json!({
        "apiVersion": "v1",
        "kind": "Service",
        "metadata": {
            "name": app_name,
            "namespace": namespace
        },
        "spec": {
            "selector": { "app": app_name },
            "ports": [{ "port": 80, "targetPort": port }]
        }
    }))?;

    let services: Api<Service> = Api::namespaced(client.clone(), namespace);
    services
        .patch(
            app_name,
            &kube::api::PatchParams::apply("pipeline-service"),
            &kube::api::Patch::Apply(service),
        )
        .await?;

    // TODO: Placeholder until we get real Gateway API
    Ok(format!("http://{}.{}.iti.local", app_name, namespace))
}
