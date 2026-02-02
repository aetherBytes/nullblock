FROM rust:1.88-bookworm AS builder
RUN apt-get update && apt-get install -y cmake libssl-dev pkg-config && rm -rf /var/lib/apt/lists/*
WORKDIR /app
COPY svc/nullblock-engrams/Cargo.toml svc/nullblock-engrams/Cargo.toml
RUN mkdir -p svc/nullblock-engrams/src && echo 'fn main() {}' > svc/nullblock-engrams/src/main.rs
RUN cd svc/nullblock-engrams && cargo build --release 2>/dev/null || true
COPY svc/nullblock-engrams/src svc/nullblock-engrams/src
RUN cd svc/nullblock-engrams && cargo build --release

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates curl postgresql-client && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/svc/nullblock-engrams/target/release/nullblock-engrams /usr/local/bin/
COPY svc/nullblock-engrams/migrations /app/migrations
COPY svc/nullblock-engrams/entrypoint.sh /app/entrypoint.sh
RUN chmod +x /app/entrypoint.sh
WORKDIR /app
EXPOSE 9004
ENTRYPOINT ["/app/entrypoint.sh"]
CMD ["nullblock-engrams"]
