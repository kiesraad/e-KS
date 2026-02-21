FROM ubuntu:24.04
RUN apt-get update && apt-get --no-install-recommends install -y adduser && apt-get upgrade -y && apt-get clean

# create a non root user to run the binary
ARG user=nonroot
ARG group=nonroot
ARG uid=2000
ARG gid=2000
RUN addgroup --gid "${gid}" "${group}" && \
    adduser --uid "${uid}" --gid "${gid}" --system --disabled-login --disabled-password "${user}"

WORKDIR /home/${user}
USER $user

ARG version=dev

COPY --chown=root:root --chmod=755 ./eks ./eks

EXPOSE 3000
ENV VERSION=${version}
ENTRYPOINT ["./eks"]
