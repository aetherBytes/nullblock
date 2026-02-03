FROM rust:1.88-bookworm AS builder
RUN apt-get update && apt-get install -y cmake libssl-dev pkg-config && rm -rf /var/lib/apt/lists/*
WORKDIR /app
COPY lib/nullblock-mcp-client lib/nullblock-mcp-client
COPY svc/nullblock-agents/Cargo.toml svc/nullblock-agents/Cargo.toml
RUN mkdir -p svc/nullblock-agents/src && echo 'fn main() {}' > svc/nullblock-agents/src/main.rs
RUN cd svc/nullblock-agents && cargo build --release 2>/dev/null || true
COPY svc/nullblock-agents/src svc/nullblock-agents/src
RUN cd svc/nullblock-agents && cargo build --release

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates curl postgresql-client && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/svc/nullblock-agents/target/release/nullblock-agents /usr/local/bin/
COPY svc/nullblock-agents/migrations /app/migrations
COPY svc/nullblock-agents/entrypoint.sh /app/entrypoint.sh
RUN chmod +x /app/entrypoint.sh
WORKDIR /app
EXPOSE 9003
ENTRYPOINT ["/app/entrypoint.sh"]
CMD ["nullblock-agents"]
