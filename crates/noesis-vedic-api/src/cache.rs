//! Aggressive caching layer for FreeAstrologyAPI.com
//! 
//! Strategy: With 50 requests/day limit, we cache aggressively:
//! - Birth charts: Forever (birth data never changes)
//! - Panchang: 24 hours (same date/location)
//! - Transits: 1 hour (current positions)
//! - Dasha periods: Forever (based on birth data)

use std::collections::HashMap;
use std::hash::Hash;
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::time::{Duration, Instant};
use tracing::{debug, info, warn};

use crate::panchang::Panchang;
use crate::dasha::VimshottariDasha;
use crate::chart::BirthChart;

/// Cache entry with expiration
#[derive(Debug, Clone)]
struct CacheEntry<T> {
    value: T,
    expires_at: Instant,
    access_count: u64,
}

impl<T> CacheEntry<T> {
    fn new(value: T, ttl: Duration) -> Self {
        Self {
            value,
            expires_at: Instant::now() + ttl,
            access_count: 1,
        }
    }
    
    fn is_expired(&self) -> bool {
        Instant::now() > self.expires_at
    }
    
    fn touch(&mut self) {
        self.access_count += 1;
    }
}

/// Multi-tier cache for API responses
#[derive(Debug, Clone)]
pub struct ApiCache {
    // L1: In-memory cache
    panchang: Arc<RwLock<HashMap<String, CacheEntry<Panchang>>>>,
    dasha: Arc<RwLock<HashMap<String, CacheEntry<VimshottariDasha>>>>,
    birth_chart: Arc<RwLock<HashMap<String, CacheEntry<BirthChart>>>>,
    
    // TTL configuration
    ttl_panchang: Duration,
    ttl_dasha: Duration,  // 0 = infinite
    ttl_birth_chart: Duration,  // 0 = infinite
    
    // Stats
    hits: Arc<RwLock<u64>>,
    misses: Arc<RwLock<u64>>,
}

impl ApiCache {
    /// Create cache with free plan optimized settings
    pub fn new() -> Self {
        Self::with_ttls(
            Duration::from_secs(86400),  // 24h for Panchang
            Duration::from_secs(0),      // Infinite for Dasha
            Duration::from_secs(0),      // Infinite for Birth Chart
        )
    }
    
    /// Create with custom TTLs
    pub fn with_ttls(
        panchang_ttl: Duration,
        dasha_ttl: Duration,
        birth_chart_ttl: Duration,
    ) -> Self {
        info!(
            "ApiCache initialized: Panchang={}s, Dasha={}s, BirthChart={}s",
            panchang_ttl.as_secs(),
            dasha_ttl.as_secs(),
            birth_chart_ttl.as_secs()
        );
        
        Self {
            panchang: Arc::new(RwLock::new(HashMap::new())),
            dasha: Arc::new(RwLock::new(HashMap::new())),
            birth_chart: Arc::new(RwLock::new(HashMap::new())),
            ttl_panchang: panchang_ttl,
            ttl_dasha: dasha_ttl,
            ttl_birth_chart: birth_chart_ttl,
            hits: Arc::new(RwLock::new(0)),
            misses: Arc::new(RwLock::new(0)),
        }
    }
    
    /// Get Panchang from cache
    pub async fn get_panchang(&self, key: &str) -> Option<Panchang> {
        self.get(&self.panchang, &key.to_string()).await
    }
    
    /// Store Panchang in cache
    pub async fn set_panchang(&self, key: &str, value: Panchang) {
        self.set(&self.panchang, key.to_string(), value, self.ttl_panchang).await;
    }
    
    /// Get Dasha from cache
    pub async fn get_dasha(&self, key: &str) -> Option<VimshottariDasha> {
        self.get(&self.dasha, &key.to_string()).await
    }
    
    /// Store Dasha in cache
    pub async fn set_dasha(&self, key: &str, value: VimshottariDasha) {
        let ttl = if self.ttl_dasha.as_secs() == 0 {
            Duration::from_secs(86400 * 365 * 10) // 10 years effectively infinite
        } else {
            self.ttl_dasha
        };
        self.set(&self.dasha, key.to_string(), value, ttl).await;
    }
    
