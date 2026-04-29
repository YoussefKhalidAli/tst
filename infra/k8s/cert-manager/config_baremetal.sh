#!/bin/bash
# setup-zot-baremetal.sh

REGISTRY_IP="10.96.163.253"
HOSTNAME="zot.registry.svc.cluster.local"

# Check for root
if [[ $EUID -ne 0 ]]; then
   echo "This script must be run as root (sudo)"
   exit 1
fi

echo "--- Updating /etc/hosts ---"
sed -i "/$HOSTNAME/d" /etc/hosts
echo "$REGISTRY_IP $HOSTNAME" >> /etc/hosts

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

echo "--- Fetching and Trusting Certificate ---"
# We assume you have kubectl access from this node or the cert is available
# If running on a worker without kubectl, you'd scp the cert here first.
if command -v kubectl &> /dev/null; then
    kubectl get secret iti-ca-secret -n cert-manager -o jsonpath='{.data.tls\.crt}' | base64 -d > /usr/local/share/ca-certificates/iti.crt
else
    echo "Warn: kubectl not found. Please ensure /usr/local/share/ca-certificates/iti.crt exists."
fi

update-ca-certificates

echo "--- Restarting Containerd ---"
systemctl restart containerd

echo "Done! Bare metal node now trusts $HOSTNAME"
