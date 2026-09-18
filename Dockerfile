# Estágio 1: Compilação do módulo nativo e empacotamento do wheel
FROM python:3.13-slim AS builder

RUN apt-get update && apt-get install -y --no-install-recommends \
    build-essential \
    curl \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain stable --profile minimal
ENV PATH="/root/.cargo/bin:${PATH}"

COPY --from=ghcr.io/astral-sh/uv:latest /uv /usr/local/bin/uv

WORKDIR /build

COPY Cargo.toml Cargo.lock pyproject.toml README.md LICENSE ./
COPY engine/ engine/
COPY orchestrator/ orchestrator/

RUN uv run --with maturin maturin build --release --out dist

# Estágio 2: Runtime enxuto e seguro (não-root)
FROM python:3.13-slim AS runtime

RUN groupadd -g 10001 appgroup && \
    useradd -u 10001 -g appgroup -s /bin/sh -m appuser

WORKDIR /app

COPY --from=ghcr.io/astral-sh/uv:latest /uv /usr/local/bin/uv

COPY --from=builder /build/dist/*.whl /tmp/
RUN uv pip install --system --no-cache /tmp/*.whl && rm -rf /tmp/*.whl

RUN mkdir -p /app/data && chown -R appuser:appgroup /app

USER appuser

VOLUME ["/app/data"]

ENTRYPOINT ["python", "-c", "import cnpydge; print('CNPydge Container Runtime - Motor:', cnpydge.version())"]

