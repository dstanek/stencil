FROM ubuntu:latest
WORKDIR /tests

RUN \
    apt-get update && \
    apt-get install -y python3 curl && \
    rm -rf /var/lib/apt/lists/*

ENV UV_VENV=/venv
ENV PATH="${UV_VENV}/bin:$PATH"

COPY --from=ghcr.io/astral-sh/uv:0.5.27 /uv /uvx /bin/
COPY requirements.txt /tests/
RUN \
    uv venv ${UV_VENV} && \
    uv pip install -r requirements.txt --python ${UV_VENV}/bin/python

CMD ["pytest", "tests.py", "-v"]