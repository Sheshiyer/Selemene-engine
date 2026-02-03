//! Hora alarm scheduler

use std::collections::HashMap;
use chrono::{NaiveDateTime, NaiveDate, Duration};
use super::{HoraAlarm, HoraNotification, PlanetaryHora};

/// Hora alarm scheduler
pub struct HoraAlarmScheduler {
    alarms: HashMap<String, HoraAlarm>,
    pending_notifications: Vec<HoraNotification>,
}

impl HoraAlarmScheduler {
    /// Create a new scheduler
    pub fn new() -> Self {
        Self {
            alarms: HashMap::new(),
            pending_notifications: Vec::new(),
        }
    }
    
    /// Add an alarm
    pub fn add_alarm(&mut self, alarm: HoraAlarm) {
        self.alarms.insert(alarm.id.clone(), alarm);
    }
    
    /// Remove an alarm
    pub fn remove_alarm(&mut self, alarm_id: &str) -> Option<HoraAlarm> {
        self.alarms.remove(alarm_id)
    }
    
    /// Enable/disable an alarm
    pub fn set_alarm_enabled(&mut self, alarm_id: &str, enabled: bool) {
        if let Some(alarm) = self.alarms.get_mut(alarm_id) {
            alarm.enabled = enabled;
        }
    }
    
    /// Schedule notifications for a day's horas
    pub fn schedule_for_day(&mut self, date: NaiveDate, horas: &[PlanetaryHora]) {
        self.pending_notifications.clear();
        
        for alarm in self.alarms.values() {
            if !alarm.enabled {
                continue;
            }
            
            for hora in horas {
                if hora.ruler.eq_ignore_ascii_case(&alarm.planet) {
                    // Create notification at hora start
                    if alarm.notify_start {
                        let notification_time = NaiveDateTime::new(date, hora.start_time);
                        self.pending_notifications.push(HoraNotification {
                            hora: hora.clone(),
                            notification_time,
                            message: alarm.message.clone().unwrap_or_else(|| {
                                format!("{} hora starting now", hora.ruler)
                            }),
                            delivered: false,
                        });
                    }
                    
                    // Create advance notification
                    if let Some(minutes) = alarm.notify_before_minutes {
                        let notification_time = NaiveDateTime::new(date, hora.start_time)
                            - Duration::minutes(minutes as i64);
                        self.pending_notifications.push(HoraNotification {
                            hora: hora.clone(),
                            notification_time,
                            message: alarm.message.clone().unwrap_or_else(|| {
                                format!("{} hora in {} minutes", hora.ruler, minutes)
                            }),
                            delivered: false,
                        });
                    }
                }
            }
        }
        
        // Sort by notification time
        self.pending_notifications.sort_by(|a, b| 
            a.notification_time.cmp(&b.notification_time)
        );
    }
    
    /// Get pending notifications
    pub fn get_pending(&self, current_time: NaiveDateTime) -> Vec<&HoraNotification> {
        self.pending_notifications.iter()
            .filter(|n| !n.delivered && n.notification_time <= current_time)
            .collect()
    }
    
    /// Mark notification as delivered
    pub fn mark_delivered(&mut self, index: usize) {
        if let Some(notification) = self.pending_notifications.get_mut(index) {
            notification.delivered = true;
        }
    }
    
    /// Get all alarms
    pub fn get_alarms(&self) -> Vec<&HoraAlarm> {
        self.alarms.values().collect()
    }
}

impl Default for HoraAlarmScheduler {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scheduler() {
        let mut scheduler = HoraAlarmScheduler::new();
        
        let alarm = HoraAlarm {
            id: "jupiter_alarm".to_string(),
            planet: "Jupiter".to_string(),
            notify_start: true,
            notify_before_minutes: Some(5),
            message: None,
            enabled: true,
        };
        
        scheduler.add_alarm(alarm);
        assert_eq!(scheduler.get_alarms().len(), 1);
    }
}
