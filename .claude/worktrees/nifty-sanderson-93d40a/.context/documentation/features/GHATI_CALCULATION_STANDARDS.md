# Ghati Calculation Standards - Technical Specification

## 🎯 Objective
Define the most appropriate method for calculating Ghati time intervals and mapping them to UTC timestamps for the Selemene Engine.

## 📚 Vedic Time System Fundamentals

### Traditional Ghati System
- **1 Day** = 60 Ghatis
- **1 Ghati** = 24 minutes (1,440 seconds)
- **1 Ghati** = 60 Palas
- **1 Pala** = 24 seconds
- **1 Pala** = 60 Vipalas
- **1 Vipala** = 0.4 seconds

### Time Measurement Methods

#### Method 1: Fixed Interval System
```
Day Duration: 24 hours (86,400 seconds)
Ghati Duration: 24 minutes (1,440 seconds)
Calculation: Fixed 24-minute intervals from midnight UTC
```

**Pros:**
- Simple implementation
- Consistent across all locations
- Easy to calculate and understand
- No dependency on sunrise/sunset

**Cons:**
- Not astronomically accurate
- Doesn't follow traditional Vedic principles
- May not align with natural day cycles

#### Method 2: Sunrise-to-Sunset Division
```
Day Duration: Sunrise to Sunset (varies by location and season)
Ghati Duration: (Sunset - Sunrise) / 60
Calculation: Divide daylight hours into 60 equal parts
```

**Pros:**
- Follows traditional Vedic principles
- Astronomically accurate for daylight
- Aligns with natural day cycles
- Location-specific accuracy

**Cons:**
- Complex implementation
- Requires sunrise/sunset calculations
- Varies significantly by season and latitude
- Night time not accounted for

#### Method 3: Solar Time Division
```
Day Duration: 24 hours based on local solar time
Ghati Duration: 24 minutes of solar time
Calculation: Based on local solar noon and longitude
```

**Pros:**
- Astronomically accurate
- Accounts for longitude differences
- Traditional Vedic approach
- Consistent with solar day

**Cons:**
- Complex longitude calculations
- Requires precise solar time computation
- May not align with standard time zones

#### Method 4: Hybrid System (Recommended)
```
Day Duration: 24 hours (86,400 seconds)
Ghati Duration: 24 minutes (1,440 seconds)
Calculation: Fixed intervals with solar time adjustments
Adjustment: Apply longitude-based solar time correction
```

**Pros:**
- Combines simplicity with accuracy
- Maintains fixed intervals for consistency
- Includes solar time corrections
- Practical for modern applications

**Cons:**
- More complex than pure fixed system
- Requires longitude-based calculations

## 🔍 Detailed Analysis of Each Method

### Method 1: Fixed Interval System

#### Implementation Details
```rust
pub struct FixedGhatiCalculator {
    base_ghati_duration: Duration, // 24 minutes
}

impl FixedGhatiCalculator {
    pub fn calculate_ghati(&self, utc_time: DateTime<Utc>) -> GhatiTime {
        let seconds_since_midnight = utc_time.time().num_seconds_from_midnight() as u32;
        let ghati_number = (seconds_since_midnight / 1440) % 60;
        let pala_number = ((seconds_since_midnight % 1440) / 24) % 60;
        let vipala_number = (seconds_since_midnight % 24) * 2.5; // 24 seconds = 60 vipalas
        
        GhatiTime {
            ghati: ghati_number as u8,
            pala: pala_number as u8,
            vipala: vipala_number as u8,
            utc_timestamp: utc_time,
        }
    }
}
```

#### Use Cases
- Simple time tracking applications
- Basic Panchanga calculations
- Educational purposes
- Quick prototyping

### Method 2: Sunrise-to-Sunset Division

