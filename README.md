# Karl Systems Challenge

A microservices architecture implementation with three separate services that communicate to achieve a common goal.

## Architecture Overview

This project consists of three microservices:

- **Service A: Authentication Service** - JWT-based authentication with RBAC (Role-Based Access Control)
- **Service B: Weather Service** - Aggregates weather data from multiple free weather APIs
- **Service C: Time Service** - Returns current time for cities with second accuracy

## Technology Stack

- **Language**: Rust
- **Web Framework**: Actix-web
- **Async Runtime**: Tokio
- **Database**: PostgreSQL with SQLx
- **Authentication**: JWT (JSON Web Tokens)
- **Caching**: In-memory caching with debouncing for rate limiting

## Project Structure

```
karl-systems-challenge/
├── shared/              # Shared types and error definitions
├── auth-service/        # Authentication and user management service
├── weather-service/     # Weather data aggregation service
└── time-service/        # Time service for cities
```

## Quick Start

### Prerequisites

- Rust (latest stable)
- Docker and Docker Compose
- PostgreSQL (or use Docker Compose)

### Running Locally

1. Start PostgreSQL:
```bash
docker compose up postgres -d
```

2. Set up environment variables (create `.env` file):
```env
DATABASE_URL=postgres://postgres:postgres@localhost:5432/auth_db
JWT_SECRET=your-secret-key-here
```

3. Run migrations (when auth-service is ready):
```bash
cd auth-service
sqlx migrate run
```

4. Start services (when implemented):
```bash
# Terminal 1 - Auth Service
cargo run -p auth-service

# Terminal 2 - Weather Service
cargo run -p weather-service

# Terminal 3 - Time Service
cargo run -p time-service
```

## Development

### Building

```bash
cargo build --workspace
```

### Testing

```bash
cargo test --workspace
```

### Running Tests

```bash
# All tests
cargo test --workspace

# Specific service
cargo test -p auth-service
```

## Documentation

- [API Contracts](./API_CONTRACTS.md) - Detailed API documentation (to be added)
- [Architecture](./ARCHITECTURE.md) - System architecture and flowcharts (to be added)

## License

MIT

