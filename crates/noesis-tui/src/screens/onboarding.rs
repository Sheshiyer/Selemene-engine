//! Onboarding Wizard — Step-by-step profile setup
//!
//! Steps: Name → Birth Date → Birth Time → Location → API Key → Done

use crate::app::{Action, ActiveScreen};
use crossterm::event::{KeyCode, KeyEvent};
use noesis_sdk::consciousness;
use noesis_sdk::{BirthData, LocalProfile};
use ratatui::prelude::*;
use ratatui::widgets::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OnboardingStep {
    Name,
    BirthDate,
    BirthTime,
    Latitude,
    Longitude,
    Timezone,
    ApiKey,
    ConsciousnessLevel,
    Confirm,
}

impl OnboardingStep {
    fn next(self) -> Self {
        match self {
            Self::Name => Self::BirthDate,
            Self::BirthDate => Self::BirthTime,
            Self::BirthTime => Self::Latitude,
            Self::Latitude => Self::Longitude,
            Self::Longitude => Self::Timezone,
            Self::Timezone => Self::ApiKey,
            Self::ApiKey => Self::ConsciousnessLevel,
            Self::ConsciousnessLevel => Self::Confirm,
            Self::Confirm => Self::Confirm,
        }
    }

    fn prev(self) -> Self {
        match self {
            Self::Name => Self::Name,
            Self::BirthDate => Self::Name,
            Self::BirthTime => Self::BirthDate,
            Self::Latitude => Self::BirthTime,
            Self::Longitude => Self::Latitude,
            Self::Timezone => Self::Longitude,
            Self::ApiKey => Self::Timezone,
            Self::ConsciousnessLevel => Self::ApiKey,
            Self::Confirm => Self::ConsciousnessLevel,
        }
    }

    fn index(self) -> usize {
        match self {
            Self::Name => 0,
            Self::BirthDate => 1,
            Self::BirthTime => 2,
            Self::Latitude => 3,
            Self::Longitude => 4,
            Self::Timezone => 5,
            Self::ApiKey => 6,
            Self::ConsciousnessLevel => 7,
            Self::Confirm => 8,
        }
    }

    fn label(self) -> &'static str {
        match self {
            Self::Name => "Your Name",
            Self::BirthDate => "Birth Date (YYYY-MM-DD)",
            Self::BirthTime => "Birth Time (HH:MM, 24h) — optional",
            Self::Latitude => "Birth Latitude (e.g. 28.6139)",
            Self::Longitude => "Birth Longitude (e.g. 77.2090)",
            Self::Timezone => "Timezone (e.g. Asia/Kolkata)",
            Self::ApiKey => "API Key — optional, press Enter to skip",
            Self::ConsciousnessLevel => "Where are you on the awareness journey?",
            Self::Confirm => "Review & Confirm",
        }
    }

    fn hint(self) -> &'static str {
        match self {
            Self::Name => "How would you like to be addressed?",
            Self::BirthDate => "Format: YYYY-MM-DD (e.g. 1990-06-15)",
            Self::BirthTime => "24-hour format. Leave blank if unknown.",
            Self::Latitude => "Decimal degrees. North is positive.",
            Self::Longitude => "Decimal degrees. East is positive.",
            Self::Timezone => "IANA timezone (e.g. America/New_York, UTC)",
            Self::ApiKey => "Get one at https://selemene.tryambakam.space",
            Self::ConsciousnessLevel => "Use ↑↓ to select, Enter to confirm. This isn't gamification — it calibrates your prompts.",
            Self::Confirm => "Press Enter to save, or ← to go back and edit.",
        }
    }
}

const TOTAL_STEPS: usize = 9;

pub struct OnboardingWizard {
    pub step: OnboardingStep,
    pub name_input: String,
    pub date_input: String,
    pub time_input: String,
    pub lat_input: String,
    pub lng_input: String,
    pub tz_input: String,
    pub api_key_input: Option<String>,
    key_buffer: String,
    pub consciousness_level: u8,
    pub error: Option<String>,
}

impl OnboardingWizard {
    pub fn new() -> Self {
        Self {
            step: OnboardingStep::Name,
            name_input: String::new(),
            date_input: String::new(),
            time_input: String::new(),
            lat_input: String::new(),
            lng_input: String::new(),
            tz_input: "UTC".to_string(),
            api_key_input: None,
            key_buffer: String::new(),
            consciousness_level: 0,
            error: None,
        }
    }