#### Implementation Details
```rust
pub struct SunriseSunsetGhatiCalculator {
    location: Coordinates,
    ephemeris: SwissEphemerisEngine,
}

impl SunriseSunsetGhatiCalculator {
    pub fn calculate_ghati(&self, utc_time: DateTime<Utc>) -> GhatiTime {
        let date = utc_time.date_naive();
        let sunrise = self.calculate_sunrise(date, self.location);
        let sunset = self.calculate_sunset(date, self.location);
        
        let daylight_duration = sunset - sunrise;
        let ghati_duration = daylight_duration / 60;
        
        if utc_time < sunrise {
            // Before sunrise - use previous day's calculation
            return self.calculate_previous_day_ghati(utc_time);
        } else if utc_time > sunset {
            // After sunset - use next day's calculation
            return self.calculate_next_day_ghati(utc_time);
        }
        
        let time_since_sunrise = utc_time - sunrise;
        let ghati_number = (time_since_sunrise.num_seconds() / ghati_duration.num_seconds()) % 60;
        
        GhatiTime {
            ghati: ghati_number as u8,
            pala: 0, // Calculate based on ghati_duration
            vipala: 0, // Calculate based on ghati_duration
            utc_timestamp: utc_time,
        }
    }
}
```

#### Use Cases
- Traditional Vedic applications
- Astrological calculations
- Religious observances
- Seasonal festivals

### Method 3: Solar Time Division

#### Implementation Details
```rust
pub struct SolarTimeGhatiCalculator {
    location: Coordinates,
    ephemeris: SwissEphemerisEngine,
}

impl SolarTimeGhatiCalculator {
    pub fn calculate_ghati(&self, utc_time: DateTime<Utc>) -> GhatiTime {
        let solar_time = self.utc_to_solar_time(utc_time, self.location);
        let solar_seconds = solar_time.time().num_seconds_from_midnight() as u32;
        
        let ghati_number = (solar_seconds / 1440) % 60;
        let pala_number = ((solar_seconds % 1440) / 24) % 60;
        let vipala_number = (solar_seconds % 24) * 2.5;
        
        GhatiTime {
            ghati: ghati_number as u8,
            pala: pala_number as u8,
            vipala: vipala_number as u8,
            utc_timestamp: utc_time,
        }
    }
    
    fn utc_to_solar_time(&self, utc_time: DateTime<Utc>, location: Coordinates) -> DateTime<Utc> {
        let longitude_offset = location.longitude * 4.0; // 1 degree = 4 minutes
        let equation_of_time = self.calculate_equation_of_time(utc_time);
        
        let solar_time = utc_time + Duration::minutes(longitude_offset as i64) + Duration::minutes(equation_of_time);
        solar_time
    }
}
```

#### Use Cases
- High-precision astronomical calculations
- Scientific applications
- Traditional Vedic astronomy
- Longitude-specific calculations

### Method 4: Hybrid System (Recommended)

#### Implementation Details
```rust
pub struct HybridGhatiCalculator {
    base_calculator: FixedGhatiCalculator,
    solar_correction: SolarTimeCorrection,
    location: Coordinates,
}

impl HybridGhatiCalculator {
    pub fn calculate_ghati(&self, utc_time: DateTime<Utc>) -> GhatiTime {
        // Get base Ghati from fixed system
        let base_ghati = self.base_calculator.calculate_ghati(utc_time);
        
        // Apply solar time correction
        let solar_correction = self.solar_correction.calculate_correction(utc_time, self.location);
        
        // Adjust Ghati based on solar correction
        let corrected_ghati = self.apply_solar_correction(base_ghati, solar_correction);
        
        corrected_ghati
    }
    
    fn apply_solar_correction(&self, base_ghati: GhatiTime, correction: Duration) -> GhatiTime {
        let correction_seconds = correction.num_seconds();
        let ghati_adjustment = correction_seconds / 1440; // 1440 seconds per Ghati
        let pala_adjustment = (correction_seconds % 1440) / 24; // 24 seconds per Pala
        
        GhatiTime {
            ghati: (base_ghati.ghati as i32 + ghati_adjustment) as u8,
            pala: (base_ghati.pala as i32 + pala_adjustment) as u8,
            vipala: base_ghati.vipala,
            utc_timestamp: base_ghati.utc_timestamp,
        }
    }
}
```

#### Use Cases
- Production applications
- Modern Vedic software
- Mobile applications
- Web services

## 🎯 Recommended Standard: Hybrid System