    /// Get Birth Chart from cache
    pub async fn get_birth_chart(&self, key: &str) -> Option<BirthChart> {
        self.get(&self.birth_chart, &key.to_string()).await
    }
    
    /// Store Birth Chart in cache
    pub async fn set_birth_chart(&self, key: &str, value: BirthChart) {
        let ttl = if self.ttl_birth_chart.as_secs() == 0 {
            Duration::from_secs(86400 * 365 * 10) // 10 years
        } else {
            self.ttl_birth_chart
        };
        self.set(&self.birth_chart, key.to_string(), value, ttl).await;
    }
    
    /// Generic get from cache
    async fn get<K, V>(
        &self,
        cache: &Arc<RwLock<HashMap<K, CacheEntry<V>>>>,
        key: &K,
    ) -> Option<V>
    where
        K: Eq + Hash,
        V: Clone,
    {
        let mut cache = cache.write().await;
        
        if let Some(entry) = cache.get_mut(key) {
            if entry.is_expired() {
                debug!("Cache entry expired, removing");
                cache.remove(key);
                self.increment_misses().await;
                return None;
            }
            
            entry.touch();
            self.increment_hits().await;
            debug!("Cache hit, access count: {}", entry.access_count);
            return Some(entry.value.clone());
        }
        
        self.increment_misses().await;
        None
    }
    
    /// Generic set in cache
    async fn set<K, V>(
        &self,
        cache: &Arc<RwLock<HashMap<K, CacheEntry<V>>>>,
        key: K,
        value: V,
        ttl: Duration,
    ) where
        K: Eq + Hash,
    {
        let mut cache = cache.write().await;
        let entry = CacheEntry::new(value, ttl);
        cache.insert(key, entry);
        debug!("Cached entry with TTL {:?}", ttl);
    }
    
    /// Increment hit counter
    async fn increment_hits(&self) {
        let mut hits = self.hits.write().await;
        *hits += 1;
    }
    
    /// Increment miss counter
    async fn increment_misses(&self) {
        let mut misses = self.misses.write().await;
        *misses += 1;
    }
    
    /// Get cache statistics
    pub async fn stats(&self) -> CacheStats {
        let hits = *self.hits.read().await;
        let misses = *self.misses.read().await;
        let total = hits + misses;
        let hit_rate = if total > 0 {
            (hits as f64 / total as f64) * 100.0
        } else {
            0.0
        };
        
        CacheStats {
            hits,
            misses,
            total,
            hit_rate,
            panchang_entries: self.panchang.read().await.len(),
            dasha_entries: self.dasha.read().await.len(),
            birth_chart_entries: self.birth_chart.read().await.len(),
        }
    }
    
    /// Clear all caches
    pub async fn clear(&self) {
        info!("Clearing all caches");
        self.panchang.write().await.clear();
        self.dasha.write().await.clear();
        self.birth_chart.write().await.clear();
        *self.hits.write().await = 0;
        *self.misses.write().await = 0;
    }
    
    /// Clean expired entries
    pub async fn clean_expired(&self) {
        debug!("Cleaning expired cache entries");
        
        let now = Instant::now();
        
        // Clean Panchang
        {
            let mut cache = self.panchang.write().await;
            cache.retain(|_, entry| entry.expires_at > now);
        }
        
        // Note: Dasha and Birth Chart have very long TTLs, rarely need cleaning
    }
}

impl Default for ApiCache {
    fn default() -> Self {
        Self::new()
    }
}

/// Cache statistics
#[derive(Debug, Clone)]
pub struct CacheStats {
    pub hits: u64,
    pub misses: u64,
    pub total: u64,
    pub hit_rate: f64,
    pub panchang_entries: usize,
    pub dasha_entries: usize,
    pub birth_chart_entries: usize,
}

impl std::fmt::Display for CacheStats {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Cache Stats: {} hits, {} misses, {:.1}% hit rate | Entries: Panchang={}, Dasha={}, BirthChart={}",
            self.hits, self.misses, self.hit_rate,
            self.panchang_entries, self.dasha_entries, self.birth_chart_entries
        )
    }
}

