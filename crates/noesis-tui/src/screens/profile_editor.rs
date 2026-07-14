//! Profile Editor — View and edit birth data and preferences

use crate::app::{Action, ActiveScreen};
use crossterm::event::{KeyCode, KeyEvent};
use noesis_sdk::{consciousness, Config, LocalProfile};
use ratatui::prelude::*;
use ratatui::widgets::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ProfileField {
    Name,
    Date,
    Time,
    Latitude,
    Longitude,
    Timezone,
    ConsciousnessLevel,
    ApiKey,
}

impl ProfileField {
    fn all() -> &'static [ProfileField] {
        &[
            Self::Name,
            Self::Date,
            Self::Time,
            Self::Latitude,
            Self::Longitude,
            Self::Timezone,
            Self::ConsciousnessLevel,
            Self::ApiKey,
        ]
    }

    fn label(self) -> &'static str {
        match self {
            Self::Name => "Name",
            Self::Date => "Birth Date",
            Self::Time => "Birth Time",
            Self::Latitude => "Latitude",
            Self::Longitude => "Longitude",
            Self::Timezone => "Timezone",
            Self::ConsciousnessLevel => "Awareness",
            Self::ApiKey => "API Key",
        }
    }
}

pub struct ProfileEditor {
    selected_field: usize,
    editing: bool,
    edit_buffer: String,
    message: Option<String>,
}

impl ProfileEditor {
    pub fn new() -> Self {
        Self {
            selected_field: 0,
            editing: false,
            edit_buffer: String::new(),
            message: None,
        }
    }

