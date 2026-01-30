# Ghati-Based Time System API Documentation

## 🎯 Overview

The Selemene Engine now provides a comprehensive Ghati-based time system with RESTful API endpoints for time conversion, Panchanga integration, and real-time tracking. This system bridges Vedic time systems with Gregorian UTC time using the Ghati cadence.

## 🏗️ Architecture

### Core Components
- **Ghati Calculator**: Multiple calculation methods (Fixed, Hybrid, Sunrise-Sunset, Solar Time)
- **Panchanga Integration**: Connects Ghati timing to Panchanga calculations
- **Real-Time Tracker**: Live Ghati tracking with event broadcasting
- **API Layer**: RESTful endpoints for all functionality

### Time System
- **1 Day** = 60 Ghatis
- **1 Ghati** = 24 minutes (1,440 seconds)
- **1 Ghati** = 60 Palas
- **1 Pala** = 24 seconds
- **1 Pala** = 60 Vipalas
- **1 Vipala** = 0.4 seconds

## 📡 API Endpoints

### Base URL
```
https://api.selemene-engine.com/api/v1
```

### 1. Ghati Time Conversion

#### Calculate Ghati Time
```http
POST /ghati/calculate
Content-Type: application/json

{
  "utc_time": "2025-01-27T12:00:00Z",
  "location": {
    "latitude": 12.9629,
    "longitude": 77.5775,
    "altitude": 920.0
  },
  "calculation_method": "hybrid",
  "precision": "high"
}
```

**Response:**
```json
{
  "success": true,
  "data": {
    "ghati": 30,
    "pala": 15,
    "vipala": 30,
    "utc_timestamp": "2025-01-27T12:00:00Z",
    "local_time": "2025-01-27T17:30:00+05:30",
    "calculation_method": "hybrid",
    "precision": "high",
    "next_ghati_transition": {
      "from_ghati": 30,
      "to_ghati": 31,
      "transition_time": "2025-01-27T12:24:00Z",
      "time_until_transition": "24m 0s"
    }
  }
}
```

#### Get Current Ghati Time
```http
POST /ghati/current
Content-Type: application/json

{
  "location": {
    "latitude": 12.9629,
    "longitude": 77.5775
  },
  "calculation_method": "hybrid"
}
```

#### Get Ghati Boundaries
```http
POST /ghati/boundaries
Content-Type: application/json

{
  "date": "2025-01-27",
  "location": {
    "latitude": 12.9629,
    "longitude": 77.5775
  },
  "calculation_method": "hybrid"
}
```

#### UTC to Ghati Conversion
```http
POST /ghati/utc-to-ghati
Content-Type: application/json

{
  "utc_time": "2025-01-27T12:00:00Z",
  "location": {
    "latitude": 12.9629,
    "longitude": 77.5775
  }
}
```

#### Ghati to UTC Conversion
```http
POST /ghati/ghati-to-utc
Content-Type: application/json

{
  "ghati_time": {
    "ghati": 30,
    "pala": 15,
    "vipala": 30,
    "date": "2025-01-27"
  },
  "location": {
    "latitude": 12.9629,
    "longitude": 77.5775
  }
}
```

#### Get Ghati Calculation Methods
```http
GET /ghati/methods
```

**Response:**
```json
{
  "success": true,
  "data": {
    "fixed": {
      "name": "Fixed Interval",
      "description": "Fixed 24-minute intervals from midnight UTC",
      "accuracy": "Low",
      "complexity": "Low",
      "use_case": "Simple applications, educational purposes"
    },
    "hybrid": {
      "name": "Hybrid System",
      "description": "Fixed intervals with solar time corrections",
      "accuracy": "High",
      "complexity": "Medium",
      "use_case": "Production applications, modern Vedic software"
    },
    "sunrise_sunset": {
      "name": "Sunrise to Sunset",
      "description": "Divide daylight hours into 60 equal parts",
      "accuracy": "High",
      "complexity": "High",
      "use_case": "Traditional Vedic applications, astrological calculations"
    },
    "solar_time": {
      "name": "Solar Time",
      "description": "Based on local solar time and longitude",
      "accuracy": "Very High",
      "complexity": "High",
      "use_case": "Scientific applications, high-precision calculations"
    }
  }
}
```

### 2. Ghati-Panchanga Integration

