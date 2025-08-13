# Selemene Engine API Documentation

## Overview

The Selemene Engine is a high-performance astronomical calculation engine for Panchanga and Vedic astrology. This API provides comprehensive access to astronomical calculations, caching, and performance optimization features.

## Base URL

- **Development**: `http://localhost:8080`
- **Staging**: `https://selemene-staging.railway.app`
- **Production**: `https://api.selemene.io`

## Authentication

The API supports two authentication methods:

### JWT Tokens
```http
Authorization: Bearer <jwt_token>
```

### API Keys
```http
Authorization: ApiKey <api_key>
```

Or as a query parameter:
```http
GET /api/v1/panchanga?api_key=<api_key>
```

## Rate Limiting

Rate limits are applied per user tier:

- **Free**: 100 requests/hour
- **Basic**: 1,000 requests/hour
- **Premium**: 10,000 requests/hour
- **Enterprise**: 100,000 requests/hour

## Endpoints

### Health & Status

#### GET /health
Check the health status of the engine.

**Response:**
```json
{
  "status": "healthy",
  "timestamp": "2025-01-27T17:00:00Z",
  "version": "1.0.0",
  "components": {
    "native_engine": "healthy",
    "swiss_ephemeris": "healthy",
    "cache_system": "healthy",
    "database": "healthy"
  }
}
```

#### GET /status
Get detailed system status and uptime.

**Response:**
```json
{
  "status": "operational",
  "uptime_seconds": 86400,
  "total_requests": 15000,
  "success_rate": 0.998,
  "average_response_time_ms": 45.2
}
```

#### GET /metrics
Get Prometheus-formatted metrics.

**Response:**
```
# HELP selemene_requests_total Total number of requests
# TYPE selemene_requests_total counter
selemene_requests_total 15000

# HELP selemene_request_duration_seconds Request duration in seconds
# TYPE selemene_request_duration_seconds histogram
selemene_request_duration_seconds_bucket{le="0.1"} 12000
selemene_request_duration_seconds_bucket{le="0.5"} 14000
selemene_request_duration_seconds_bucket{le="1.0"} 15000
```

### Core Calculations

#### POST /api/v1/panchanga
Calculate Panchanga (five elements) for a given date and location.

**Request Body:**
```json
{
  "date": "2025-01-27",
  "coordinates": {
    "latitude": 19.0760,
    "longitude": 72.8777
  },
  "precision": "Standard",
  "timezone": "Asia/Kolkata"
}
```

**Response:**
```json
{
  "status": "success",
  "data": {
    "date": "2025-01-27",
    "coordinates": {
      "latitude": 19.0760,
      "longitude": 72.8777
    },
    "tithi": {
      "name": "Shukla Paksha Pratipada",
      "number": 1,
      "start_time": "2025-01-27T06:30:00Z",
      "end_time": "2025-01-28T08:45:00Z"
    },
    "nakshatra": {
      "name": "Ashwini",
      "number": 1,
      "start_time": "2025-01-27T06:30:00Z",
      "end_time": "2025-01-28T09:15:00Z"
    },
    "yoga": {
      "name": "Vishkumbha",
      "start_time": "2025-01-27T06:30:00Z",
      "end_time": "2025-01-28T10:00:00Z"
    },
    "karana": {
      "name": "Bava",
      "start_time": "2025-01-27T06:30:00Z",
      "end_time": "2025-01-27T18:00:00Z"
    },
    "vara": {
      "name": "Monday",
      "number": 1
    },
    "solar_longitude": 307.5,
    "lunar_longitude": 0.8,
    "julian_day": 2460365.5,
    "calculation_time": "0.045s",
    "precision_used": "Standard",
    "backend_used": "native"
  }
}
```

#### POST /api/v1/panchanga/batch
Calculate Panchanga for multiple dates/locations in parallel.

**Request Body:**
```json
{
  "requests": [
    {
      "date": "2025-01-27",
      "coordinates": {"latitude": 19.0760, "longitude": 72.8777},
      "precision": "Standard"
    },
    {
      "date": "2025-06-15",
      "coordinates": {"latitude": 28.6139, "longitude": 77.2090},
      "precision": "High"
    }
  ]
}
```

#### POST /api/v1/panchanga/range
Calculate Panchanga for a date range.

**Request Body:**
```json
{
  "start_date": "2025-01-01",
  "end_date": "2025-01-31",
  "coordinates": {
    "latitude": 19.0760,
    "longitude": 72.8777
  },
  "precision": "Standard",
  "interval_days": 1
}
```

### Individual Elements

#### POST /api/v1/solar/position
Calculate solar position and velocity.

#### POST /api/v1/lunar/position
Calculate lunar position and velocity.

