ARG RUST_IMAGE=rust:1.88-bookworm
ARG RUNTIME_IMAGE=debian:bookworm-slim

FROM ${RUST_IMAGE} AS builder
WORKDIR /app
# Build context should be repo root so common/ path deps resolve.
COPY common /app/common
COPY backend/session_token_server /app/backend/session_token_server
WORKDIR /app/backend/session_token_server
RUN cargo build --release

FROM ${RUNTIME_IMAGE}
RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates curl \
    && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/backend/session_token_server/target/release/funnyx-session-token-server /usr/local/bin/service
ENV HTTP_HOST=0.0.0.0
ENV HTTP_PORT=8082
ENV SESSION_STORE=memory
ENV RUST_LOG=info
EXPOSE 8082
ENTRYPOINT ["/usr/local/bin/service"]