#### Calculate Ghati-Panchanga
```http
POST /ghati-panchanga/calculate
Content-Type: application/json

{
  "utc_time": "2025-01-27T12:00:00Z",
  "location": {
    "latitude": 12.9629,
    "longitude": 77.5775
  },
  "calculation_method": "hybrid",
  "precision": "high"
}
```

**Response:**
```json
{
  "success": true,
  "data": {
    "ghati_time": {
      "ghati": 30,
      "pala": 15,
      "vipala": 30,
      "utc_timestamp": "2025-01-27T12:00:00Z",
      "calculation_method": "hybrid",
      "precision": "high"
    },
    "panchanga": {
      "tithi": 15.5,
      "nakshatra": 20.2,
      "yoga": 25.8,
      "karana": 7.1,
      "vara": 1.0,
      "solar_longitude": 120.5,
      "lunar_longitude": 135.2
    },
    "next_change": {
      "ghati_transition": {
        "from_ghati": 30,
        "to_ghati": 31,
        "transition_time": "2025-01-27T12:24:00Z"
      },
      "changed_element": "tithi",
      "old_value": 15.5,
      "new_value": 15.6,
      "change_time": "2025-01-27T12:24:00Z"
    }
  }
}
```

#### Get Current Ghati-Panchanga
```http
POST /ghati-panchanga/current
Content-Type: application/json

{
  "location": {
    "latitude": 12.9629,
    "longitude": 77.5775
  }
}
```

#### Get Daily Ghati-Panchanga
```http
POST /ghati-panchanga/daily
Content-Type: application/json

{
  "date": "2025-01-27",
  "location": {
    "latitude": 12.9629,
    "longitude": 77.5775
  },
  "calculation_method": "hybrid"
}
```

#### Find Next Panchanga Change
```http
POST /ghati-panchanga/next-change
Content-Type: application/json

{
  "location": {
    "latitude": 12.9629,
    "longitude": 77.5775
  },
  "max_ghatis": 10,
  "calculation_method": "hybrid"
}
```

#### Get Ghati Timing for Panchanga Element Changes
```http
POST /ghati-panchanga/element-changes
Content-Type: application/json

{
  "date": "2025-01-27",
  "location": {
    "latitude": 12.9629,
    "longitude": 77.5775
  },
  "element": "tithi",
  "calculation_method": "hybrid"
}
```

#### Calculate Panchanga with Ghati Precision
```http
POST /ghati-panchanga/precision
Content-Type: application/json

{
  "date": "2025-01-27",
  "latitude": 12.9629,
  "longitude": 77.5775,
  "ghati_precision": "high",
  "calculation_method": "hybrid"
}
```

#### Get Available Panchanga Elements
```http
GET /ghati-panchanga/elements
```

**Response:**
```json
{
  "success": true,
  "data": {
    "tithi": {
      "name": "Tithi",
      "description": "Lunar day (1-30)",
      "unit": "days",
      "precision": "high"
    },
    "nakshatra": {
      "name": "Nakshatra",
      "description": "Lunar mansion (1-27)",
      "unit": "degrees",
      "precision": "high"
    },
    "yoga": {
      "name": "Yoga",
      "description": "Auspicious combination (1-27)",
      "unit": "degrees",
      "precision": "medium"
    },
    "karana": {
      "name": "Karana",
      "description": "Half Tithi (1-11)",
      "unit": "half_days",
      "precision": "high"
    },
    "vara": {
      "name": "Vara",
      "description": "Weekday (1-7)",
      "unit": "days",
      "precision": "low"
    }
  }
}
```

### 3. Real-Time Tracking

#### Create Tracker
```http
POST /realtime/tracker/create
Content-Type: application/json

{
  "tracker_id": "my_tracker",
  "location": {
    "latitude": 12.9629,
    "longitude": 77.5775,
    "altitude": 920.0
  },
  "update_interval_ms": 1000,
  "precision": "high",
  "calculation_method": "hybrid",
  "enable_panchanga": true,
  "enable_notifications": true,
  "max_history": 1000
}
```

#### Start Tracker
```http
POST /realtime/tracker/start
Content-Type: application/json

{
  "tracker_id": "my_tracker"
}
```

#### Stop Tracker
```http
POST /realtime/tracker/stop
Content-Type: application/json

{
  "tracker_id": "my_tracker"
}
```

