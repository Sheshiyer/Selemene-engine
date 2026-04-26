# Agent 34: Integration Guide
**Current Period Detection & Transition Tracking**

## Quick Start

### 1. Calculate Current Period

```rust
use engine_vimshottari::calculator::{
    calculate_mahadashas, 
    calculate_complete_timeline,
    find_current_period
};
use chrono::Utc;

// Assume we have birth data
let birth_time = Utc::now(); // Your birth time
let starting_planet = VedicPlanet::Ketu; // From nakshatra calculation
let balance = 4.375; // From balance calculation

// Generate complete timeline
let mahadashas = calculate_mahadashas(birth_time, starting_planet, balance);
let complete_timeline = calculate_complete_timeline(mahadashas);

// Find current period
let current = find_current_period(&complete_timeline, Utc::now()).unwrap();

println!("Currently in:");
println!("  Mahadasha: {:?} (ends {})", 
    current.mahadasha.planet, current.mahadasha.end);
println!("  Antardasha: {:?} (ends {})", 
    current.antardasha.planet, current.antardasha.end);
println!("  Pratyantardasha: {:?} (ends {})", 
    current.pratyantardasha.planet, current.pratyantardasha.end);
```

### 2. Get Upcoming Transitions

```rust
use engine_vimshottari::calculator::calculate_upcoming_transitions;

// Get next 10 transitions
let transitions = calculate_upcoming_transitions(
    &complete_timeline, 
    Utc::now(), 
    10
);

for t in transitions {
    match t.transition_type {
        TransitionType::Mahadasha => {
            println!("🌟 MAJOR: {} → {} in {} days", 
                t.from_planet, t.to_planet, t.days_until);
        },
        TransitionType::Antardasha => {
            println!("⭐ Medium: {} → {} in {} days", 
                t.from_planet, t.to_planet, t.days_until);
        },
        TransitionType::Pratyantardasha => {
            println!("✨ Minor: {} → {} in {} days", 
                t.from_planet, t.to_planet, t.days_until);
        },
    }
}
```

## API Endpoint Example

```rust
use axum::{extract::Query, Json};
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
struct PeriodQuery {
    birth_time: DateTime<Utc>,
    query_time: Option<DateTime<Utc>>,
    transition_count: Option<usize>,
}

#[derive(Serialize)]
struct PeriodResponse {
    current: CurrentPeriod,
    upcoming: Vec<UpcomingTransition>,
}

async fn get_current_periods(
    Query(params): Query<PeriodQuery>
) -> Json<PeriodResponse> {
    // Calculate from birth data (cached in production)
    let timeline = calculate_full_timeline(params.birth_time);
    
    let query_time = params.query_time.unwrap_or_else(Utc::now);
    let current = find_current_period(&timeline, query_time).unwrap();
    
    let count = params.transition_count.unwrap_or(5);
    let upcoming = calculate_upcoming_transitions(&timeline, query_time, count);
    
    Json(PeriodResponse { current, upcoming })
}
```

## Response Format

### Current Period Response
```json
{
  "current": {
    "mahadasha": {
      "planet": "Venus",
      "start": "2020-01-15T00:00:00Z",
      "end": "2040-01-15T00:00:00Z",
      "years": 20.0
    },
    "antardasha": {
      "planet": "Mercury",
      "start": "2024-09-15T00:00:00Z",
      "end": "2027-07-15T00:00:00Z",
      "years": 2.833
    },
    "pratyantardasha": {
      "planet": "Jupiter",
      "start": "2026-01-20T00:00:00Z",
      "end": "2026-05-31T00:00:00Z",
      "days": 131.47
    },
    "current_time": "2026-01-31T05:00:00Z"
  },
  "upcoming": [
    {
      "transition_type": "Pratyantardasha",
      "from_planet": "Jupiter",
      "to_planet": "Saturn",
      "transition_date": "2026-05-31T00:00:00Z",
      "days_until": 120
    }
  ]
}
```

## Caching Strategy

For production, cache the complete timeline by birth_time:

```rust
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

type TimelineCache = Arc<RwLock<HashMap<String, Vec<Mahadasha>>>>;

async fn get_or_calculate_timeline(
    cache: &TimelineCache,
    birth_time: DateTime<Utc>,
    starting_planet: VedicPlanet,
    balance: f64,
) -> Vec<Mahadasha> {
    let key = format!("{:?}_{:?}_{}", birth_time, starting_planet, balance);
    
    // Try read lock first
    {
        let read = cache.read().await;
        if let Some(timeline) = read.get(&key) {
            return timeline.clone();
        }
    }
    
    // Calculate and cache
    let mahadashas = calculate_mahadashas(birth_time, starting_planet, balance);
    let timeline = calculate_complete_timeline(mahadashas);
    
    let mut write = cache.write().await;
    write.insert(key, timeline.clone());
    timeline
}
```

