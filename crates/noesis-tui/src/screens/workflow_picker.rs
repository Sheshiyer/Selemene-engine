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
        id: "birth-blueprint",
        name: "Birth Blueprint",
        icon: "🌟",
        description: "Core identity mapping — numerology, human design, vimshottari",
        engines: "numerology, human-design, vimshottari",
    },
    WorkflowEntry {
        id: "daily-practice",
        name: "Daily Practice",
        icon: "🌅",
        description: "Daily rhythm optimization — panchanga, vedic clock, biorhythm",
        engines: "panchanga, vedic-clock, biorhythm",
    },
    WorkflowEntry {
        id: "decision-support",
        name: "Decision Support",
        icon: "🧭",
        description: "Multi-perspective decision mirrors — tarot, i-ching, human design",
        engines: "tarot, i-ching, human-design",
    },
    WorkflowEntry {
        id: "self-inquiry",
        name: "Self-Inquiry",
        icon: "🌑",
        description: "Shadow work and type exploration — gene keys, enneagram",
        engines: "gene-keys, enneagram",
    },
    WorkflowEntry {
        id: "creative-expression",
        name: "Creative Expression",
        icon: "✡",
        description: "Generative symbol and geometry exploration — sigil forge, sacred geometry",
        engines: "sigil-forge, sacred-geometry",
    },
    WorkflowEntry {
        id: "full-spectrum",
        name: "Full Spectrum",
        icon: "🔮",
        description: "Complete integration of all consciousness engines",
        engines: "all 16 engines",
    },
];

pub struct WorkflowPicker {
    pub selected: usize,
    pub loading: bool,
    pub filter: String,
    pub filtering: bool,
}

impl WorkflowPicker {
    pub fn new() -> Self {
        Self {
            selected: 0,
            loading: false,
            filter: String::new(),
            filtering: false,
        }
    }

    fn filtered_workflows(&self) -> Vec<(usize, &WorkflowEntry)> {
        if self.filter.is_empty() {
            WORKFLOW_LIST.iter().enumerate().collect()
        } else {
            let q = self.filter.to_lowercase();
            WORKFLOW_LIST
                .iter()
                .enumerate()
                .filter(|(_, e)| {
                    e.name.to_lowercase().contains(&q)
                        || e.description.to_lowercase().contains(&q)
                        || e.engines.to_lowercase().contains(&q)
                })
                .collect()
        }
    }

    pub fn draw(&self, frame: &mut Frame, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Title
                Constraint::Length(3), // Filter
                Constraint::Min(8),    // List
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

        // Filter
        let (filter_text, filter_border_color) = if self.filtering {
            (format!("🔍 {}_", self.filter), Color::Magenta)
        } else if !self.filter.is_empty() {
            (format!("🔍 {}", self.filter), Color::DarkGray)
        } else {
            ("🔍 Press / to filter...".to_string(), Color::DarkGray)
        };
        let filter = Paragraph::new(filter_text)
            .style(Style::default().fg(Color::Yellow))
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .title(" Filter ")
                    .border_style(Style::default().fg(filter_border_color)),
            );
        frame.render_widget(filter, chunks[1]);

        // Workflow list
        let workflows = self.filtered_workflows();
        let items: Vec<ListItem> = workflows
            .iter()
            .enumerate()
            .map(|(display_idx, (_, entry))| {
                let is_selected = display_idx == self.selected;
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
            &format!("{} workflows", workflows.len())
        };

        let list = List::new(items).block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .title(format!(" Workflows — {} ", status))
                .title_style(Style::default().fg(Color::Magenta))
                .border_style(Style::default().fg(Color::DarkGray)),
        );
        frame.render_widget(list, chunks[2]);

