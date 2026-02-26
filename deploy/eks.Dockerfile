FROM gcr.io/distroless/static-debian13:nonroot
ARG version=dev

COPY --chown=root:root --chmod=755 ./eks ./eks

EXPOSE 3000
ENV VERSION=${version}
ENTRYPOINT ["./eks"]
