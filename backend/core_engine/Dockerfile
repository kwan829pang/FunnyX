# FunnyX Core Engine — CPU-only matching (no GPU / CUDA).
# Intended for cloud CPU instances and Docker. Do not attach nvidia devices.
ARG RUST_IMAGE=rust:1.90-bookworm
ARG RUNTIME_IMAGE=debian:bookworm-slim

FROM ${RUST_IMAGE} AS builder
WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY src ./src
RUN cargo build --release

FROM ${RUNTIME_IMAGE}
RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates curl \
    && rm -rf /var/lib/apt/lists/* \
    && mkdir -p /data
ARG BIN_NAME=funnyx-core-engine
COPY --from=builder /app/target/release/${BIN_NAME} /usr/local/bin/service
ENV HTTP_HOST=0.0.0.0
ENV RUST_LOG=info
ENV DATA_DIR=/data
# CPU-only policy for cloud/Docker (no GPU reservation in compose either).
ENV CPU_ONLY=1
ENV TOKIO_WORKER_THREADS=2
EXPOSE 18200
ENTRYPOINT ["/usr/local/bin/service"]
