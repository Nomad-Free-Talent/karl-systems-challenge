FROM rust:1.75 as builder
WORKDIR /app

# Copy workspace files
COPY Cargo.toml Cargo.lock ./
COPY shared ./shared
COPY time-service ./time-service

# Build the application
WORKDIR /app/time-service
RUN cargo build --release

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/time-service /usr/local/bin/time-service

EXPOSE 8002

CMD ["time-service"]

