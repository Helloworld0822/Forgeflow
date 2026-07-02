# syntax=docker/dockerfile:1

FROM docker.io/library/node:22-bookworm AS frontend
WORKDIR /build/frontend
COPY frontend/package.json frontend/package-lock.json* ./
RUN npm ci
COPY frontend/ ./
RUN npm run build

FROM docker.io/library/rust:1-bookworm AS builder
WORKDIR /build
COPY Cargo.toml ./
COPY Cargo.lock* ./
COPY src ./src
COPY --from=frontend /build/static ./static
RUN cargo build --release

FROM docker.io/library/debian:bookworm-slim AS runtime
RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app
COPY --from=builder /build/target/release/autoforge /usr/local/bin/autoforge
COPY --from=frontend /build/static ./static

ENV HOST=0.0.0.0
ENV PORT=8080
ENV RUST_LOG=info

EXPOSE 8080
ENTRYPOINT ["/usr/local/bin/autoforge"]
CMD ["serve"]
