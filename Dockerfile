# --- Builder stage ---
FROM rust:1.89.0 as builder

WORKDIR /app

COPY . .

RUN cargo build --release

# --- Runtime stage ---
FROM debian:latest

RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates \
    && apt-get clean \
    && rm -rf /var/lib/apt/lists/*

RUN update-ca-certificates

WORKDIR /app

COPY --from=builder /app/target/release/mcim-rust-api ./mcim-rust-api

EXPOSE 8080

CMD ["./mcim-rust-api"]