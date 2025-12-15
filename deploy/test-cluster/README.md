# Test cluster setup

The test cluster requires a few cluster wide applications to function, such as the traefik ingress controller, cert-manager
and a postgres database. To following explains the required installation steps.

## Traefik ingress controller

```shell
helm upgrade --install traefik oci://ghcr.io/traefik/helm/traefik -n ingress --create-namespace -f traefik-values.yaml
```

## Cert manager
```shell
helm upgrade --install cert-manager oci://quay.io/jetstack/charts/cert-manager -n cert-manager --create-namespace -f cert-manager-values.yaml
kubectl apply -f cert-issuers.yaml
```

## Postgres

```shell
helm upgrade --install postgresql oci://registry-1.docker.io/bitnamicharts/postgresql --version 18.1.14 -n postgresql --create-namespace -f psql-values.yaml
```


## Docker pull secret
To avoid rate limits by GitHub, we must authenticate when pulling docker images from the GitHub container registry.
For that, first create a [personal access token](https://github.com/settings/tokens/new) in GitHub with scope `read:packages`.
Then, you need to place this into the GitHub environment as `IMAGE_PULL_SECRET_USERNAME` variable and `IMAGE_PULL_SECRET_TOKEN` secret.
