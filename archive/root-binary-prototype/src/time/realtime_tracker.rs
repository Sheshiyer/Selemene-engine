use chrono::{DateTime, Utc, Duration};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::{RwLock, broadcast};
use tokio::time::interval;
use std::time::Duration as StdDuration;

use crate::{
    models::Coordinates,
    time::{
        GhatiCalculationConfig,
        GhatiCalculationMethod, GhatiPrecision, GhatiTime, GhatiTransition,
        GhatiPanchangaService, GhatiPanchangaResult, GhatiPanchangaChange,
        MockPanchangaCalculator
    },
};

/// Real-time Ghati tracking event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GhatiTrackingEvent {
    pub event_type: GhatiEventType,
    pub timestamp: DateTime<Utc>,
    pub ghati_time: GhatiTime,
    pub panchanga: Option<GhatiPanchangaResult>,
    pub change: Option<GhatiPanchangaChange>,
    pub location: Coordinates,
}

/// Types of Ghati tracking events
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum GhatiEventType {
    GhatiTransition,        // Ghati boundary crossed
    PanchangaChange,        // Panchanga element changed
    PrecisionUpdate,        // Precision level update
    LocationUpdate,         // Location changed
    ServiceStart,           // Tracking service started
    ServiceStop,            // Tracking service stopped
    Error,                  // Error occurred
}

/// Real-time Ghati tracker configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GhatiTrackerConfig {
    pub update_interval_ms: u64,        // Update interval in milliseconds
    pub precision: GhatiPrecision,      // Tracking precision
    pub calculation_method: GhatiCalculationMethod,
    pub enable_panchanga: bool,         // Enable Panchanga tracking
    pub enable_notifications: bool,     // Enable event notifications
    pub max_history: usize,             // Maximum history entries
}

impl Default for GhatiTrackerConfig {
    fn default() -> Self {
        Self {
            update_interval_ms: 1000,   // 1 second updates
            precision: GhatiPrecision::High,
            calculation_method: GhatiCalculationMethod::Hybrid,
            enable_panchanga: true,
            enable_notifications: true,
            max_history: 1000,
        }
    }
}

/// Real-time Ghati tracker state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GhatiTrackerState {
    pub is_running: bool,
    pub current_ghati: Option<GhatiTime>,
    pub current_panchanga: Option<GhatiPanchangaResult>,
    pub next_transition: Option<GhatiTransition>,
    pub next_panchanga_change: Option<GhatiPanchangaChange>,
    pub location: Option<Coordinates>,
    pub last_update: Option<DateTime<Utc>>,
    pub error_count: u64,
    pub total_updates: u64,
}

/// Real-time Ghati tracker
pub struct GhatiRealtimeTracker {
    config: GhatiTrackerConfig,
    state: Arc<RwLock<GhatiTrackerState>>,
    event_sender: broadcast::Sender<GhatiTrackingEvent>,
    ghati_service: Arc<GhatiPanchangaService>,
    shutdown_sender: broadcast::Sender<()>,
}

impl GhatiRealtimeTracker {
    pub fn new(config: GhatiTrackerConfig, location: Coordinates) -> Self {
        // Create Ghati calculation configuration
        let ghati_config = GhatiCalculationConfig {
            method: config.calculation_method,
            precision: config.precision,
            solar_correction: config.calculation_method == GhatiCalculationMethod::Hybrid,
            equation_of_time: config.calculation_method == GhatiCalculationMethod::Hybrid,
            seasonal_adjustment: false,
        };

        // Create Panchanga calculator (using mock for now)
        let panchanga_calculator = Arc::new(MockPanchangaCalculator);

        // Create Ghati-Panchanga service
        let ghati_service = Arc::new(GhatiPanchangaService::new(ghati_config, panchanga_calculator));

        // Create event channel
        let (event_sender, _) = broadcast::channel(1000);

        // Create shutdown channel
        let (shutdown_sender, _) = broadcast::channel(1);

        // Initialize state
        let state = Arc::new(RwLock::new(GhatiTrackerState {
            is_running: false,
            current_ghati: None,
            current_panchanga: None,
            next_transition: None,
            next_panchanga_change: None,
            location: Some(location),
            last_update: None,
            error_count: 0,
            total_updates: 0,
        }));

        Self {
            config,
            state,
            event_sender,
            ghati_service,
            shutdown_sender,
        }
    }

