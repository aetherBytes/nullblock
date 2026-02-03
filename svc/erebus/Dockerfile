FROM rust:1.88-bookworm AS builder
RUN apt-get update && apt-get install -y cmake libssl-dev pkg-config && rm -rf /var/lib/apt/lists/*
WORKDIR /app
COPY lib/nullblock-mcp-client lib/nullblock-mcp-client
COPY svc/erebus/Cargo.toml svc/erebus/Cargo.toml
COPY svc/erebus/Cargo.lock svc/erebus/Cargo.lock
RUN mkdir -p svc/erebus/src && echo 'fn main() {}' > svc/erebus/src/main.rs
RUN cd svc/erebus && cargo build --release 2>/dev/null || true
COPY svc/erebus/src svc/erebus/src
RUN cd svc/erebus && cargo build --release --bin erebus --bin seed-agent-keys

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates curl postgresql-client && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/svc/erebus/target/release/erebus /usr/local/bin/
COPY --from=builder /app/svc/erebus/target/release/seed-agent-keys /usr/local/bin/
COPY svc/erebus/migrations /app/migrations
COPY svc/erebus/entrypoint.sh /app/entrypoint.sh
RUN chmod +x /app/entrypoint.sh
WORKDIR /app
EXPOSE 3000
ENTRYPOINT ["/app/entrypoint.sh"]
CMD ["erebus"]
