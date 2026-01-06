FROM ubuntu:24.04

ENV VERSION=dev
ENV POSTGRESQL_VERSION=18
RUN apt-get update \
    && DEBIAN_FRONTEND=noninteractive apt-get upgrade -y \
    && DEBIAN_FRONTEND=noninteractive apt-get install --no-install-recommends -y \
    curl \
    ca-certificates \
    openssl \
    bzip2 \
    adduser

RUN install -d /usr/share/postgresql-common/pgdg \
    && curl -o /usr/share/postgresql-common/pgdg/apt.postgresql.org.asc --fail https://www.postgresql.org/media/keys/ACCC4CF8.asc \
    && echo "deb [signed-by=/usr/share/postgresql-common/pgdg/apt.postgresql.org.asc] http://apt.postgresql.org/pub/repos/apt/ noble-pgdg main" > /etc/apt/sources.list.d/pgdg.list \
    && apt-get update \
    && DEBIAN_FRONTEND=noninteractive apt-get install -y --no-install-recommends \
        postgresql-client-$POSTGRESQL_VERSION \
    && rm -rf /var/lib/apt/lists/*

# create a non root user to run the binary
ARG user=nonroot
ARG group=nonroot
ARG uid=2000
ARG gid=2000
RUN addgroup --gid ${gid} ${group} && adduser --uid ${uid} --gid ${gid} --system --disabled-login --disabled-password ${user}

WORKDIR /home/${user}
USER $user

# during runtime, you must set working environment variables
ENV PGHOST=127.0.0.1
ENV PGPORT=5432
ENV PGUSER=postgres
ENV PGPASSWORD="supersecure"