## Notification System Integration

Track upcoming transitions and send notifications:

```rust
async fn check_upcoming_transitions(
    user_id: &str,
    birth_data: BirthData,
) -> Vec<Notification> {
    let timeline = get_cached_timeline(birth_data).await;
    let transitions = calculate_upcoming_transitions(&timeline, Utc::now(), 20);
    
    let mut notifications = Vec::new();
    
    for t in transitions {
        // Notify for major transitions in next 30 days
        if matches!(t.transition_type, TransitionType::Mahadasha) && t.days_until <= 30 {
            notifications.push(Notification {
                user_id: user_id.to_string(),
                title: format!("Major Period Shift: {} → {}", t.from_planet, t.to_planet),
                message: format!("Your Mahadasha changes in {} days", t.days_until),
                urgency: Urgency::High,
            });
        }
        
        // Notify for medium transitions in next 7 days
        if matches!(t.transition_type, TransitionType::Antardasha) && t.days_until <= 7 {
            notifications.push(Notification {
                user_id: user_id.to_string(),
                title: format!("Period Shift: {} → {}", t.from_planet, t.to_planet),
                message: format!("Your Antardasha changes in {} days", t.days_until),
                urgency: Urgency::Medium,
            });
        }
    }
    
    notifications
}
```

## Dashboard Widget Example

```typescript
interface CurrentPeriodWidget {
  title: string;
  mahadasha: PeriodInfo;
  antardasha: PeriodInfo;
  pratyantardasha: PeriodInfo;
  nextTransition: TransitionInfo;
}

interface PeriodInfo {
  planet: string;
  endsIn: string; // "in 5 years", "in 120 days"
  progress: number; // 0-100 percentage
}

function renderCurrentPeriods(data: PeriodResponse) {
  const now = new Date();
  
  const mahaProgress = calculateProgress(
    data.current.mahadasha.start,
    data.current.mahadasha.end,
    now
  );
  
  const nextTransition = data.upcoming[0];
  
  return {
    title: "Your Current Consciousness Periods",
    mahadasha: {
      planet: data.current.mahadasha.planet,
      endsIn: formatDuration(data.current.mahadasha.end, now),
      progress: mahaProgress
    },
    antardasha: {
      planet: data.current.antardasha.planet,
      endsIn: formatDuration(data.current.antardasha.end, now),
      progress: calculateProgress(
        data.current.antardasha.start,
        data.current.antardasha.end,
        now
      )
    },
    pratyantardasha: {
      planet: data.current.pratyantardasha.planet,
      endsIn: formatDuration(data.current.pratyantardasha.end, now),
      progress: calculateProgress(
        data.current.pratyantardasha.start,
        data.current.pratyantardasha.end,
        now
      )
    },
    nextTransition: {
      type: nextTransition.transition_type,
      from: nextTransition.from_planet,
      to: nextTransition.to_planet,
      daysUntil: nextTransition.days_until
    }
  };
}
```

## Performance Considerations

1. **Timeline Calculation**: O(729) - Do once per user, cache aggressively
2. **Current Period Lookup**: O(log 729) ≈ 10 comparisons - Very fast
3. **Transition Calculation**: O(n) where n = requested count - Typically fast for n ≤ 20

### Recommended Caching
- **Timeline**: Cache forever (doesn't change for a birth chart)
- **Current Period**: Cache for 1 day (rarely changes)
- **Transitions**: Cache for 1 day

## Error Handling

```rust
match find_current_period(&timeline, query_time) {
    Some(current) => {
        // Period found
        Ok(current)
    },
    None => {
        // Query time outside 120-year cycle
        Err(EngineError::InvalidInput(
            "Query time outside Vimshottari cycle range".to_string()
        ))
    }
}
```

## Next Steps

1. Add consciousness quality mappings for each planet
2. Implement period recommendations engine
3. Add historical period analysis
4. Create meditation/practice suggestions per period

---

**Integration Status**: Ready for production use  
**Dependencies**: Agents 31, 32, 33 (complete timeline calculation)  
**API Ready**: JSON serializable, cacheable, efficient