    pub fn draw(
        &self,
        frame: &mut Frame,
        area: Rect,
        profile: &Option<LocalProfile>,
        config: &Config,
    ) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Title
                Constraint::Min(12),   // Fields
                Constraint::Length(3), // Message
                Constraint::Length(3), // Footer
            ])
            .split(area);

        // Title
        let title = Paragraph::new(Span::styled(
            "👤 Profile Editor",
            Style::default()
                .fg(Color::Green)
                .add_modifier(Modifier::BOLD),
        ))
        .alignment(Alignment::Center)
        .block(
            Block::default()
                .borders(Borders::BOTTOM)
                .border_style(Style::default().fg(Color::DarkGray)),
        );
        frame.render_widget(title, chunks[0]);

        // Fields
        if let Some(ref p) = profile {
            let fields = ProfileField::all();
            let items: Vec<ListItem> = fields
                .iter()
                .enumerate()
                .map(|(i, field)| {
                    let is_selected = i == self.selected_field;
                    let is_editing = is_selected && self.editing;

                    let value = if is_editing {
                        format!("{}_", self.edit_buffer)
                    } else if *field == ProfileField::ApiKey {
                        // API key lives in ~/.noesis/config.toml
                        if config.api_key.is_some() {
                            "••••••••".to_string()
                        } else {
                            "Not set".to_string()
                        }
                    } else if *field == ProfileField::ConsciousnessLevel {
                        consciousness::level_display(p.consciousness_level)
                    } else {
                        self.get_field_value(p, *field)
                    };

                    let prefix = if is_selected { "▸ " } else { "  " };
                    let label_style = if is_selected {
                        Style::default()
                            .fg(Color::Green)
                            .add_modifier(Modifier::BOLD)
                    } else {
                        Style::default().fg(Color::DarkGray)
                    };

                    let value_style = if is_editing {
                        Style::default().fg(Color::Cyan)
                    } else {
                        Style::default().fg(Color::White)
                    };

                    let content = Line::from(vec![
                        Span::styled(prefix, label_style),
                        Span::styled(format!("{:<12}", field.label()), label_style),
                        Span::styled(value, value_style),
                    ]);

                    ListItem::new(content)
                })
                .collect();

            let list = List::new(items).block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .title(" Birth Data ")
                    .title_style(Style::default().fg(Color::Green))
                    .border_style(Style::default().fg(Color::DarkGray)),
            );
            frame.render_widget(list, chunks[1]);
        } else {
            let no_profile = Paragraph::new(Span::styled(
                "No profile found. Complete onboarding first.",
                Style::default().fg(Color::DarkGray),
            ))
            .alignment(Alignment::Center)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .border_style(Style::default().fg(Color::DarkGray)),
            );
            frame.render_widget(no_profile, chunks[1]);
        }

        // Message
        if let Some(ref msg) = self.message {
            let style = if msg.starts_with("✓") {
                Style::default().fg(Color::Green)
            } else {
                Style::default().fg(Color::Red)
            };
            let message =
                Paragraph::new(Span::styled(msg.as_str(), style)).alignment(Alignment::Center);
            frame.render_widget(message, chunks[2]);
        }

        // Footer
        let footer_text = if self.editing {
            vec![
                Span::styled(" Enter ", Style::default().fg(Color::Green)),
                Span::raw("Save  "),
                Span::styled(" Esc ", Style::default().fg(Color::Red)),
                Span::raw("Cancel"),
            ]
        } else {
            vec![
                Span::styled(" ↑↓ ", Style::default().fg(Color::Green)),
                Span::raw("Navigate  "),
                Span::styled(" Enter ", Style::default().fg(Color::Green)),
                Span::raw("Edit  "),
                Span::styled(" Esc ", Style::default().fg(Color::Red)),
                Span::raw("Back"),
            ]
        };

        let footer = Paragraph::new(Line::from(footer_text))
            .alignment(Alignment::Center)
            .block(
                Block::default()
                    .borders(Borders::TOP)
                    .border_style(Style::default().fg(Color::DarkGray)),
            );
        frame.render_widget(footer, chunks[3]);
    }

    fn get_field_value(&self, profile: &LocalProfile, field: ProfileField) -> String {
        match field {
            ProfileField::Name => profile.name.clone(),
            ProfileField::Date => profile.birth_data.date.clone(),
            ProfileField::Time => profile
                .birth_data
                .time
                .clone()
                .unwrap_or_else(|| "Not set".into()),
            ProfileField::Latitude => format!("{:.4}", profile.birth_data.latitude),
            ProfileField::Longitude => format!("{:.4}", profile.birth_data.longitude),
            ProfileField::Timezone => profile.birth_data.timezone.clone(),
            ProfileField::ConsciousnessLevel => {
                consciousness::level_display(profile.consciousness_level)
            }
            ProfileField::ApiKey => {
                unreachable!("ApiKey field value is read from config, not LocalProfile")
            }
        }
    }

    pub fn handle_key(
        &mut self,
        key: KeyEvent,
        profile: &mut Option<LocalProfile>,
        config: &mut Config,
    ) -> Action {
        if self.editing {
            return self.handle_edit_key(key, profile, config);
        }

        match key.code {
            KeyCode::Esc => Action::Navigate(ActiveScreen::Welcome),
            KeyCode::Up | KeyCode::Char('k') => {
                if self.selected_field > 0 {
                    self.selected_field -= 1;
                }
                self.message = None;
                Action::None
            }
            KeyCode::Down | KeyCode::Char('j') => {
                if self.selected_field < ProfileField::all().len() - 1 {
                    self.selected_field += 1;
                }
                self.message = None;
                Action::None
            }
            KeyCode::Enter => {
                if let Some(ref mut p) = profile {
                    let field = ProfileField::all()[self.selected_field];
                    if field == ProfileField::ConsciousnessLevel {
                        // Cycle consciousness level 0→1→2→3→4→5→0
                        p.consciousness_level = (p.consciousness_level + 1) % 6;
                        if let Err(e) = p.save() {
                            self.message = Some(format!("⚠ Save failed: {}", e));
                        } else {
                            let info = consciousness::get_level(p.consciousness_level);
                            self.message =
                                Some(format!("✓ Set to {} — {}", info.state, info.description));
                        }
                        return Action::None;
                    }
                    if field == ProfileField::ApiKey {
                        self.edit_buffer = String::new();
                    } else {
                        self.edit_buffer = self.get_field_value(p, field);
                        if self.edit_buffer == "Not set" {
                            self.edit_buffer.clear();
                        }
                    }
                    self.editing = true;
                    self.message = None;
                }
                Action::None
            }
            _ => Action::None,
        }
    }

    fn handle_edit_key(
        &mut self,
        key: KeyEvent,
        profile: &mut Option<LocalProfile>,
        config: &mut Config,
    ) -> Action {
        match key.code {
            KeyCode::Esc => {
                self.editing = false;
                self.edit_buffer.clear();
                Action::None
            }
            KeyCode::Enter => {
                self.editing = false;
                let field = ProfileField::all()[self.selected_field];

                // API key is stored in keychain, not in LocalProfile
                if field == ProfileField::ApiKey {
                    let value = self.edit_buffer.trim().to_string();
                    if value.is_empty() {
                        self.message = Some("⚠ API key cannot be empty".into());
                    } else {
                        // Store in ~/.noesis/config.toml (industry standard)
                        config.api_key = Some(value);
                        match config.save() {
                            Ok(()) => {
                                self.message =
                                    Some("✓ API Key saved to ~/.noesis/config.toml".into());
                                self.edit_buffer.clear();
                                return Action::ReloadConfig;
                            }
                            Err(e) => {
                                self.message = Some(format!("⚠ Save error: {}", e));
                            }
                        }
                    }
                    self.edit_buffer.clear();
                    return Action::None;
                }

                if let Some(ref mut p) = profile {
                    match self.apply_field(p, field) {
                        Ok(()) => {
                            if let Err(e) = p.save() {
                                self.message = Some(format!("⚠ Save failed: {}", e));
                            } else {
                                self.message = Some(format!("✓ {} updated", field.label()));
                            }
                        }
                        Err(e) => {
                            self.message = Some(format!("⚠ {}", e));
                        }
                    }
                }
                self.edit_buffer.clear();
                Action::None
            }
            KeyCode::Backspace => {
                self.edit_buffer.pop();
                Action::None
            }
            KeyCode::Char(c) => {
                self.edit_buffer.push(c);
                Action::None
            }
            _ => Action::None,
        }
    }

    fn apply_field(&self, profile: &mut LocalProfile, field: ProfileField) -> Result<(), String> {
        let value = self.edit_buffer.trim().to_string();
        match field {
            ProfileField::Name => {
                if value.is_empty() {
                    return Err("Name cannot be empty".into());
                }
                profile.name = value;
            }
            ProfileField::Date => {
                if chrono::NaiveDate::parse_from_str(&value, "%Y-%m-%d").is_err() {
                    return Err("Invalid date. Use YYYY-MM-DD".into());
                }
                profile.birth_data.date = value;
            }
            ProfileField::Time => {
                if !value.is_empty() && chrono::NaiveTime::parse_from_str(&value, "%H:%M").is_err()
                {
                    return Err("Invalid time. Use HH:MM".into());
                }
                profile.birth_data.time = if value.is_empty() { None } else { Some(value) };
            }
            ProfileField::Latitude => {
                let lat: f64 = value.parse().map_err(|_| "Invalid number")?;
                if !(-90.0..=90.0).contains(&lat) {
                    return Err("Latitude must be -90 to 90".into());
                }
                profile.birth_data.latitude = lat;
            }
            ProfileField::Longitude => {
                let lng: f64 = value.parse().map_err(|_| "Invalid number")?;
                if !(-180.0..=180.0).contains(&lng) {
                    return Err("Longitude must be -180 to 180".into());
                }
                profile.birth_data.longitude = lng;
            }
            ProfileField::Timezone => {
                if value.is_empty() {
                    return Err("Timezone cannot be empty".into());
                }
                profile.birth_data.timezone = value;
            }
            ProfileField::ConsciousnessLevel => {
                // Handled via Enter cycling in handle_key, not via text edit
                unreachable!("ConsciousnessLevel is cycled via Enter, not apply_field");
            }
            ProfileField::ApiKey => {
                // Handled separately in handle_edit_key
                unreachable!("ApiKey is handled via config, not apply_field");
            }
        }
        Ok(())
    }
}
