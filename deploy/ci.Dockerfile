FROM ubuntu:24.04 AS final-base
RUN apt-get update && apt-get install adduser -y && apt-get upgrade -y

# create a non root user to run the binary
ARG user=nonroot
ARG group=nonroot
ARG uid=2000
ARG gid=2000
RUN addgroup --gid ${gid} ${group} && adduser --uid ${uid} --gid ${gid} --system --disabled-login --disabled-password ${user}

WORKDIR /home/${user}
USER $user

FROM final-base AS e-ks-core
ARG version=dev

COPY --chown=nonroot:nonroot ./e-ks-core ./e-ks-core
RUN chmod 700 e-ks-core

EXPOSE 3000
ENV VERSION=${version}
ENTRYPOINT ["./e-ks-core"]
