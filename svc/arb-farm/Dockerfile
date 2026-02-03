FROM rust:1.88-bookworm AS builder
RUN apt-get update && apt-get install -y cmake libssl-dev pkg-config && rm -rf /var/lib/apt/lists/*
WORKDIR /app
COPY lib/nullblock-mcp-client lib/nullblock-mcp-client
COPY svc/arb-farm/Cargo.toml svc/arb-farm/Cargo.toml
RUN mkdir -p svc/arb-farm/src && echo 'fn main() {}' > svc/arb-farm/src/main.rs
RUN cd svc/arb-farm && cargo build --release 2>/dev/null || true
COPY svc/arb-farm/src svc/arb-farm/src
RUN cd svc/arb-farm && cargo build --release

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates curl postgresql-client && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/svc/arb-farm/target/release/arb-farm /usr/local/bin/
COPY svc/arb-farm/migrations /app/migrations
COPY svc/arb-farm/entrypoint.sh /app/entrypoint.sh
RUN chmod +x /app/entrypoint.sh
WORKDIR /app
EXPOSE 9007
ENTRYPOINT ["/app/entrypoint.sh"]
CMD ["arb-farm"]
