FROM python:3.12-slim

ARG UID=1000
ARG GID=1000

ENV PYTHONDONTWRITEBYTECODE=1 \
    PIP_DISABLE_PIP_VERSION_CHECK=1

WORKDIR /work/

RUN addgroup --system --gid "${GID}" djlint \
    && adduser --system --uid "${UID}" --ingroup djlint --home /home/djlint djlint \
    && chown -R djlint:djlint /work \
    && pip install --no-cache-dir djlint

USER djlint

CMD ["djlint", "--ignore=H025", "--reformat", "--lint", "--indent=2", "templates"]
