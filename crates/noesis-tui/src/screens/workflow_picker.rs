//! Workflow Picker — Scrollable list of 6 workflows

use crate::app::{Action, ActiveScreen};
use crossterm::event::{KeyCode, KeyEvent};
use noesis_sdk::{LocalProfile, NoesisClient};
use ratatui::prelude::*;
use ratatui::widgets::*;

struct WorkflowEntry {
    id: &'static str,
    name: &'static str,
    icon: &'static str,
    description: &'static str,
    engines: &'static str,
}

const WORKFLOW_LIST: &[WorkflowEntry] = &[
    WorkflowEntry {
        id: "daily-practice",
        name: "Daily Practice",
        icon: "🌅",
        description: "Today's energetic weather — transits, biorhythm, vedic clock",
        engines: "panchanga, biorhythm, vedic-clock, transits",
    },
    WorkflowEntry {
        id: "birth-chart-analysis",
        name: "Birth Chart Analysis",
        icon: "🌟",
        description: "Complete natal analysis across multiple systems",
        engines: "panchanga, numerology, human-design, gene-keys, vimshottari",
    },
    WorkflowEntry {
        id: "transit-forecast",
        name: "Transit Forecast",
        icon: "🪐",
        description: "Current and upcoming planetary influences",
        engines: "transits, panchanga, vimshottari",
    },
    WorkflowEntry {
        id: "consciousness-calibration",
        name: "Consciousness Calibration",
        icon: "🧭",
        description: "Multi-engine consciousness level assessment",
        engines: "biofield, nadabrahman, gene-keys, sacred-geometry",
    },
    WorkflowEntry {
        id: "somatic-inquiry",
        name: "Somatic Inquiry",
        icon: "🫀",
        description: "Body-centered awareness and energy mapping",
        engines: "biofield, biorhythm, vedic-clock, face-reading",
    },
    WorkflowEntry {
        id: "shadow-work",
        name: "Shadow Work",
        icon: "🌑",
        description: "Shadow-Gift-Siddhi exploration with tarot integration",
        engines: "gene-keys, tarot, enneagram, sigil-forge",
    },
];

pub struct WorkflowPicker {
    pub selected: usize,
    pub loading: bool,
}

impl WorkflowPicker {
    pub fn new() -> Self {
        Self {
            selected: 0,
            loading: false,
        }
    }

    pub fn draw(&self, frame: &mut Frame, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Title
                Constraint::Min(8),   // List
                Constraint::Length(5), // Detail panel
                Constraint::Length(3), // Footer
            ])
            .split(area);

        // Title
        let title = Paragraph::new(Span::styled(
            "🌊 Select Workflow",
            Style::default()
                .fg(Color::Magenta)
                .add_modifier(Modifier::BOLD),
        ))
        .alignment(Alignment::Center)
        .block(
            Block::default()
                .borders(Borders::BOTTOM)
                .border_style(Style::default().fg(Color::DarkGray)),
        );
        frame.render_widget(title, chunks[0]);

        // Workflow list
        let items: Vec<ListItem> = WORKFLOW_LIST
            .iter()
            .enumerate()
            .map(|(i, entry)| {
                let is_selected = i == self.selected;
                let style = if is_selected {
                    Style::default()
                        .fg(Color::Magenta)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(Color::White)
                };

                let prefix = if is_selected { "▸ " } else { "  " };
                let content = Line::from(vec![
                    Span::styled(format!("{}{} {}", prefix, entry.icon, entry.name), style),
                    Span::styled(
                        format!("  {}", entry.description),
                        Style::default().fg(Color::DarkGray),
                    ),
                ]);

                ListItem::new(content)
            })
            .collect();

        let status = if self.loading {
            "⏳ Executing..."
        } else {
            "6 workflows"
        };

        let list = List::new(items).block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .title(format!(" Workflows — {} ", status))
                .title_style(Style::default().fg(Color::Magenta))
                .border_style(Style::default().fg(Color::DarkGray)),
        );
        frame.render_widget(list, chunks[1]);

        // Detail panel — show engines for selected workflow
        if let Some(entry) = WORKFLOW_LIST.get(self.selected) {
            let detail = Paragraph::new(vec![
                Line::from(vec![
                    Span::styled("Engines: ", Style::default().fg(Color::DarkGray)),
                    Span::styled(entry.engines, Style::default().fg(Color::Yellow)),
                ]),
            ])
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .title(" Details ")
                    .title_style(Style::default().fg(Color::DarkGray))
                    .border_style(Style::default().fg(Color::DarkGray)),
            );
            frame.render_widget(detail, chunks[2]);
        }

        // Footer
        let footer = Paragraph::new(Line::from(vec![
            Span::styled(" ↑↓ ", Style::default().fg(Color::Magenta)),
            Span::raw("Navigate  "),
            Span::styled(" Enter ", Style::default().fg(Color::Green)),
            Span::raw("Execute  "),
            Span::styled(" Esc ", Style::default().fg(Color::Red)),
            Span::raw("Back"),
        ]))
        .alignment(Alignment::Center)
        .block(
            Block::default()
                .borders(Borders::TOP)
                .border_style(Style::default().fg(Color::DarkGray)),
        );
        frame.render_widget(footer, chunks[3]);
    }

    pub async fn handle_key(
        &mut self,
        key: KeyEvent,
        client: &Option<NoesisClient>,
        profile: &Option<LocalProfile>,
    ) -> Action {
        match key.code {
            KeyCode::Esc => Action::Navigate(ActiveScreen::Welcome),
            KeyCode::Up | KeyCode::Char('k') => {
                if self.selected > 0 {
                    self.selected -= 1;
                }
                Action::None
            }
            KeyCode::Down | KeyCode::Char('j') => {
                if self.selected < WORKFLOW_LIST.len() - 1 {
                    self.selected += 1;
                }
                Action::None
            }
            KeyCode::Enter => {
                if let Some(entry) = WORKFLOW_LIST.get(self.selected) {
                    if let (Some(client), Some(profile)) = (client, profile) {
                        let workflow_id = entry.id.to_string();
                        let input = profile.to_engine_input();
                        self.loading = true;

                        match client.workflow(&workflow_id, input).await {
                            Ok(result) => {
                                self.loading = false;
                                return Action::ShowWorkflowResult {
                                    workflow_id,
                                    result,
                                };
                            }
                            Err(e) => {
                                self.loading = false;
                                tracing::error!("Workflow execution failed: {}", e);
                            }
                        }
                    }
                }
                Action::None
            }
            _ => Action::None,
        }
    }
}
