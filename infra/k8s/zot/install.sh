#!/bin/bash

helm repo add project-zot http://zotregistry.dev/helm-charts
helm repo update project-zot
helm upgrade --install zot project-zot/zot --namespace registry --create-namespace -f ./config.yaml
