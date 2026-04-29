# Pipeline Service

`pipeline` is a Rust service that builds and deploys applications into Kubernetes.

It exposes an HTTP API that:

- creates a Kaniko build `Job` from a Git repository,
- watches that job until it succeeds/fails,
- deploys the built image to the apps namespace as a `Deployment` + `Service`,
- tracks per-job status in memory.

## What this service does

High-level flow:

1. Client calls `POST /deploy` with repo + ref + app name.
2. Service creates a Kubernetes `Job` in the build namespace (`build-pipeline`) using Kaniko.
3. Service watches the job status.
4. On success, service applies a `Deployment` and `Service` in the apps namespace (`apps`).
5. Client polls `GET /status/{job_name}` for build/deploy state.

Current status values:

- `building`
- `deploying`
- `running` (includes a URL)
- `failed` (includes a reason)

## API

### `POST /deploy`

Request body:

```json
{
  "git_url": "https://github.com/example/my-app.git",
  "git_ref": "main",
  "app_name": "my-app",
  "app_port": 3000
}
```

Response:

```json
{
  "job_name": "my-app-a1b2c3d4",
  "status": "building"
}
```

### `GET /status/{job_name}`

Example:

```json
{
  "running": {
    "url": "http://my-app.apps.iti.local"
  }
}
```

If the job is unknown, returns `null`.

## Runtime configuration in code

Defined in `src/main.rs`:

- `PORT = 6969`
- `REGISTRY_URL = zot.registry.svc.cluster.local:5000`
- `BUILD_NAMESPACE = build-pipeline`
- `APPS_NAMESPACE = apps`

## Kubernetes dependencies and assumptions

This service assumes the cluster already has:

- namespace `build-pipeline`
- namespace `apps`
- secret `registry-creds` in `build-pipeline` with key `config.json` (mounted to `/kaniko/.docker/config.json`)
- reachable registry at `zot.registry.svc.cluster.local:5000`
- Kaniko image `martizih/kaniko:latest` available to the cluster

Build job behavior:

- job name format: `<app_name>-<8-char-uuid>`
- image tags pushed:
  - `<registry>/<app_name>:<git_ref>`
  - `<registry>/<app_name>:latest`
- Kaniko context is derived by replacing `https://` with `git://` in `git_url`
- build jobs use `ttlSecondsAfterFinished: 120`

Deploy behavior:

- creates/updates `Deployment` and `Service` named `<app_name>` in `apps`
- container port: `<app_port>`
- service port: `80 -> <app_port>`
- image pull policy: `Always`
- returned app URL is currently a placeholder format:
  - `http://<app_name>.<namespace>.iti.local`

## Running locally

Prerequisites:

- Rust toolchain (edition 2024 project)
- access to a Kubernetes cluster (`kube::Client::try_default()` must resolve config)
- permissions to create/watch Jobs in `build-pipeline` and apply Deployments/Services in `apps`

Start service:
cargo build
cargo run

Service listens on `0.0.0.0:6969`.

## Notes for teammates

- Status is stored in-memory (`DashMap`), so statuses are lost on restart.
- URL generation is temporary until real ingress/gateway integration is implemented.
- Deployments are server-side applied with field manager `pipeline-service` (idempotent updates).

## Source map

- `src/main.rs` — service entrypoint and constants
- `src/router.rs` — API routes, request/response types, status store
- `src/pipeline/build.rs` — Kaniko job creation
- `src/pipeline/watch.rs` — build job watcher
- `src/pipeline/deploy.rs` — deployment/service apply logic
- `src/pipeline/mod.rs` — orchestration from build to deploy