#### Get Tracker State
```http
POST /realtime/tracker/state
Content-Type: application/json

{
  "tracker_id": "my_tracker"
}
```

**Response:**
```json
{
  "success": true,
  "data": {
    "is_running": true,
    "current_ghati": {
      "ghati": 30,
      "pala": 15,
      "vipala": 30,
      "utc_timestamp": "2025-01-27T12:00:00Z",
      "calculation_method": "hybrid",
      "precision": "high"
    },
    "current_panchanga": {
      "tithi": 15.5,
      "nakshatra": 20.2,
      "yoga": 25.8,
      "karana": 7.1,
      "vara": 1.0
    },
    "next_transition": {
      "from_ghati": 30,
      "to_ghati": 31,
      "transition_time": "2025-01-27T12:24:00Z"
    },
    "next_panchanga_change": {
      "changed_element": "tithi",
      "change_time": "2025-01-27T12:24:00Z"
    },
    "location": {
      "latitude": 12.9629,
      "longitude": 77.5775,
      "altitude": 920.0
    },
    "last_update": "2025-01-27T12:00:00Z",
    "error_count": 0,
    "total_updates": 100
  }
}
```

#### Get Current Ghati from Tracker
```http
POST /realtime/tracker/current-ghati
Content-Type: application/json

{
  "tracker_id": "my_tracker"
}
```

#### Get Current Ghati-Panchanga from Tracker
```http
POST /realtime/tracker/current-ghati-panchanga
Content-Type: application/json

{
  "tracker_id": "my_tracker"
}
```

#### Get Next Ghati Transition from Tracker
```http
POST /realtime/tracker/next-transition
Content-Type: application/json

{
  "tracker_id": "my_tracker"
}
```

#### Get Time Until Next Ghati
```http
POST /realtime/tracker/time-until-next
Content-Type: application/json

{
  "tracker_id": "my_tracker"
}
```

#### Update Tracker Location
```http
POST /realtime/tracker/update-location
Content-Type: application/json

{
  "tracker_id": "my_tracker",
  "location": {
    "latitude": 19.0760,
    "longitude": 72.8777,
    "altitude": 14.0
  }
}
```

#### Remove Tracker
```http
POST /realtime/tracker/remove
Content-Type: application/json

{
  "tracker_id": "my_tracker"
}
```

#### List All Trackers
```http
GET /realtime/tracker/list
```

**Response:**
```json
{
  "success": true,
  "data": [
    {
      "tracker_id": "tracker_1",
      "status": "running",
      "location": {
        "latitude": 12.9629,
        "longitude": 77.5775,
        "altitude": 920.0
      },
      "created_at": "2025-01-27T10:00:00Z",
      "last_update": "2025-01-27T12:00:00Z"
    },
    {
      "tracker_id": "tracker_2",
      "status": "stopped",
      "location": {
        "latitude": 19.0760,
        "longitude": 72.8777,
        "altitude": 14.0
      },
      "created_at": "2025-01-27T11:00:00Z",
      "last_update": "2025-01-27T11:30:00Z"
    }
  ]
}
```

#### Get Tracker Configuration
```http
POST /realtime/tracker/config
Content-Type: application/json

{
  "tracker_id": "my_tracker"
}
```

#### Update Tracker Configuration
```http
POST /realtime/tracker/config/update
Content-Type: application/json

{
  "tracker_id": "my_tracker",
  "config": {
    "update_interval_ms": 500,
    "precision": "extreme",
    "enable_panchanga": true,
    "enable_notifications": false
  }
}
```

#### Get Tracking Events (WebSocket)
```http
POST /realtime/tracker/events
Content-Type: application/json

{
  "tracker_id": "my_tracker"
}
```

**Note:** This endpoint is designed for WebSocket connections for real-time event streaming.

#### Get Tracking Statistics
```http
GET /realtime/stats
```

**Response:**
```json
{
  "success": true,
  "data": {
    "total_trackers": 2,
    "active_trackers": 1,
    "total_events": 1500,
    "events_per_second": 1.0,
    "average_update_interval": 1000,
    "error_rate": 0.01,
    "uptime": "2h 30m 45s"
  }
}
```

## 🔧 Configuration Options

