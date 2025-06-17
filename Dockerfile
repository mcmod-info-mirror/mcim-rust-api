FROM rust:1.87.0 AS builder

WORKDIR /app

COPY Cargo.toml ./Cargo.toml

COPY src ./src

RUN cargo build --release

FROM ubuntu:latest

RUN apt-get update \
 && apt-get install -y --no-install-recommends ca-certificates \
 && apt-get clean \
 && rm -rf /var/lib/apt/lists/*

RUN update-ca-certificates

WORKDIR /app

COPY --from=builder /app/target/release/mcim-rust-api ./mcim-rust-api

EXPOSE 8080

CMD ["./mcim-rust-api"]