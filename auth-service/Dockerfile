FROM rust:1.88 as builder
WORKDIR /app

# Enable SQLx offline mode - no database connection needed during build
ENV SQLX_OFFLINE=true

# Copy workspace files
COPY Cargo.toml Cargo.lock ./
COPY .sqlx ./.sqlx
COPY shared ./shared
COPY auth-service ./auth-service
COPY weather-service ./weather-service
COPY time-service ./time-service

# Build the application
WORKDIR /app/auth-service
RUN cargo build --release

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/auth-service /usr/local/bin/auth-service

EXPOSE 8000

CMD ["auth-service"]