### Ghati Calculation Methods
- **fixed**: Fixed 24-minute intervals from midnight UTC
- **hybrid**: Fixed intervals with solar time corrections (Recommended)
- **sunrise_sunset**: Divide daylight hours into 60 equal parts
- **solar_time**: Based on local solar time and longitude

### Precision Levels
- **standard**: Ghati level only
- **high**: Ghati + Pala level
- **extreme**: Ghati + Pala + Vipala level

### Tracker Configuration
- **update_interval_ms**: Update interval in milliseconds (default: 1000)
- **precision**: Tracking precision level
- **calculation_method**: Ghati calculation method
- **enable_panchanga**: Enable Panchanga tracking (default: true)
- **enable_notifications**: Enable event notifications (default: true)
- **max_history**: Maximum history entries (default: 1000)

## 📊 Error Handling

All endpoints return consistent error responses:

```json
{
  "success": false,
  "error": "Error message description",
  "timestamp": "2025-01-27T12:00:00Z"
}
```

### Common Error Codes
- **400**: Bad Request - Invalid input parameters
- **404**: Not Found - Tracker or resource not found
- **500**: Internal Server Error - Server-side error

## 🚀 Usage Examples

### Basic Ghati Time Calculation
```bash
curl -X POST https://api.selemene-engine.com/api/v1/ghati/calculate \
  -H "Content-Type: application/json" \
  -d '{
    "location": {
      "latitude": 12.9629,
      "longitude": 77.5775
    },
    "calculation_method": "hybrid"
  }'
```

### Real-Time Tracking Setup
```bash
# Create tracker
curl -X POST https://api.selemene-engine.com/api/v1/realtime/tracker/create \
  -H "Content-Type: application/json" \
  -d '{
    "tracker_id": "my_tracker",
    "location": {
      "latitude": 12.9629,
      "longitude": 77.5775
    }
  }'

# Start tracking
curl -X POST https://api.selemene-engine.com/api/v1/realtime/tracker/start \
  -H "Content-Type: application/json" \
  -d '{"tracker_id": "my_tracker"}'

# Get current state
curl -X POST https://api.selemene-engine.com/api/v1/realtime/tracker/state \
  -H "Content-Type: application/json" \
  -d '{"tracker_id": "my_tracker"}'
```

### Daily Ghati-Panchanga Analysis
```bash
curl -X POST https://api.selemene-engine.com/api/v1/ghati-panchanga/daily \
  -H "Content-Type: application/json" \
  -d '{
    "date": "2025-01-27",
    "location": {
      "latitude": 12.9629,
      "longitude": 77.5775
    },
    "calculation_method": "hybrid"
  }'
```

## 🎯 Key Features

### 1. **Multiple Calculation Methods**
- Fixed interval system for simplicity
- Hybrid system with solar corrections for accuracy
- Sunrise-sunset division for traditional accuracy
- Solar time system for scientific precision

### 2. **Real-Time Tracking**
- Live Ghati time updates
- Panchanga change detection
- Event broadcasting system
- Multi-tracker management

### 3. **Comprehensive Integration**
- Ghati timing with Panchanga calculations
- Change detection and prediction
- Historical analysis capabilities
- Precision-based calculations

### 4. **Production Ready**
- RESTful API design
- Comprehensive error handling
- Scalable architecture
- Real-time event system

## 🔮 Future Enhancements

### Planned Features
- **WebSocket Support**: Real-time event streaming
- **Historical Data**: Past Ghati calculations
- **Batch Processing**: Multiple location calculations
- **Mobile SDK**: Native mobile integration
- **Advanced Analytics**: Usage patterns and insights

### Research Opportunities
- **Machine Learning**: Predictive Ghati calculations
- **Advanced Algorithms**: Optimized mathematical implementations
- **Distributed Processing**: Multi-instance calculation distribution
- **Real-Time Optimization**: Dynamic performance tuning

## 📚 Conclusion

The Ghati-based time system API provides a comprehensive solution for bridging Vedic time systems with modern UTC time. With multiple calculation methods, real-time tracking, and Panchanga integration, it serves as a robust foundation for Vedic astrology applications, cultural preservation, and scientific research.

The system is designed for production use with scalable architecture, comprehensive error handling, and extensive API coverage. It successfully combines traditional Vedic principles with modern software engineering practices.
