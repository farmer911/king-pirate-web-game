# syntax=docker/dockerfile:1
# King Pirate Rust server: portal and client files (:8081), game sockets (:8003 WebSocket, :8002 TCP).
FROM rust:1.95-slim AS builder
WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY src ./src
RUN cargo build --release --locked

FROM debian:bookworm-slim
COPY --from=builder /app/target/release/king-pirate /usr/local/bin/king-pirate
WORKDIR /app
EXPOSE 8002 8003 8081
CMD ["king-pirate"]
