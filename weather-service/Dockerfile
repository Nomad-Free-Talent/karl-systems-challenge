FROM rust:1.88 as builder
WORKDIR /app

ENV SQLX_OFFLINE=true

# Copy workspace files
COPY Cargo.toml Cargo.lock ./
COPY .sqlx ./.sqlx
COPY shared ./shared
COPY auth-service ./auth-service
COPY weather-service ./weather-service
COPY time-service ./time-service

# Build the application
WORKDIR /app/weather-service
RUN cargo build --release

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/weather-service /usr/local/bin/weather-service

EXPOSE 8001

CMD ["weather-service"]

