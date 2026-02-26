FROM ghcr.io/tweedegolf/typst-webservice:0.5.1

ENV VERSION=dev

USER root
COPY --chown=root:root / ./assets/

RUN find ./assets -type d -exec chmod 755 {} + && \
    find ./assets -type f -exec chmod 644 {} +
USER nonroot