#### POST /api/v1/tithi
Calculate Tithi (lunar day) information.

#### POST /api/v1/nakshatra
Calculate Nakshatra (lunar mansion) information.

#### POST /api/v1/yoga
Calculate Yoga (solar-lunar combination).

#### POST /api/v1/karana
Calculate Karana (half-Tithi).

#### POST /api/v1/vara
Calculate Vara (weekday).

#### POST /api/v1/houses
Calculate astrological houses.

#### POST /api/v1/planets
Calculate planetary positions.

### Cache Management

#### GET /api/v1/cache/stats
Get cache performance statistics.

**Response:**
```json
{
  "l1_cache": {
    "hits": 8500,
    "misses": 1500,
    "hit_rate": 0.85,
    "size_mb": 128,
    "entries": 5000
  },
  "l2_cache": {
    "hits": 1200,
    "misses": 300,
    "hit_rate": 0.80,
    "ttl_seconds": 3600
  },
  "l3_cache": {
    "hits": 800,
    "misses": 200,
    "hit_rate": 0.80,
    "size_mb": 1024
  },
  "overall": {
    "total_hits": 10500,
    "total_misses": 2000,
    "overall_hit_rate": 0.84
  }
}
```

#### POST /api/v1/cache/clear
Clear all cache layers.

### Engine Management

#### GET /api/v1/engine/stats
Get engine performance statistics.

#### GET /api/v1/engine/config
Get current engine configuration.

#### POST /api/v1/engine/config
Update engine configuration.

### Performance Optimization

#### POST /api/v1/performance/optimize
Run performance optimization routines.

**Response:**
```json
{
  "status": "success",
  "message": "Performance optimization completed",
  "optimizations": [
    "Cache preloading",
    "Routing strategy adjustment",
    "Memory optimization"
  ],
  "timestamp": "2025-01-27T17:00:00Z"
}
```

#### POST /api/v1/performance/benchmark
Run performance benchmarks.

**Response:**
```json
{
  "status": "success",
  "message": "Benchmarks completed",
  "benchmarks": {
    "single_calculation": "0.5ms",
    "batch_calculation": "45.2ms",
    "cache_performance": "0.1ms",
    "memory_usage": "2.3ms"
  },
  "timestamp": "2025-01-27T17:00:00Z"
}
```

## Data Types

### Coordinates
```json
{
  "latitude": 19.0760,
  "longitude": 72.8777
}
```

### Precision Levels
- `"Standard"` - Standard precision (~1 arcminute)
- `"High"` - High precision (~0.1 arcminute)
- `"Extreme"` - Extreme precision (~0.01 arcminute)

### Timezone
ISO 8601 timezone identifier (e.g., "Asia/Kolkata", "UTC")

## Error Handling

All endpoints return consistent error responses:

```json
{
  "status": "error",
  "error": {
    "code": "VALIDATION_ERROR",
    "message": "Invalid date format",
    "details": "Date must be in YYYY-MM-DD format"
  },
  "timestamp": "2025-01-27T17:00:00Z"
}
```

### Error Codes
- `VALIDATION_ERROR` - Invalid input data
- `CALCULATION_ERROR` - Error during calculation
- `AUTHENTICATION_ERROR` - Invalid or missing authentication
- `RATE_LIMIT_EXCEEDED` - Rate limit exceeded
- `INTERNAL_ERROR` - Internal server error
- `SERVICE_UNAVAILABLE` - Service temporarily unavailable

## Response Headers

All responses include:
- `Content-Type: application/json`
- `X-Request-ID` - Unique request identifier
- `X-Response-Time` - Response time in milliseconds
- `X-Cache-Status` - Cache hit/miss status

## Pagination

For endpoints returning multiple results:

```json
{
  "status": "success",
  "data": [...],
  "pagination": {
    "page": 1,
    "per_page": 100,
    "total": 1000,
    "total_pages": 10,
    "has_next": true,
    "has_prev": false
  }
}
```

## WebSocket Support

WebSocket endpoints for real-time updates:

- `ws://localhost:8080/ws/panchanga` - Real-time Panchanga updates
- `ws://localhost:8080/ws/notifications` - System notifications

## SDKs and Libraries

Official client libraries:
- **Rust**: `selemene-engine-client`
- **Python**: `selemene-python`
- **JavaScript**: `selemene-js`
- **Go**: `selemene-go`

## Support

- **Documentation**: [https://docs.selemene.io](https://docs.selemene.io)
- **API Status**: [https://status.selemene.io](https://status.selemene.io)
- **Support Email**: support@selemene.io
- **GitHub Issues**: [https://github.com/selemene/selemene-engine](https://github.com/selemene/selemene-engine)
