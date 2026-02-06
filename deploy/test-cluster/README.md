# Test cluster setup

The test cluster requires a few cluster wide applications to function, such as the traefik ingress controller, cert-manager
and a postgres database. To following explains the required installation steps.

## Traefik Gateway Controller

```shell
# Apply Gateway API CRDs
kubectl apply -f https://github.com/kubernetes-sigs/gateway-api/releases/download/v1.4.0/standard-install.yaml
# Apply traefik Gateway API CRDs
kubectl apply -f https://raw.githubusercontent.com/traefik/traefik/v3.6/docs/content/reference/dynamic-configuration/kubernetes-gateway-rbac.yml

# Install traefik
helm upgrade --install traefik oci://ghcr.io/traefik/helm/traefik -n ingress --create-namespace -f traefik-values.yaml

kubectl apply -f traefik-middleware.yaml
```

## Cert manager

```shell
helm upgrade --install cert-manager oci://quay.io/jetstack/charts/cert-manager -n cert-manager --create-namespace -f cert-manager-values.yaml
kubectl apply -f cert-issuers.yaml

# Use wildcard certificate for *.kiesraad.net as default certificate
kubectl apply -f ingress-cert.yaml
kubectl apply -f traefik-tlsstore.yaml
```


## Scaleway cert manager webhook (for wildcard certificates)

Create new credentials in the [Scaleway console](https://console.scaleway.com/credentials/credentials) with just DNS write permissions and fill them in below.

```shell
export SCW_ACCESS_KEY=<access-key>
export SCW_SECRET_KEY=<secret-key>

helm repo add scaleway https://helm.scw.cloud/
helm repo update
helm upgrade --install --namespace cert-manager scaleway-certmanager-webhook scaleway/scaleway-certmanager-webhook \
  --set secret.accessKey=$SCW_ACCESS_KEY \
  --set secret.secretKey=$SCW_SECRET_KEY
```

## Oauth2 proxy
```shell
export CLIENT_ID=<client-id>
export CLIENT_SECRET=<client-secret>
export COOKIE_SECRET=<cookie-secret>

helm repo add oauth2-proxy https://oauth2-proxy.github.io/manifests
helm upgrade --install oauth2-proxy oauth2-proxy/oauth2-proxy -n ingress \
 -f oauth2-proxy-values.yaml \
 --set config.clientID=$CLIENT_ID \
 --set config.clientSecret=$CLIENT_SECRET  \
 --set config.cookieSecret=$COOKIE_SECRET
 
kubectl apply -f oauth-ingress.yaml
```

## Postgres

```shell
export PG_PASSWORD=<password>

helm upgrade --install postgresql \
  oci://registry-1.docker.io/bitnamicharts/postgresql \
  --version 18.1.14 -n postgresql --create-namespace -f psql-values.yaml \
  --set auth.postgresPassword=$PG_PASSWORD
```

## Docker pull secret

To avoid rate limits by GitHub, we must authenticate when pulling docker images from the GitHub container registry.
For that, first create a [personal access token](https://github.com/settings/tokens/new) in GitHub with scope
`read:packages`.
Then, you need to place this into the GitHub environment as `IMAGE_PULL_SECRET_USERNAME` variable and
`IMAGE_PULL_SECRET_TOKEN` secret.
