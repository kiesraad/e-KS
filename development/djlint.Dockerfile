FROM python:3.12-slim

ENV PYTHONDONTWRITEBYTECODE=1 \
    PIP_DISABLE_PIP_VERSION_CHECK=1

WORKDIR /work/

RUN pip install --no-cache-dir djlint

CMD ["djlint", "--ignore=H025", "--reformat", "templates"]
