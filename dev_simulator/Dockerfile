# Shared image for FunnyX local Rust services.
# Build context = crate directory; pass BIN_NAME (cargo package / binary name).
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
    && rm -rf /var/lib/apt/lists/*
ARG BIN_NAME
COPY --from=builder /app/target/release/${BIN_NAME} /usr/local/bin/service
ENV HTTP_HOST=0.0.0.0
ENV RUST_LOG=info
ENTRYPOINT ["/usr/local/bin/service"]
