# Install and setup NGINX Gateway Fabric
kubectl kustomize "https://github.com/nginx/nginx-gateway-fabric/config/crd/gateway-api/standard?ref=v2.5.1" | kubectl apply -f -
helm install ngf oci://ghcr.io/nginx/charts/nginx-gateway-fabric --create-namespace -n nginx-gateway --version 0.0.0-edge --wait --set nginx.service.type=NodePort

# Create gateway
kubectl apply -f ./gateway.yml