# syntax=docker/dockerfile:1
FROM rust:1.85-slim AS builder
WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY src ./src
RUN cargo build --release

FROM debian:bookworm-slim
WORKDIR /app
COPY --from=builder /app/target/release/king-pirate /usr/local/bin/king-pirate
EXPOSE 843 8002 8003 8081
CMD ["king-pirate"]
