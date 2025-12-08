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
- SQLx CLI (for migrations): `cargo install sqlx-cli`

### Running with Docker Compose

1. Clone the repository:
```bash
git clone <repository-url>
cd karl-systems-challenge
```

2. Set up environment variables:
```bash
cp .env.example .env
# Edit .env and set JWT_SECRET to a secure value
```

3. Start all services:
```bash
docker compose up -d
```

4. Run database migrations:
```bash
docker compose exec auth-service sqlx migrate run
```

5. Services will be available at:
   - Auth Service: http://localhost:8000
   - Weather Service: http://localhost:8001
   - Time Service: http://localhost:8002

### Running Locally (Development)

1. Start PostgreSQL:
```bash
docker compose up postgres -d
```

2. Set up environment variables (create `.env` file):
```env
DATABASE_URL=postgres://postgres:postgres@localhost:5432/auth_db
JWT_SECRET=your-secret-key-here
AUTH_SERVICE_URL=http://localhost:8000
PORT=8000
```

3. Run migrations:
```bash
cd auth-service
sqlx migrate run
cd ..
```

4. Start services in separate terminals:
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

## Usage Examples

### Register a User

```bash
curl -X POST http://localhost:8000/auth/register \
  -H "Content-Type: application/json" \
  -d '{
    "username": "testuser",
    "email": "test@example.com",
    "password": "password123"
  }'
```

### Login

```bash
curl -X POST http://localhost:8000/auth/login \
  -H "Content-Type: application/json" \
  -d '{
    "username": "testuser",
    "password": "password123"
  }'
```

### Get Weather (requires JWT token)

```bash
curl http://localhost:8001/weather/London \
  -H "Authorization: Bearer <your-token>"
```

### Get Time (requires JWT token)

```bash
curl http://localhost:8002/time/London \
  -H "Authorization: Bearer <your-token>"
```

## Documentation

- [API Contracts](./API_CONTRACTS.md) - Detailed API documentation
- [Architecture](./ARCHITECTURE.md) - System architecture and flowcharts

## License

MIT