    pub fn draw(&self, frame: &mut Frame, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Title
                Constraint::Length(3), // Progress bar
                Constraint::Min(10),   // Content
                Constraint::Length(3), // Footer
            ])
            .split(area);

        // Title
        let title = Paragraph::new(Line::from(vec![
            Span::styled("✦ ", Style::default().fg(Color::Cyan)),
            Span::styled(
                "Noesis Setup",
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ),
        ]))
        .alignment(Alignment::Center)
        .block(
            Block::default()
                .borders(Borders::BOTTOM)
                .border_style(Style::default().fg(Color::DarkGray)),
        );
        frame.render_widget(title, chunks[0]);

        // Progress
        let progress = self.step.index() as f64 / TOTAL_STEPS as f64;
        let gauge = Gauge::default()
            .block(Block::default().title(format!(
                " Step {} of {} ",
                self.step.index() + 1,
                TOTAL_STEPS
            )))
            .gauge_style(Style::default().fg(Color::Cyan).bg(Color::DarkGray))
            .ratio(progress)
            .label(format!("{}%", (progress * 100.0) as u16));
        frame.render_widget(gauge, chunks[1]);

        // Content area
        self.draw_step(frame, chunks[2]);

        // Footer
        let footer_text = if self.step == OnboardingStep::Confirm {
            vec![
                Span::styled(" Enter ", Style::default().fg(Color::Green)),
                Span::raw("Save & continue  "),
                Span::styled(" ← ", Style::default().fg(Color::Yellow)),
                Span::raw("Go back  "),
                Span::styled(" Esc ", Style::default().fg(Color::Red)),
                Span::raw("Skip setup"),
            ]
        } else {
            vec![
                Span::styled(" Enter ", Style::default().fg(Color::Green)),
                Span::raw("Next  "),
                Span::styled(" ← ", Style::default().fg(Color::Yellow)),
                Span::raw("Back  "),
                Span::styled(" Esc ", Style::default().fg(Color::Red)),
                Span::raw("Skip"),
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

    fn draw_step(&self, frame: &mut Frame, area: Rect) {
        let inner = Layout::default()
            .direction(Direction::Vertical)
            .margin(2)
            .constraints([
                Constraint::Length(2), // Label
                Constraint::Length(3), // Input
                Constraint::Length(2), // Hint
                Constraint::Length(2), // Error
                Constraint::Min(0),    // Preview (confirm step) / Level picker
            ])
            .split(area);

        // Label
        let label = Paragraph::new(Span::styled(
            self.step.label(),
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        ));
        frame.render_widget(label, inner[0]);

        if self.step == OnboardingStep::Confirm {
            // Show summary instead of input
            self.draw_confirm(frame, inner[4]);
        } else if self.step == OnboardingStep::ConsciousnessLevel {
            // Show level picker in the larger area
            self.draw_level_picker(frame, inner[1], inner[4]);
        } else {
            // Input field
            let current_value = self.current_input();
            let input = Paragraph::new(format!("{}_", current_value))
                .style(Style::default().fg(Color::Cyan))
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .border_type(BorderType::Rounded)
                        .border_style(Style::default().fg(Color::Cyan)),
                );
            frame.render_widget(input, inner[1]);
        }

        // Hint
        let hint = Paragraph::new(Span::styled(
            self.step.hint(),
            Style::default().fg(Color::DarkGray),
        ));
        frame.render_widget(hint, inner[2]);

        // Error
        if let Some(ref err) = self.error {
            let error = Paragraph::new(Span::styled(
                format!("⚠ {}", err),
                Style::default().fg(Color::Red),
            ));
            frame.render_widget(error, inner[3]);
        }
    }

    fn draw_level_picker(&self, frame: &mut Frame, compact_area: Rect, detail_area: Rect) {
        // Show current selection in the compact input area
        let current = consciousness::get_level(self.consciousness_level);
        let selection_text = format!(
            "  {} {} — {}",
            current.dots, current.state, current.description
        );
        let selection = Paragraph::new(Span::styled(
            selection_text,
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(Color::Cyan)),
        );
        frame.render_widget(selection, compact_area);

        // Show all levels in the detail area
        let items: Vec<ListItem> = consciousness::LEVELS
            .iter()
            .map(|lvl| {
                let is_selected = lvl.level == self.consciousness_level;
                let prefix = if is_selected { "▸ " } else { "  " };
                let style = if is_selected {
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(Color::White)
                };

                let content = vec![
                    Line::from(vec![
                        Span::styled(format!("{}{} {}", prefix, lvl.dots, lvl.state), style),
                        Span::styled(
                            format!("  — {}", lvl.description),
                            Style::default().fg(Color::DarkGray),
                        ),
                    ]),
                    Line::from(Span::styled(
                        format!("    Prompts: {}", lvl.prompt_style),
                        Style::default().fg(if is_selected {
                            Color::Yellow
                        } else {
                            Color::DarkGray
                        }),
                    )),
                ];

                ListItem::new(content)
            })
            .collect();

        let list = List::new(items).block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .title(" Awareness Levels ")
                .title_style(Style::default().fg(Color::Yellow))
                .border_style(Style::default().fg(Color::DarkGray)),
        );
        frame.render_widget(list, detail_area);
    }

    fn draw_confirm(&self, frame: &mut Frame, area: Rect) {
        let lines = vec![
            Line::from(vec![
                Span::styled("  Name:      ", Style::default().fg(Color::DarkGray)),
                Span::styled(&self.name_input, Style::default().fg(Color::White)),
            ]),
            Line::from(vec![
                Span::styled("  Date:      ", Style::default().fg(Color::DarkGray)),
                Span::styled(&self.date_input, Style::default().fg(Color::White)),
            ]),
            Line::from(vec![
                Span::styled("  Time:      ", Style::default().fg(Color::DarkGray)),
                Span::styled(
                    if self.time_input.is_empty() {
                        "Not set"
                    } else {
                        &self.time_input
                    },
                    Style::default().fg(Color::White),
                ),
            ]),
            Line::from(vec![
                Span::styled("  Location:  ", Style::default().fg(Color::DarkGray)),
                Span::styled(
                    format!("{}, {}", self.lat_input, self.lng_input),
                    Style::default().fg(Color::White),
                ),
            ]),
            Line::from(vec![
                Span::styled("  Timezone:  ", Style::default().fg(Color::DarkGray)),
                Span::styled(&self.tz_input, Style::default().fg(Color::White)),
            ]),
            Line::from(vec![
                Span::styled("  API Key:   ", Style::default().fg(Color::DarkGray)),
                Span::styled(
                    if self.key_buffer.is_empty() {
                        "Not set (offline mode)"
                    } else {
                        "••••••••••••"
                    },
                    Style::default().fg(Color::White),
                ),
            ]),
            Line::from(vec![
                Span::styled("  Awareness: ", Style::default().fg(Color::DarkGray)),
                Span::styled(
                    consciousness::level_display(self.consciousness_level),
                    Style::default().fg(Color::Cyan),
                ),
            ]),
        ];

        let summary = Paragraph::new(lines).block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .title(" Profile Summary ")
                .title_style(Style::default().fg(Color::Green))
                .border_style(Style::default().fg(Color::Green)),
        );
        frame.render_widget(summary, area);
    }

    fn current_input(&self) -> &str {
        match self.step {
            OnboardingStep::Name => &self.name_input,
            OnboardingStep::BirthDate => &self.date_input,
            OnboardingStep::BirthTime => &self.time_input,
            OnboardingStep::Latitude => &self.lat_input,
            OnboardingStep::Longitude => &self.lng_input,
            OnboardingStep::Timezone => &self.tz_input,
            OnboardingStep::ApiKey => &self.key_buffer,
            OnboardingStep::ConsciousnessLevel | OnboardingStep::Confirm => "",
        }
    }

    fn current_input_mut(&mut self) -> &mut String {
        match self.step {
            OnboardingStep::Name => &mut self.name_input,
            OnboardingStep::BirthDate => &mut self.date_input,
            OnboardingStep::BirthTime => &mut self.time_input,
            OnboardingStep::Latitude => &mut self.lat_input,
            OnboardingStep::Longitude => &mut self.lng_input,
            OnboardingStep::Timezone => &mut self.tz_input,
            OnboardingStep::ApiKey => &mut self.key_buffer,
            OnboardingStep::ConsciousnessLevel | OnboardingStep::Confirm => unreachable!(),
        }
    }

    /// Validate current step before advancing.
    fn validate_step(&self) -> Result<(), String> {
        match self.step {
            OnboardingStep::Name => {
                if self.name_input.trim().is_empty() {
                    return Err("Name cannot be empty".into());
                }
            }
            OnboardingStep::BirthDate => {
                if chrono::NaiveDate::parse_from_str(&self.date_input, "%Y-%m-%d").is_err() {
                    return Err("Invalid date format. Use YYYY-MM-DD".into());
                }
            }
            OnboardingStep::BirthTime => {
                // Optional — empty is fine
                if !self.time_input.is_empty()
                    && chrono::NaiveTime::parse_from_str(&self.time_input, "%H:%M").is_err()
                {
                    return Err("Invalid time format. Use HH:MM (24h)".into());
                }
            }
            OnboardingStep::Latitude => match self.lat_input.parse::<f64>() {
                Ok(lat) if (-90.0..=90.0).contains(&lat) => {}
                _ => return Err("Latitude must be between -90 and 90".into()),
            },
            OnboardingStep::Longitude => match self.lng_input.parse::<f64>() {
                Ok(lng) if (-180.0..=180.0).contains(&lng) => {}
                _ => return Err("Longitude must be between -180 and 180".into()),
            },
            OnboardingStep::Timezone => {
                if self.tz_input.trim().is_empty() {
                    return Err("Timezone cannot be empty".into());
                }
            }
            OnboardingStep::ApiKey
            | OnboardingStep::ConsciousnessLevel
            | OnboardingStep::Confirm => {}
        }
        Ok(())
    }

    pub fn handle_key(&mut self, key: KeyEvent, config: &mut noesis_sdk::Config) -> Action {
        match key.code {
            KeyCode::Esc => {
                // Skip onboarding
                return Action::Navigate(ActiveScreen::Welcome);
            }
            KeyCode::Enter => {
                if self.step == OnboardingStep::Confirm {
                    // Store API key if provided
                    if !self.key_buffer.is_empty() {
                        self.api_key_input = Some(self.key_buffer.clone());
                        config.api_key = Some(self.key_buffer.clone());
                    }
                    return Action::Navigate(ActiveScreen::Welcome);
                }

                // Validate before advancing
                match self.validate_step() {
                    Ok(()) => {
                        self.error = None;
                        self.step = self.step.next();
                    }
                    Err(e) => {
                        self.error = Some(e);
                    }
                }
            }
            KeyCode::Left | KeyCode::BackTab => {
                self.error = None;
                self.step = self.step.prev();
            }
            KeyCode::Up | KeyCode::Char('k') if self.step == OnboardingStep::ConsciousnessLevel => {
                if self.consciousness_level > 0 {
                    self.consciousness_level -= 1;
                }
            }
            KeyCode::Down | KeyCode::Char('j')
                if self.step == OnboardingStep::ConsciousnessLevel =>
            {
                if self.consciousness_level < 5 {
                    self.consciousness_level += 1;
                }
            }
            KeyCode::Char(c)
                if self.step != OnboardingStep::Confirm
                    && self.step != OnboardingStep::ConsciousnessLevel =>
            {
                self.current_input_mut().push(c);
                self.error = None;
            }
            KeyCode::Backspace
                if self.step != OnboardingStep::Confirm
                    && self.step != OnboardingStep::ConsciousnessLevel =>
            {
                self.current_input_mut().pop();
                self.error = None;
            }
            _ => {}
        }
        Action::None
    }

    /// Build a LocalProfile from wizard inputs.
    pub fn build_profile(&self) -> Option<LocalProfile> {
        let lat: f64 = self.lat_input.parse().ok()?;
        let lng: f64 = self.lng_input.parse().ok()?;

        let birth_data = BirthData {
            name: Some(self.name_input.clone()),
            date: self.date_input.clone(),
            time: if self.time_input.is_empty() {
                None
            } else {
                Some(self.time_input.clone())
            },
            latitude: lat,
            longitude: lng,
            timezone: self.tz_input.clone(),
        };

        let mut profile = LocalProfile::new(&self.name_input, birth_data);
        profile.consciousness_level = self.consciousness_level;
        Some(profile)
    }
}