### Rationale for Selection

1. **Practicality**: Maintains fixed 24-minute intervals for consistency
2. **Accuracy**: Includes solar time corrections for astronomical accuracy
3. **Flexibility**: Can be configured for different precision requirements
4. **Compatibility**: Works with existing time systems and APIs
5. **Performance**: Efficient calculation without complex astronomical computations

### Implementation Strategy

#### Phase 1: Basic Hybrid System
- Implement fixed 24-minute Ghati intervals
- Add basic longitude-based solar time correction
- Support for standard time zones

#### Phase 2: Enhanced Solar Corrections
- Add equation of time calculations
- Include seasonal variations
- Support for different calculation methods

#### Phase 3: Advanced Features
- Multiple calculation methods (user selectable)
- Historical Ghati calculations
- High-precision corrections

### Configuration Options

```rust
pub struct GhatiCalculationConfig {
    pub method: GhatiCalculationMethod,
    pub precision: GhatiPrecision,
    pub solar_correction: bool,
    pub equation_of_time: bool,
    pub seasonal_adjustment: bool,
}

pub enum GhatiCalculationMethod {
    Fixed,           // Method 1
    SunriseSunset,   // Method 2
    SolarTime,       // Method 3
    Hybrid,          // Method 4 (Recommended)
}

pub enum GhatiPrecision {
    Standard,        // Ghati level
    High,           // Pala level
    Extreme,        // Vipala level
}
```

## 📊 Performance Comparison

| Method | Complexity | Accuracy | Performance | Use Case |
|--------|------------|----------|-------------|----------|
| Fixed | Low | Low | High | Simple apps |
| Sunrise-Sunset | High | High | Medium | Traditional |
| Solar Time | High | Very High | Medium | Scientific |
| Hybrid | Medium | High | High | Production |

## 🔧 Technical Implementation

### Core Data Structures

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GhatiTime {
    pub ghati: u8,           // 0-59
    pub pala: u8,            // 0-59
    pub vipala: u8,          // 0-59
    pub utc_timestamp: DateTime<Utc>,
    pub location: Coordinates,
    pub calculation_method: GhatiCalculationMethod,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GhatiBoundary {
    pub ghati_number: u8,
    pub utc_timestamp: DateTime<Utc>,
    pub local_time: DateTime<FixedOffset>,
    pub panchanga: Option<PanchangaResult>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GhatiTransition {
    pub from_ghati: u8,
    pub to_ghati: u8,
    pub transition_time: DateTime<Utc>,
    pub panchanga_change: Option<PanchangaChange>,
}
```

### API Interface

```rust
pub trait GhatiCalculator {
    fn calculate_ghati(&self, utc_time: DateTime<Utc>, location: Coordinates) -> GhatiTime;
    fn calculate_ghati_boundaries(&self, date: Date, location: Coordinates) -> Vec<GhatiBoundary>;
    fn get_next_ghati_transition(&self, current_time: DateTime<Utc>, location: Coordinates) -> GhatiTransition;
    fn utc_to_ghati(&self, utc_time: DateTime<Utc>, location: Coordinates) -> GhatiTime;
    fn ghati_to_utc(&self, ghati_time: GhatiTime, location: Coordinates) -> DateTime<Utc>;
}
```

## 🎯 Conclusion

The **Hybrid System (Method 4)** is recommended as the standard for the Selemene Engine because it:

1. **Balances simplicity with accuracy**
2. **Provides consistent 24-minute intervals**
3. **Includes solar time corrections for precision**
4. **Supports multiple precision levels**
5. **Enables future enhancements**

This approach will serve as the foundation for all Ghati-based time calculations in the Selemene Engine, providing a robust and flexible system that can be extended and refined as needed.

## 📋 Next Steps

1. **Implement HybridGhatiCalculator**: Core calculation engine
2. **Add Solar Time Corrections**: Longitude and equation of time
3. **Create Configuration System**: User-selectable calculation methods
4. **Build Test Suite**: Validation against known Ghati calculations
5. **Integrate with Panchanga**: Connect Ghati timing to Panchanga elements
