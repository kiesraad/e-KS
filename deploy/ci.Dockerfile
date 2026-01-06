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

FROM final-base AS eks_core
ARG version=dev

COPY --chown=nonroot:nonroot ./eks_core ./eks_core
RUN chmod 700 eks_core

EXPOSE 3000
ENV VERSION=${version}
ENTRYPOINT ["./eks_core"]

FROM final-base AS migrate_db
ARG version=dev

COPY --chown=nonroot:nonroot ./migrate_db ./migrate_db
RUN chmod 700 migrate_db

EXPOSE 3000
ENV VERSION=${version}
ENTRYPOINT ["./migrate_db"]