    /// Start real-time tracking
    pub async fn start(&self) -> Result<(), String> {
        let mut state = self.state.write().await;
        if state.is_running {
            return Err("Tracker is already running".to_string());
        }
        state.is_running = true;
        drop(state);

        // Send service start event
        self.send_event(GhatiEventType::ServiceStart, None, None).await;

        // Start tracking loop
        let tracker = self.clone();
        tokio::spawn(async move {
            if let Err(e) = tracker.tracking_loop().await {
                eprintln!("Tracking loop error: {}", e);
            }
        });

        Ok(())
    }

    /// Stop real-time tracking
    pub async fn stop(&self) -> Result<(), String> {
        let mut state = self.state.write().await;
        if !state.is_running {
            return Err("Tracker is not running".to_string());
        }
        state.is_running = false;
        drop(state);

        // Send shutdown signal
        let _ = self.shutdown_sender.send(());

        // Send service stop event
        self.send_event(GhatiEventType::ServiceStop, None, None).await;

        Ok(())
    }

    /// Update location
    pub async fn update_location(&self, location: Coordinates) -> Result<(), String> {
        let mut state = self.state.write().await;
        state.location = Some(location.clone());
        drop(state);

        // Send location update event
        self.send_event(GhatiEventType::LocationUpdate, None, None).await;

        Ok(())
    }

    /// Get current tracker state
    pub async fn get_state(&self) -> GhatiTrackerState {
        self.state.read().await.clone()
    }

    /// Subscribe to tracking events
    pub fn subscribe(&self) -> broadcast::Receiver<GhatiTrackingEvent> {
        self.event_sender.subscribe()
    }

    /// Get current Ghati time
    pub async fn get_current_ghati(&self) -> Result<GhatiTime, String> {
        let state = self.state.read().await;
        let location = state.location.as_ref()
            .ok_or("Location not set")?;

        self.ghati_service
            .calculate_ghati(Utc::now(), location.clone())
    }

    /// Get current Ghati-Panchanga
    pub async fn get_current_ghati_panchanga(&self) -> Result<GhatiPanchangaResult, String> {
        let state = self.state.read().await;
        let location = state.location.as_ref()
            .ok_or("Location not set")?;

        self.ghati_service.get_current_ghati_panchanga(location.clone()).await
    }

    /// Get next Ghati transition
    pub async fn get_next_ghati_transition(&self) -> Result<GhatiTransition, String> {
        let state = self.state.read().await;
        let location = state.location.as_ref()
            .ok_or("Location not set")?;

        self.ghati_service
            .get_next_ghati_transition(Utc::now(), location.clone())
    }

    /// Get time until next Ghati transition
    pub async fn get_time_until_next_ghati(&self) -> Result<Duration, String> {
        let transition = self.get_next_ghati_transition().await?;
        let now = Utc::now();
        Ok(transition.transition_time - now)
    }

    /// Main tracking loop
    async fn tracking_loop(&self) -> Result<(), String> {
        let mut interval = interval(StdDuration::from_millis(self.config.update_interval_ms));
        let mut shutdown_receiver = self.shutdown_sender.subscribe();

        loop {
            tokio::select! {
                _ = interval.tick() => {
                    if let Err(e) = self.update_tracking().await {
                        eprintln!("Update error: {}", e);
                        self.send_event(GhatiEventType::Error, None, None).await;
                    }
                }
                _ = shutdown_receiver.recv() => {
                    break;
                }
            }
        }

        Ok(())
    }

