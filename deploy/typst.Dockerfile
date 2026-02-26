FROM alpine:latest AS set-file-permissions
COPY . ./assets/

RUN find ./assets -type d -exec chmod 755 {} + && \
    find ./assets -type f -exec chmod 644 {} +

FROM ghcr.io/tweedegolf/typst-webservice:0.5.1

ENV VERSION=dev

COPY --from=set-file-permissions --chown=root:root ./assets/ ./assets/

