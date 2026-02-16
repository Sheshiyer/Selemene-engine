//! History Browser — Browse past readings from the API

use crate::app::{Action, ActiveScreen};
use crate::utils::format_name;
use crossterm::event::{KeyCode, KeyEvent};
use noesis_sdk::{NoesisClient, client::ReadingRecord};
use ratatui::prelude::*;
use ratatui::widgets::*;

pub struct HistoryBrowser {
    pub entries: Vec<ReadingRecord>,
    pub selected: usize,
    pub loading: bool,
    pub loaded: bool,
    pub error: Option<String>,
}

impl HistoryBrowser {
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
            selected: 0,
            loading: false,
            loaded: false,
            error: None,
        }
    }

    pub fn draw(&self, frame: &mut Frame, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Title
                Constraint::Min(8),   // List
                Constraint::Length(3), // Footer
            ])
            .split(area);

        // Title
        let title = Paragraph::new(Span::styled(
            "📜 Reading History",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        ))
        .alignment(Alignment::Center)
        .block(
            Block::default()
                .borders(Borders::BOTTOM)
                .border_style(Style::default().fg(Color::DarkGray)),
        );
        frame.render_widget(title, chunks[0]);

        // Content
        if self.loading {
            let loading = Paragraph::new(Span::styled(
                "⏳ Loading readings...",
                Style::default().fg(Color::DarkGray),
            ))
            .alignment(Alignment::Center)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .border_style(Style::default().fg(Color::DarkGray)),
            );
            frame.render_widget(loading, chunks[1]);
        } else if let Some(ref err) = self.error {
            let error_text = Paragraph::new(vec![
                Line::from(Span::styled(
                    "⚠ Failed to load readings",
                    Style::default().fg(Color::Red),
                )),
                Line::from(""),
                Line::from(Span::styled(
                    err.as_str(),
                    Style::default().fg(Color::DarkGray),
                )),
                Line::from(""),
                Line::from(Span::raw("Press 'r' to retry")),
            ])
            .alignment(Alignment::Center)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .border_style(Style::default().fg(Color::Red)),
            );
            frame.render_widget(error_text, chunks[1]);
        } else if self.entries.is_empty() {
            let empty = Paragraph::new(Span::styled(
                "No readings yet. Run an engine to create one.",
                Style::default().fg(Color::DarkGray),
            ))
            .alignment(Alignment::Center)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .border_style(Style::default().fg(Color::DarkGray)),
            );
            frame.render_widget(empty, chunks[1]);
        } else {
            let items: Vec<ListItem> = self
                .entries
                .iter()
                .enumerate()
                .map(|(i, entry)| {
                    let is_selected = i == self.selected;
                    let style = if is_selected {
                        Style::default()
                            .fg(Color::Yellow)
                            .add_modifier(Modifier::BOLD)
                    } else {
                        Style::default().fg(Color::White)
                    };

                    let prefix = if is_selected { "▸ " } else { "  " };
                    let level_str = format!("  Phase {}", entry.consciousness_level);
                    let time_str = entry.created_at.format("%Y-%m-%d %H:%M").to_string();

                    let content = Line::from(vec![
                        Span::styled(
                            format!("{}{}", prefix, format_name(&entry.engine_id)),
                            style,
                        ),
                        Span::styled(
                            format!("  {}", time_str),
                            Style::default().fg(Color::DarkGray),
                        ),
                        Span::styled(level_str, Style::default().fg(Color::Green)),
                    ]);

                    ListItem::new(content)
                })
                .collect();

            let list = List::new(items).block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .title(format!(" {} readings ", self.entries.len()))
                    .title_style(Style::default().fg(Color::Yellow))
                    .border_style(Style::default().fg(Color::DarkGray)),
            );
            frame.render_widget(list, chunks[1]);
        }

        // Footer
        let footer = Paragraph::new(Line::from(vec![
            Span::styled(" ↑↓ ", Style::default().fg(Color::Yellow)),
            Span::raw("Navigate  "),
            Span::styled(" Enter ", Style::default().fg(Color::Cyan)),
            Span::raw("View  "),
            Span::styled(" r ", Style::default().fg(Color::Green)),
            Span::raw("Refresh  "),
            Span::styled(" Esc ", Style::default().fg(Color::Red)),
            Span::raw("Back"),
        ]))
        .alignment(Alignment::Center)
        .block(
            Block::default()
                .borders(Borders::TOP)
                .border_style(Style::default().fg(Color::DarkGray)),
        );
        frame.render_widget(footer, chunks[2]);
    }

    pub async fn handle_key(
        &mut self,
        key: KeyEvent,
        client: &Option<NoesisClient>,
    ) -> Action {
        // Auto-load on first visit
        if !self.loaded && !self.loading {
            self.fetch_readings(client).await;
        }

        match key.code {
            KeyCode::Esc => Action::Navigate(ActiveScreen::Welcome),
            KeyCode::Up | KeyCode::Char('k') => {
                if !self.entries.is_empty() && self.selected > 0 {
                    self.selected -= 1;
                }
                Action::None
            }
            KeyCode::Down | KeyCode::Char('j') => {
                if !self.entries.is_empty() && self.selected < self.entries.len() - 1 {
                    self.selected += 1;
                }
                Action::None
            }
            KeyCode::Enter => {
                if let Some(entry) = self.entries.get(self.selected) {
                    // Reconstruct EngineOutput from stored result_data
                    match serde_json::from_value::<noesis_sdk::EngineOutput>(entry.result_data.clone()) {
                        Ok(output) => Action::ShowEngineResult {
                            engine_id: entry.engine_id.clone(),
                            output,
                        },
                        Err(_) => {
                            // If deserialization fails, build a minimal output
                            let output = noesis_sdk::EngineOutput {
                                engine_id: entry.engine_id.clone(),
                                result: entry.result_data.clone(),
                                witness_prompt: entry.witness_prompt.clone().unwrap_or_default(),
                                consciousness_level: entry.consciousness_level as u8,
                                metadata: noesis_sdk::CalculationMetadata {
                                    calculation_time_ms: entry.calculation_time_ms.unwrap_or(0.0),
                                    backend: "stored".into(),
                                    precision_achieved: "Standard".into(),
                                    cached: false,
                                    timestamp: entry.created_at,
                                    engine_version: String::new(),
                                },
                            };
                            Action::ShowEngineResult {
                                engine_id: entry.engine_id.clone(),
                                output,
                            }
                        }
                    }
                } else {
                    Action::None
                }
            }
            KeyCode::Char('r') => {
                self.fetch_readings(client).await;
                Action::None
            }
            _ => Action::None,
        }
    }

    async fn fetch_readings(&mut self, client: &Option<NoesisClient>) {
        let Some(client) = client else {
            self.error = Some("No API client configured. Add an API key first.".into());
            self.loaded = true;
            return;
        };

        self.loading = true;
        self.error = None;

        match client.list_readings(Some(50)).await {
            Ok(readings) => {
                self.entries = readings;
                self.selected = 0;
            }
            Err(e) => {
                self.error = Some(format!("{}", e));
            }
        }

        self.loading = false;
        self.loaded = true;
    }
}