/// Generate cache key for birth data
pub fn birth_key(
    year: i32,
    month: u32,
    day: u32,
    hour: u32,
    minute: u32,
    lat: f64,
    lng: f64,
) -> String {
    format!(
        "birth:{}-{:02}-{:02}T{:02}:{:02}:{:.4}:{:.4}",
        year, month, day, hour, minute, lat, lng
    )
}

/// Generate cache key for daily Panchang
pub fn panchang_key(
    year: i32,
    month: u32,
    day: u32,
    lat: f64,
    lng: f64,
) -> String {
    format!(
        "panchang:{}-{:02}-{:02}:{:.4}:{:.4}",
        year, month, day, lat, lng
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_cache_hit_miss() {
        let cache = ApiCache::new();
        let key = "test_key";
        
        // Miss
        assert!(cache.get_panchang(key).await.is_none());
        
        // Set
        let panchang = Panchang {
            date: crate::panchang::DateInfo {
                year: 2024,
                month: 1,
                day: 1,
                day_of_week: 1,
                julian_day: 2460315.0,
                hindu_date: None,
            },
            location: crate::panchang::Location {
                latitude: 12.97,
                longitude: 77.59,
                timezone: 5.5,
                name: None,
            },
            tithi: crate::panchang::Tithi {
                number: 1,
                name_tithi: crate::panchang::TithiName::Pratipada,
                start_time: "00:00".to_string(),
                end_time: "23:59".to_string(),
                is_complete: true,
            },
            nakshatra: crate::panchang::Nakshatra {
                number: 1,
                name_nakshatra: crate::panchang::NakshatraName::Ashwini,
                pada: 1,
                start_time: "00:00".to_string(),
                end_time: "23:59".to_string(),
                longitude: 0.0,
            },
            yoga: crate::panchang::Yoga {
                number: 1,
                name_yoga: crate::panchang::YogaName::Vishkumbha,
                start_time: "00:00".to_string(),
                end_time: "23:59".to_string(),
            },
            karana: crate::panchang::Karana {
                name_karana: crate::panchang::KaranaName::Bava,
                karana_type: crate::panchang::KaranaType::Movable,
                start_time: "00:00".to_string(),
                end_time: "23:59".to_string(),
            },
            vara: crate::panchang::Vara::Monday,
            paksha: crate::panchang::Paksha::Shukla,
            planets: crate::panchang::PlanetaryPositions {
                sun: crate::panchang::PlanetPosition {
                    name: "Sun".to_string(),
                    longitude: 120.0,
                    latitude: 0.0,
                    speed: 1.0,
                    sign: "Leo".to_string(),
                    nakshatra: "Magha".to_string(),
                    pada: 1,
                    is_retrograde: false,
                },
                moon: crate::panchang::PlanetPosition {
                    name: "Moon".to_string(),
                    longitude: 0.0,
                    latitude: 0.0,
                    speed: 13.0,
                    sign: "Aries".to_string(),
                    nakshatra: "Ashwini".to_string(),
                    pada: 1,
                    is_retrograde: false,
                },
                mars: None,
                mercury: None,
                jupiter: None,
                venus: None,
                saturn: None,
                rahu: None,
                ketu: None,
            },
            day_boundaries: crate::panchang::DayBoundaries {
                sunrise: "06:00".to_string(),
                sunset: "18:00".to_string(),
                next_sunrise: "06:01".to_string(),
                day_duration: "12:00".to_string(),
                night_duration: "12:00".to_string(),
            },
            ayanamsa: 24.0,
        };
        cache.set_panchang(key, panchang.clone()).await;
        
        // Hit
        let result = cache.get_panchang(key).await;
        assert!(result.is_some());
        
        // Check stats
        let stats = cache.stats().await;
        assert_eq!(stats.hits, 1);
        assert_eq!(stats.misses, 1);
    }

    #[test]
    fn test_birth_key_generation() {
        let key = birth_key(1991, 8, 13, 13, 31, 12.9716, 77.5946);
        assert!(key.contains("1991-08-13"));
        assert!(key.contains("13:31"));
        assert!(key.contains("12.9716"));
    }
}
