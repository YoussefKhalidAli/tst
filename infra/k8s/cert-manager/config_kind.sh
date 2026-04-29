#!/bin/bash
# setup-zot-kind.sh

NODE_NAME="local-control-plane"
REGISTRY_IP="10.96.163.253"
HOSTNAME="zot.registry.svc.cluster.local"


# Create certificates
kubectl apply -f https://github.com/cert-manager/cert-manager/releases/latest/download/cert-manager.yaml # Install cert-manager

kubectl wait --for=condition=Available deployment/cert-manager -n cert-manager --timeout=60s

kubectl apply -f ./issuer.yml
kubectl apply -f ./ca-cert.yml
kubectl apply -f ./iti-issuer.yaml
kubectl apply -f ./apps-cert.yml

kubectl create namespace registry
kubectl apply -f ./zot-cert.yml

kubectl wait --for=condition=Ready certificate/zot-tls -n registry --timeout=60s # I think this works

echo "--- Extracting Root CA from K8s ---"
kubectl get secret iti-ca-secret -n cert-manager -o jsonpath='{.data.tls\.crt}' | base64 -d > iti-root.crt

echo "--- Updating /etc/hosts inside Kind node ---"
# Remove old entry if exists and add new one
docker exec $NODE_NAME sh -c "sed -i '/$HOSTNAME/d' /etc/hosts && echo '$REGISTRY_IP $HOSTNAME' >> /etc/hosts"

echo "--- Injecting CA and updating trust store ---"
docker cp iti-root.crt $NODE_NAME:/usr/local/share/ca-certificates/iti.crt
docker exec $NODE_NAME update-ca-certificates

echo "--- Restarting Containerd ---"
docker exec $NODE_NAME systemctl restart containerd

echo "Done! Kind node now trusts $HOSTNAME"