    /// Update tracking state
    async fn update_tracking(&self) -> Result<(), String> {
        let (location, prev_ghati, prev_panchanga) = {
            let state = self.state.read().await;
            let location = state
                .location
                .clone()
                .ok_or("Location not set")?;
            (location, state.current_ghati.clone(), state.current_panchanga.clone())
        };

        let current_time = Utc::now();

        // Update Ghati time
        let current_ghati = self
            .ghati_service
            .calculate_ghati(current_time, location.clone())?;

        // Check for Ghati transition
        let ghati_transition = if let Some(prev_ghati) = &prev_ghati {
            if prev_ghati.ghati != current_ghati.ghati {
                Some(
                    self.ghati_service
                        .get_next_ghati_transition(current_time, location.clone())?,
                )
            } else {
                None
            }
        } else {
            None
        };

        // Update Panchanga if enabled
        let current_panchanga = if self.config.enable_panchanga {
            match self.ghati_service.get_current_ghati_panchanga(location.clone()).await {
                Ok(panchanga) => Some(panchanga),
                Err(e) => {
                    eprintln!("Panchanga calculation error: {}", e);
                    None
                }
            }
        } else {
            None
        };

        // Check for Panchanga changes
        let panchanga_change = if let (Some(_prev_panchanga), Some(current_panchanga)) =
            (&prev_panchanga, &current_panchanga)
        {
            current_panchanga.next_change.clone()
        } else {
            None
        };

        let next_transition = self
            .ghati_service
            .get_next_ghati_transition(current_time, location.clone())
            .ok();
        let next_panchanga_change =
            current_panchanga.as_ref().and_then(|p| p.next_change.clone());

        // Update state
        let mut state = self.state.write().await;
        state.current_ghati = Some(current_ghati.clone());
        state.current_panchanga = current_panchanga.clone();
        state.last_update = Some(current_time);
        state.total_updates += 1;
        state.next_transition = next_transition;
        state.next_panchanga_change = next_panchanga_change;

        drop(state);

        // Send events
        if let Some(_transition) = ghati_transition {
            self.send_event(
                GhatiEventType::GhatiTransition,
                Some(current_ghati.clone()),
                None,
            )
            .await;
        }

        if let Some(change) = panchanga_change {
            self.send_event(
                GhatiEventType::PanchangaChange,
                Some(current_ghati),
                Some(change),
            )
            .await;
        }

        Ok(())
    }

    /// Send tracking event
    async fn send_event(
        &self,
        event_type: GhatiEventType,
        ghati_time: Option<GhatiTime>,
        change: Option<GhatiPanchangaChange>,
    ) {
        let state = self.state.read().await;
        let location = state.location.clone().unwrap_or(Coordinates {
            latitude: 12.9629,
            longitude: 77.5775,
            altitude: None,
        });

        let ghati_time = ghati_time.unwrap_or_else(|| GhatiTime {
            ghati: 0,
            pala: 0,
            vipala: 0,
            utc_timestamp: Utc::now(),
            location: location.clone(),
            calculation_method: GhatiCalculationMethod::Hybrid,
            precision: GhatiPrecision::High,
        });

        let event = GhatiTrackingEvent {
            event_type,
            timestamp: Utc::now(),
            ghati_time,
            panchanga: state.current_panchanga.clone(),
            change,
            location,
        };

        let _ = self.event_sender.send(event);
    }
}

impl Clone for GhatiRealtimeTracker {
    fn clone(&self) -> Self {
        Self {
            config: self.config.clone(),
            state: self.state.clone(),
            event_sender: self.event_sender.clone(),
            ghati_service: self.ghati_service.clone(),
            shutdown_sender: self.shutdown_sender.clone(),
        }
    }
}

/// Ghati tracking service manager
pub struct GhatiTrackingService {
    trackers: Arc<RwLock<std::collections::HashMap<String, GhatiRealtimeTracker>>>,
}

impl GhatiTrackingService {
    pub fn new() -> Self {
        Self {
            trackers: Arc::new(RwLock::new(std::collections::HashMap::new())),
        }
    }

    /// Create a new tracker
    pub async fn create_tracker(
        &self,
        id: String,
        config: GhatiTrackerConfig,
        location: Coordinates,
    ) -> Result<(), String> {
        let mut trackers = self.trackers.write().await;
        if trackers.contains_key(&id) {
            return Err(format!("Tracker with id '{}' already exists", id));
        }

        let tracker = GhatiRealtimeTracker::new(config, location);
        trackers.insert(id, tracker);
        Ok(())
    }

