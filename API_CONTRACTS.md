# API Contracts

This document describes the API contracts for all microservices in the Karl Systems Challenge.

## Authentication Service (Port 8000)

### Public Endpoints

#### POST /auth/register
Register a new user.

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
    "user_id": "uuid",
    "username": "string",
    "email": "string"
  }
}
```

#### POST /auth/login
Login and receive JWT token.

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
    "token": "string",
    "user_id": "uuid",
    "username": "string",
    "roles": ["string"]
  }
}
```

### Admin Endpoints (Require Admin Role)

#### GET /admin/users
List all users.

**Headers:** `Authorization: Bearer <token>`

**Response:** `200 OK`
```json
{
  "data": [
    {
      "id": "uuid",
      "username": "string",
      "email": "string",
      "is_active": true,
      "created_at": "timestamp"
    }
  ]
}
```

#### POST /admin/users
Create a new user (admin only).

**Request:**
```json
{
  "username": "string",
  "email": "string",
  "password": "string"
}
```

#### GET /admin/users/{id}
Get user by ID.

#### PUT /admin/users/{id}
Update user.

**Request:**
```json
{
  "username": "string (optional)",
  "email": "string (optional)",
  "password": "string (optional)",
  "is_active": true (optional)
}
```

#### DELETE /admin/users/{id}
Delete user.

#### GET /admin/roles
List all roles.

#### POST /admin/roles
Create a new role.

**Request:**
```json
{
  "name": "string",
  "description": "string (optional)"
}
```

#### GET /admin/permissions
List all permissions.

#### POST /admin/permissions
Create a new permission.

**Request:**
```json
{
  "name": "string",
  "resource": "string",
  "action": "string"
}
```

#### POST /admin/users/{user_id}/roles
Assign role to user.

**Request:**
```json
{
  "role_id": "uuid"
}
```

#### DELETE /admin/users/{user_id}/roles/{role_id}
Remove role from user.

#### POST /admin/roles/{role_id}/permissions
Assign permission to role.

**Request:**
```json
{
  "permission_id": "uuid"
}
```

## Weather Service (Port 8001)

### Protected Endpoints (Require Valid JWT)

#### GET /weather/{city}
Get aggregated weather data for a city.

**Headers:** `Authorization: Bearer <token>`

**Query Parameters:**
- `cache` (optional): Set to `false` to force fresh data

**Response:** `200 OK`
```json
{
  "data": {
    "city": "string",
    "timestamp": "timestamp",
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
        "condition": "Light Cloud"
      },
      {
        "provider": "openmeteo",
        "temperature": 16.0,
        "condition": "Partly Cloudy"
      }
    ]
  }
}
```

#### GET /weather/{city}/providers
Get weather data from individual providers.

**Response:** `200 OK`
```json
{
  "data": [
    {
      "provider": "metaweather",
      "temperature": 15.0,
      "condition": "Light Cloud"
    }
  ]
}
```

## Time Service (Port 8002)

### Protected Endpoints (Require Valid JWT)

#### GET /time/{city}
Get current time for a city.

**Headers:** `Authorization: Bearer <token>`

**Response:** `200 OK`
```json
{
  "data": {
    "city": "string",
    "timezone": "Europe/London",
    "datetime": "2024-01-15T10:30:45.123456+00:00",
    "utc_offset": "+00:00",
    "unix_time": 1705315845
  }
}
```

#### GET /time/timezone/{timezone}
Get current time for a timezone.

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

#### GET /time/timezones
List all cached timezones.

**Response:** `200 OK`
```json
{
  "data": [
    "Europe/London",
    "America/New_York",
    "Asia/Tokyo"
  ]
}
```

## Error Responses

All endpoints may return the following error responses:

**400 Bad Request:**
```json
{
  "error": "Error message"
}
```

**401 Unauthorized:**
```json
{
  "error": "Missing or invalid token"
}
```

**403 Forbidden:**
```json
{
  "error": "Insufficient permissions"
}
```

**404 Not Found:**
```json
{
  "error": "Resource not found"
}
```

**500 Internal Server Error:**
```json
{
  "error": "Internal server error"
}
```

## Authentication

All protected endpoints require a JWT token in the Authorization header:

```
Authorization: Bearer <token>
```

Tokens are obtained by logging in through the auth service and expire after 24 hours.

