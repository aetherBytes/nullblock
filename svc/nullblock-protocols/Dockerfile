FROM rust:1.88-bookworm AS builder
RUN apt-get update && apt-get install -y cmake libssl-dev pkg-config && rm -rf /var/lib/apt/lists/*
WORKDIR /app
COPY lib/nullblock-mcp-client lib/nullblock-mcp-client
COPY svc/nullblock-protocols/Cargo.toml svc/nullblock-protocols/Cargo.toml
RUN mkdir -p svc/nullblock-protocols/src && echo 'fn main() {}' > svc/nullblock-protocols/src/main.rs
RUN cd svc/nullblock-protocols && cargo build --release 2>/dev/null || true
COPY svc/nullblock-protocols/src svc/nullblock-protocols/src
RUN touch svc/nullblock-protocols/src/main.rs && cd svc/nullblock-protocols && cargo build --release

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates curl && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/svc/nullblock-protocols/target/release/nullblock-protocols /usr/local/bin/
WORKDIR /app
EXPOSE 8001
CMD ["nullblock-protocols"]