        // Detail panel — show engines for selected workflow
        let workflows = self.filtered_workflows();
        if let Some((_, entry)) = workflows.get(self.selected) {
            let detail = Paragraph::new(vec![Line::from(vec![
                Span::styled("Engines: ", Style::default().fg(Color::DarkGray)),
                Span::styled(entry.engines, Style::default().fg(Color::Yellow)),
            ])])
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .title(" Details ")
                    .title_style(Style::default().fg(Color::DarkGray))
                    .border_style(Style::default().fg(Color::DarkGray)),
            );
            frame.render_widget(detail, chunks[3]);
        }

        // Footer
        let footer_spans = if self.filtering {
            vec![
                Span::styled(" Type ", Style::default().fg(Color::Yellow)),
                Span::raw("to filter  "),
                Span::styled(" ↑↓ ", Style::default().fg(Color::Magenta)),
                Span::raw("Navigate  "),
                Span::styled(" Enter ", Style::default().fg(Color::Green)),
                Span::raw("Execute  "),
                Span::styled(" Esc ", Style::default().fg(Color::Red)),
                Span::raw("Cancel"),
            ]
        } else {
            vec![
                Span::styled(" / ", Style::default().fg(Color::Yellow)),
                Span::raw("Filter  "),
                Span::styled(" ↑↓/jk ", Style::default().fg(Color::Magenta)),
                Span::raw("Navigate  "),
                Span::styled(" Enter ", Style::default().fg(Color::Green)),
                Span::raw("Execute  "),
                Span::styled(" Esc ", Style::default().fg(Color::Red)),
                Span::raw("Back"),
            ]
        };
        let footer = Paragraph::new(Line::from(footer_spans))
            .alignment(Alignment::Center)
            .block(
                Block::default()
                    .borders(Borders::TOP)
                    .border_style(Style::default().fg(Color::DarkGray)),
            );
        frame.render_widget(footer, chunks[4]);
    }

    pub async fn handle_key(
        &mut self,
        key: KeyEvent,
        client: &Option<NoesisClient>,
        profile: &Option<LocalProfile>,
    ) -> Action {
        if self.filtering {
            // Filtering mode: all chars go to filter, arrows navigate
            match key.code {
                KeyCode::Esc => {
                    self.filter.clear();
                    self.filtering = false;
                    self.selected = 0;
                    Action::None
                }
                KeyCode::Enter => {
                    let workflows = self.filtered_workflows();
                    if let Some((_, entry)) = workflows.get(self.selected) {
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
                                    return Action::ShowError(format!(
                                        "Workflow execution failed: {}",
                                        e
                                    ));
                                }
                            }
                        } else if profile.is_none() {
                            return Action::ShowError(
                                "No profile configured. Complete onboarding first.".into(),
                            );
                        } else {
                            return Action::ShowError(
                                "No API client. Set an API key first.".into(),
                            );
                        }
                    }
                    Action::None
                }
                KeyCode::Up => {
                    let count = self.filtered_workflows().len();
                    if count > 0 && self.selected > 0 {
                        self.selected -= 1;
                    }
                    Action::None
                }
                KeyCode::Down => {
                    let count = self.filtered_workflows().len();
                    if count > 0 && self.selected < count - 1 {
                        self.selected += 1;
                    }
                    Action::None
                }
                KeyCode::Backspace => {
                    self.filter.pop();
                    self.selected = 0;
                    Action::None
                }
                KeyCode::Char(c) => {
                    self.filter.push(c);
                    self.selected = 0;
                    Action::None
                }
                _ => Action::None,
            }
        } else {
            // Normal mode: j/k navigate, / enters filter mode
            match key.code {
                KeyCode::Esc => Action::Navigate(ActiveScreen::Welcome),
                KeyCode::Up | KeyCode::Char('k') => {
                    let count = self.filtered_workflows().len();
                    if count > 0 && self.selected > 0 {
                        self.selected -= 1;
                    }
                    Action::None
                }
                KeyCode::Down | KeyCode::Char('j') => {
                    let count = self.filtered_workflows().len();
                    if count > 0 && self.selected < count - 1 {
                        self.selected += 1;
                    }
                    Action::None
                }
                KeyCode::Enter => {
                    let workflows = self.filtered_workflows();
                    if let Some((_, entry)) = workflows.get(self.selected) {
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
                                    return Action::ShowError(format!(
                                        "Workflow execution failed: {}",
                                        e
                                    ));
                                }
                            }
                        } else if profile.is_none() {
                            return Action::ShowError(
                                "No profile configured. Complete onboarding first.".into(),
                            );
                        } else {
                            return Action::ShowError(
                                "No API client. Set an API key first.".into(),
                            );
                        }
                    }
                    Action::None
                }
                KeyCode::Char('/') => {
                    self.filtering = true;
                    Action::None
                }
                _ => Action::None,
            }
        }
    }
}