    /// Start a tracker
    pub async fn start_tracker(&self, id: &str) -> Result<(), String> {
        let trackers = self.trackers.read().await;
        let tracker = trackers.get(id)
            .ok_or_else(|| format!("Tracker with id '{}' not found", id))?;

        tracker.start().await
    }

    /// Stop a tracker
    pub async fn stop_tracker(&self, id: &str) -> Result<(), String> {
        let trackers = self.trackers.read().await;
        let tracker = trackers.get(id)
            .ok_or_else(|| format!("Tracker with id '{}' not found", id))?;

        tracker.stop().await
    }

    /// Get tracker state
    pub async fn get_tracker_state(&self, id: &str) -> Result<GhatiTrackerState, String> {
        let trackers = self.trackers.read().await;
        let tracker = trackers.get(id)
            .ok_or_else(|| format!("Tracker with id '{}' not found", id))?;

        Ok(tracker.get_state().await)
    }

    /// Subscribe to tracker events
    pub async fn subscribe_to_tracker(&self, id: &str) -> Result<broadcast::Receiver<GhatiTrackingEvent>, String> {
        let trackers = self.trackers.read().await;
        let tracker = trackers.get(id)
            .ok_or_else(|| format!("Tracker with id '{}' not found", id))?;

        Ok(tracker.subscribe())
    }

    /// Remove a tracker
    pub async fn remove_tracker(&self, id: &str) -> Result<(), String> {
        let mut trackers = self.trackers.write().await;
        if let Some(tracker) = trackers.remove(id) {
            let _ = tracker.stop().await; // Stop the tracker
        }
        Ok(())
    }

    /// List all trackers
    pub async fn list_trackers(&self) -> Vec<String> {
        let trackers = self.trackers.read().await;
        trackers.keys().cloned().collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::time::sleep;
    use std::time::Duration as StdDuration;

    #[tokio::test]
    async fn test_ghati_tracker_creation() {
        let config = GhatiTrackerConfig::default();
        let location = Coordinates {
            latitude: 12.9629,
            longitude: 77.5775,
            altitude: Some(920.0),
        };

        let tracker = GhatiRealtimeTracker::new(config, location);
        let state = tracker.get_state().await;
        
        assert!(!state.is_running);
        assert!(state.location.is_some());
    }

    #[tokio::test]
    async fn test_ghati_tracker_start_stop() {
        let config = GhatiTrackerConfig {
            update_interval_ms: 100, // Fast updates for testing
            ..Default::default()
        };
        let location = Coordinates {
            latitude: 12.9629,
            longitude: 77.5775,
            altitude: Some(920.0),
        };

        let tracker = GhatiRealtimeTracker::new(config, location);
        
        // Start tracker
        tracker.start().await.unwrap();
        
        // Wait a bit for updates
        sleep(StdDuration::from_millis(500)).await;
        
        let state = tracker.get_state().await;
        assert!(state.is_running);
        assert!(state.total_updates > 0);
        
        // Stop tracker
        tracker.stop().await.unwrap();
        
        let state = tracker.get_state().await;
        assert!(!state.is_running);
    }

    #[tokio::test]
    async fn test_ghati_tracking_service() {
        let service = GhatiTrackingService::new();
        let config = GhatiTrackerConfig::default();
        let location = Coordinates {
            latitude: 12.9629,
            longitude: 77.5775,
            altitude: Some(920.0),
        };

        // Create tracker
        service.create_tracker("test_tracker".to_string(), config, location).await.unwrap();
        
        // Start tracker
        service.start_tracker("test_tracker").await.unwrap();
        
        // Wait a bit
        sleep(StdDuration::from_millis(500)).await;
        
        // Get state
        let state = service.get_tracker_state("test_tracker").await.unwrap();
        assert!(state.is_running);
        
        // Stop tracker
        service.stop_tracker("test_tracker").await.unwrap();
        
        // Remove tracker
        service.remove_tracker("test_tracker").await.unwrap();
    }
}
