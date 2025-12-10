# API Contracts

This document describes the complete API contracts for all microservices in the Karl Systems Challenge.

## Table of Contents

- [Authentication Service](#authentication-service-port-8000)
- [Weather Service](#weather-service-port-8001)
- [Time Service](#time-service-port-8002)
- [Error Responses](#error-responses)
- [Authentication](#authentication)

---

## Authentication Service (Port 8000)

Base URL: `http://localhost:8000`

### Public Endpoints

#### GET /health
Health check endpoint (no authentication required).

**Response:** `200 OK`
```
OK
```

#### POST /auth/register
Register a new user account. New users are automatically assigned the "user" role with `weather:read` and `time:read` permissions.

**Request:**
```json
{
  "username": "string",
  "email": "string",
  "password": "string"
}
```

**Request Validation:**
- `username`: Required, non-empty string
- `email`: Required, non-empty string, must be unique
- `password`: Required, minimum 8 characters

**Response:** `201 Created`
```json
{
  "data": {
    "user_id": "550e8400-e29b-41d4-a716-446655440000",
    "username": "johndoe",
    "email": "john@example.com"
  }
}
```

**Error Responses:**
- `400 Bad Request`: Missing required fields or password too short
- `409 Conflict`: Username or email already exists

**Example:**
```bash
curl -X POST http://localhost:8000/auth/register \
  -H "Content-Type: application/json" \
  -d '{
    "username": "johndoe",
    "email": "john@example.com",
    "password": "securepass123"
  }'
```

#### POST /auth/login
Authenticate and receive a JWT token. Token expires after 24 hours.

**Request:**
```json
{
  "username": "string",
  "password": "string"
}
```

**Response:** `200 OK`
```json
{
  "data": {
    "token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...",
    "user_id": "550e8400-e29b-41d4-a716-446655440000",
    "username": "johndoe",
    "roles": ["user"]
  }
}
```

**Error Responses:**
- `401 Unauthorized`: Invalid username or password
- `403 Forbidden`: User account is inactive

**Example:**
```bash
curl -X POST http://localhost:8000/auth/login \
  -H "Content-Type: application/json" \
  -d '{
    "username": "johndoe",
    "password": "securepass123"
  }'
```

### Admin Endpoints (Require Admin Role)

All admin endpoints require:
1. Valid JWT token in Authorization header
2. User must have "admin" role

#### GET /admin/users
List all users in the system.

**Headers:** `Authorization: Bearer <token>`

**Response:** `200 OK`
```json
{
  "data": [
    {
      "id": "550e8400-e29b-41d4-a716-446655440000",
      "username": "johndoe",
      "email": "john@example.com",
      "is_active": true,
      "created_at": "2024-01-15T10:30:45.123456Z"
    }
  ]
}
```

**Error Responses:**
- `401 Unauthorized`: Missing or invalid token
- `403 Forbidden`: User does not have admin role

**Example:**
```bash
curl http://localhost:8000/admin/users \
  -H "Authorization: Bearer <token>"
```

#### POST /admin/users
Create a new user (admin only).

**Headers:** `Authorization: Bearer <token>`

**Request:**
```json
{
  "username": "string",
  "email": "string",
  "password": "string"
}
```

**Response:** `201 Created`
```json
{
  "data": {
    "id": "550e8400-e29b-41d4-a716-446655440000",
    "username": "newuser",
    "email": "newuser@example.com",
    "is_active": true,
    "created_at": "2024-01-15T10:30:45.123456Z"
  }
}
```

**Error Responses:**
- `400 Bad Request`: Missing required fields or password too short
- `401 Unauthorized`: Missing or invalid token
- `403 Forbidden`: User does not have admin role
- `409 Conflict`: Username or email already exists

#### GET /admin/users/{id}
Get user details by ID.

**Headers:** `Authorization: Bearer <token>`

**Path Parameters:**
- `id`: UUID of the user

**Response:** `200 OK`
```json
{
  "data": {
    "id": "550e8400-e29b-41d4-a716-446655440000",
    "username": "johndoe",
    "email": "john@example.com",
    "is_active": true,
    "created_at": "2024-01-15T10:30:45.123456Z"
  }
}
```

**Error Responses:**
- `401 Unauthorized`: Missing or invalid token
- `403 Forbidden`: User does not have admin role
- `404 Not Found`: User not found

**Example:**
```bash
curl http://localhost:8000/admin/users/550e8400-e29b-41d4-a716-446655440000 \
  -H "Authorization: Bearer <token>"
```

#### PUT /admin/users/{id}
Update user information.

**Headers:** `Authorization: Bearer <token>`

**Path Parameters:**
- `id`: UUID of the user

**Request:**
```json
{
  "username": "string (optional)",
  "email": "string (optional)",
  "password": "string (optional)",
  "is_active": true
}
```

**Response:** `200 OK`
```json
{
  "data": {
    "id": "550e8400-e29b-41d4-a716-446655440000",
    "username": "updatedusername",
    "email": "updated@example.com",
    "is_active": false,
    "created_at": "2024-01-15T10:30:45.123456Z"
  }
}
```

**Error Responses:**
- `401 Unauthorized`: Missing or invalid token
- `403 Forbidden`: User does not have admin role
- `404 Not Found`: User not found

**Example:**
```bash
curl -X PUT http://localhost:8000/admin/users/550e8400-e29b-41d4-a716-446655440000 \
  -H "Authorization: Bearer <token>" \
  -H "Content-Type: application/json" \
  -d '{
    "is_active": false
  }'
```

#### DELETE /admin/users/{id}
Delete a user from the system.

**Headers:** `Authorization: Bearer <token>`

**Path Parameters:**
- `id`: UUID of the user

**Response:** `204 No Content`

**Error Responses:**
- `401 Unauthorized`: Missing or invalid token
- `403 Forbidden`: User does not have admin role
- `404 Not Found`: User not found

**Example:**
```bash
curl -X DELETE http://localhost:8000/admin/users/550e8400-e29b-41d4-a716-446655440000 \
  -H "Authorization: Bearer <token>"
```

#### GET /admin/roles
List all roles in the system.

**Headers:** `Authorization: Bearer <token>`

**Response:** `200 OK`
```json
{
  "data": [
    {
      "id": "660e8400-e29b-41d4-a716-446655440000",
      "name": "admin",
      "description": "Administrator role",
      "created_at": "2024-01-15T10:30:45.123456Z"
    },
    {
      "id": "770e8400-e29b-41d4-a716-446655440000",
      "name": "user",
      "description": "Standard user role",
      "created_at": "2024-01-15T10:30:45.123456Z"
    }
  ]
}
```

**Error Responses:**
- `401 Unauthorized`: Missing or invalid token
- `403 Forbidden`: User does not have admin role

#### POST /admin/roles
Create a new role.

**Headers:** `Authorization: Bearer <token>`

**Request:**
```json
{
  "name": "string",
  "description": "string (optional)"
}
```

**Response:** `201 Created`
```json
{
  "data": {
    "id": "880e8400-e29b-41d4-a716-446655440000",
    "name": "moderator",
    "description": "Moderator role",
    "created_at": "2024-01-15T10:30:45.123456Z"
  }
}
```

**Error Responses:**
- `400 Bad Request`: Role name is required
- `401 Unauthorized`: Missing or invalid token
- `403 Forbidden`: User does not have admin role
- `409 Conflict`: Role already exists

#### GET /admin/permissions
List all permissions in the system.

**Headers:** `Authorization: Bearer <token>`

**Response:** `200 OK`
```json
{
  "data": [
    {
      "id": "990e8400-e29b-41d4-a716-446655440000",
      "name": "user:read",
      "resource": "user",
      "action": "read",
      "created_at": "2024-01-15T10:30:45.123456Z"
    },
    {
      "id": "aa0e8400-e29b-41d4-a716-446655440000",
      "name": "weather:read",
      "resource": "weather",
      "action": "read",
      "created_at": "2024-01-15T10:30:45.123456Z"
    }
  ]
}
```

**Error Responses:**
- `401 Unauthorized`: Missing or invalid token
- `403 Forbidden`: User does not have admin role

#### POST /admin/permissions
Create a new permission.

**Headers:** `Authorization: Bearer <token>`

**Request:**
```json
{
  "name": "string",
  "resource": "string",
  "action": "string"
}
```

**Response:** `201 Created`
```json
{
  "data": {
    "id": "bb0e8400-e29b-41d4-a716-446655440000",
    "name": "weather:write",
    "resource": "weather",
    "action": "write",
    "created_at": "2024-01-15T10:30:45.123456Z"
  }
}
```

**Error Responses:**
- `400 Bad Request`: Permission name, resource, and action are required
- `401 Unauthorized`: Missing or invalid token
- `403 Forbidden`: User does not have admin role
- `409 Conflict`: Permission already exists

#### POST /admin/users/{user_id}/roles
Assign a role to a user.

**Headers:** `Authorization: Bearer <token>`

**Path Parameters:**
- `user_id`: UUID of the user

**Request:**
```json
{
  "role_id": "660e8400-e29b-41d4-a716-446655440000"
}
```

**Response:** `201 Created`
```json
{
  "data": null,
  "message": "Role assigned to user successfully"
}
```

**Error Responses:**
- `401 Unauthorized`: Missing or invalid token
- `403 Forbidden`: User does not have admin role
- `404 Not Found`: User or role not found

**Example:**
```bash
curl -X POST http://localhost:8000/admin/users/550e8400-e29b-41d4-a716-446655440000/roles \
  -H "Authorization: Bearer <token>" \
  -H "Content-Type: application/json" \
  -d '{
    "role_id": "660e8400-e29b-41d4-a716-446655440000"
  }'
```

#### DELETE /admin/users/{user_id}/roles/{role_id}
Remove a role from a user.

**Headers:** `Authorization: Bearer <token>`

**Path Parameters:**
- `user_id`: UUID of the user
- `role_id`: UUID of the role

**Response:** `204 No Content`

**Error Responses:**
- `401 Unauthorized`: Missing or invalid token
- `403 Forbidden`: User does not have admin role
- `404 Not Found`: User-role assignment not found

**Example:**
```bash
curl -X DELETE http://localhost:8000/admin/users/550e8400-e29b-41d4-a716-446655440000/roles/660e8400-e29b-41d4-a716-446655440000 \
  -H "Authorization: Bearer <token>"
```

#### POST /admin/roles/{role_id}/permissions
Assign a permission to a role.

**Headers:** `Authorization: Bearer <token>`

**Path Parameters:**
- `role_id`: UUID of the role

**Request:**
```json
{
  "permission_id": "990e8400-e29b-41d4-a716-446655440000"
}
```

**Response:** `201 Created`
```json
{
  "data": null,
  "message": "Permission assigned to role successfully"
}
```

**Error Responses:**
- `401 Unauthorized`: Missing or invalid token
- `403 Forbidden`: User does not have admin role
- `404 Not Found`: Role or permission not found

**Example:**
```bash
curl -X POST http://localhost:8000/admin/roles/660e8400-e29b-41d4-a716-446655440000/permissions \
  -H "Authorization: Bearer <token>" \
  -H "Content-Type: application/json" \
  -d '{
    "permission_id": "990e8400-e29b-41d4-a716-446655440000"
  }'
```

---

## Weather Service (Port 8001)

Base URL: `http://localhost:8001`

### Public Endpoints

#### GET /health
Health check endpoint (no authentication required).

**Response:** `200 OK`
```
OK
```

### Protected Endpoints (Require Valid JWT with `weather:read` Permission)

All weather endpoints require:
1. Valid JWT token in Authorization header
2. User must have `weather:read` permission (included in default "user" role)

**Note:** JWT validation is performed locally by the Weather Service using the shared JWT secret. The service does not make HTTP calls to the Auth Service for token validation.

#### GET /weather/{city}
Get aggregated weather data for a city. Data is cached for 30 minutes.

**Headers:** `Authorization: Bearer <token>`

**Path Parameters:**
- `city`: City name (e.g., "London", "New York")

**Query Parameters:**
- `cache` (optional): Set to `false` to force fresh data from external APIs

**Response:** `200 OK`
```json
{
  "data": {
    "city": "London",
    "timestamp": "2024-01-15T10:30:45.123456Z",
    "aggregated": {
      "temperature": 15.5,
      "condition": "Partly Cloudy",
      "humidity": 65,
      "wind_speed": 12.3
    },
    "sources": [
      {
        "provider": "metaweather",
        "temperature": 15.0,
        "condition": "Light Cloud",
        "humidity": 60,
        "wind_speed": 10.0
      },
      {
        "provider": "openmeteo",
        "temperature": 16.0,
        "condition": "Partly Cloudy",
        "humidity": 70,
        "wind_speed": 14.6
      }
    ]
  }
}
```

**Error Responses:**
- `401 Unauthorized`: Missing or invalid token
- `403 Forbidden`: User does not have `weather:read` permission
- `500 Internal Server Error`: Failed to fetch or aggregate weather data

**Example:**
```bash
# Get cached weather data
curl http://localhost:8001/weather/London \
  -H "Authorization: Bearer <token>"

# Force fresh data
curl "http://localhost:8001/weather/London?cache=false" \
  -H "Authorization: Bearer <token>"
```

#### GET /weather/{city}/providers
Get weather data from individual providers without aggregation.

**Headers:** `Authorization: Bearer <token>`

**Path Parameters:**
- `city`: City name (e.g., "London", "New York")

**Response:** `200 OK`
```json
{
  "data": [
    {
      "provider": "metaweather",
      "temperature": 15.0,
      "condition": "Light Cloud",
      "humidity": 60,
      "wind_speed": 10.0
    },
    {
      "provider": "openmeteo",
      "temperature": 16.0,
      "condition": "Partly Cloudy",
      "humidity": 70,
      "wind_speed": 14.6
    }
  ]
}
```

**Error Responses:**
- `401 Unauthorized`: Missing or invalid token
- `403 Forbidden`: User does not have `weather:read` permission
- `500 Internal Server Error`: Failed to fetch weather data

**Example:**
```bash
curl http://localhost:8001/weather/London/providers \
  -H "Authorization: Bearer <token>"
```

**Caching Details:**
- Cache TTL: 30 minutes (1800 seconds)
- Cache key: City name (case-sensitive)
- Cache storage: In-memory (DashMap)
- Rate limiting: Minimum 1 second delay between API calls per provider

---

## Time Service (Port 8002)

Base URL: `http://localhost:8002`

### Public Endpoints

#### GET /health
Health check endpoint (no authentication required).

**Response:** `200 OK`
```
OK
```

### Protected Endpoints (Require Valid JWT with `time:read` Permission)

All time endpoints require:
1. Valid JWT token in Authorization header
2. User must have `time:read` permission (included in default "user" role)

**Note:** JWT validation is performed locally by the Time Service using the shared JWT secret. The service does not make HTTP calls to the Auth Service for token validation.

#### GET /time/{city}
Get current time for a city. Supports major cities with automatic timezone mapping.

**Headers:** `Authorization: Bearer <token>`

**Path Parameters:**
- `city`: City name (supported cities: London, New York, NYC, Los Angeles, LA, Tokyo, Shanghai, Paris, Berlin, Sydney, Chicago, Toronto)

**Response:** `200 OK`
```json
{
  "data": {
    "city": "London",
    "timezone": "Europe/London",
    "datetime": "2024-01-15T10:30:45.123456+00:00",
    "utc_offset": "+00:00",
    "unix_time": 1705315845
  }
}
```

**Error Responses:**
- `401 Unauthorized`: Missing or invalid token
- `403 Forbidden`: User does not have `time:read` permission
- `404 Not Found`: City not supported

**Example:**
```bash
curl http://localhost:8002/time/London \
  -H "Authorization: Bearer <token>"
```

**Supported Cities:**
- London → Europe/London
- New York, NYC → America/New_York
- Los Angeles, LA → America/Los_Angeles
- Tokyo → Asia/Tokyo
- Shanghai → Asia/Shanghai
- Paris → Europe/Paris
- Berlin → Europe/Berlin
- Sydney → Australia/Sydney
- Chicago → America/Chicago
- Toronto → America/Toronto

#### GET /time/timezone/{timezone}
Get current time for a specific timezone.

**Headers:** `Authorization: Bearer <token>`

**Path Parameters:**
- `timezone`: IANA timezone identifier (e.g., "Europe/London", "America/New_York")

**Response:** `200 OK`
```json
{
  "data": {
    "city": "Europe/London",
    "timezone": "Europe/London",
    "datetime": "2024-01-15T10:30:45.123456+00:00",
    "utc_offset": "+00:00",
    "unix_time": 1705315845
  }
}
```

**Error Responses:**
- `401 Unauthorized`: Missing or invalid token
- `403 Forbidden`: User does not have `time:read` permission
- `404 Not Found`: Timezone data not found (cache miss and API unavailable)

**Example:**
```bash
curl http://localhost:8002/time/timezone/Europe/London \
  -H "Authorization: Bearer <token>"
```

#### GET /time/timezones
List all cached timezones.

**Headers:** `Authorization: Bearer <token>`

**Response:** `200 OK`
```json
{
  "data": [
    "Europe/London",
    "America/New_York",
    "Asia/Tokyo",
    "Europe/Paris",
    "America/Los_Angeles"
  ]
}
```

**Error Responses:**
- `401 Unauthorized`: Missing or invalid token
- `403 Forbidden`: User does not have `time:read` permission

**Example:**
```bash
curl http://localhost:8002/time/timezones \
  -H "Authorization: Bearer <token>"
```

**Caching Details:**
- Cache is pre-populated on service startup with major timezones
- Cache is refreshed every hour in the background
- Cache storage: In-memory (Arc<RwLock>)
- External API: WorldTimeAPI.org
- API timeout: 5 seconds

---

## Error Responses

All endpoints may return the following error responses:

### 400 Bad Request
Invalid request parameters or missing required fields.

```json
{
  "error": "Bad request: Username, email, and password are required"
}
```

### 401 Unauthorized
Missing or invalid authentication token.

```json
{
  "error": "Unauthorized: Missing or invalid token"
}
```

### 403 Forbidden
User does not have required permissions or role.

```json
{
  "error": "Forbidden: Permission 'weather:read' required"
}
```

### 404 Not Found
Requested resource not found.

```json
{
  "error": "Not found: User with id 550e8400-e29b-41d4-a716-446655440000 not found"
}
```

### 409 Conflict
Resource conflict (e.g., duplicate username or email).

```json
{
  "error": "Conflict: Username already exists"
}
```

### 500 Internal Server Error
Internal server error occurred.

```json
{
  "error": "Internal server error: Failed to aggregate weather: Connection timeout"
}
```

---

## Authentication

### JWT Token Format

All protected endpoints require a JWT token in the Authorization header:

```
Authorization: Bearer <token>
```

### Token Details

- **Algorithm**: HS256 (HMAC SHA-256)
- **Expiration**: 24 hours from issuance
- **Claims**:
  - `sub`: User ID (UUID)
  - `username`: Username
  - `roles`: Array of role names
  - `permissions`: Array of permission names
  - `exp`: Expiration timestamp (Unix)
  - `iat`: Issued at timestamp (Unix)

### Token Validation

**Important:** JWT tokens are validated locally by each service using a shared secret. Services do not make HTTP calls to the Auth Service for token validation. This design provides:

- **Performance**: No network latency for token validation
- **Resilience**: Services can validate tokens even if Auth Service is temporarily unavailable
- **Scalability**: No single point of failure for authentication

### Obtaining a Token

1. Register a new user via `POST /auth/register`
2. Login via `POST /auth/login` to receive a JWT token
3. Include the token in the `Authorization` header for all protected endpoints

### Permission Model

#### Default Roles

- **user**: Standard user role
  - Permissions: `weather:read`, `time:read`
  - Automatically assigned to new users

- **admin**: Administrator role
  - Permissions: `user:read`, `user:write`
  - Required for all `/admin/*` endpoints

#### Permissions

- `user:read`: Read user data (admin endpoints)
- `user:write`: Create/update/delete users (admin endpoints)
- `weather:read`: Access weather data endpoints
- `time:read`: Access time data endpoints

### Example Authentication Flow

```bash
# 1. Register a new user
curl -X POST http://localhost:8000/auth/register \
  -H "Content-Type: application/json" \
  -d '{
    "username": "johndoe",
    "email": "john@example.com",
    "password": "securepass123"
  }'

# 2. Login to get token
TOKEN=$(curl -X POST http://localhost:8000/auth/login \
  -H "Content-Type: application/json" \
  -d '{
    "username": "johndoe",
    "password": "securepass123"
  }' | jq -r '.data.token')

# 3. Use token for protected endpoints
curl http://localhost:8001/weather/London \
  -H "Authorization: Bearer $TOKEN"
```
