FROM rust:1.94 AS builder
WORKDIR /app

COPY Cargo.toml Cargo.lock ./
COPY src ./src

RUN cargo build --release

FROM debian:bookworm-slim
WORKDIR /app

COPY --from=builder /app/target/release/backend /usr/local/bin/backend

EXPOSE 8080

CMD ["backend"]
