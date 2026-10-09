# FunnyX Core Engine — same layout as dev_simulator/Dockerfile
ARG RUST_IMAGE=rust:1.88-bookworm
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
EXPOSE 18200
ENTRYPOINT ["/usr/local/bin/service"]
