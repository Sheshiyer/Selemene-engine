//! Engine Picker — Scrollable list of 16 engines with descriptions

use crate::app::{Action, ActiveScreen};
use crossterm::event::{KeyCode, KeyEvent};
use noesis_sdk::{LocalProfile, NoesisClient};
use ratatui::prelude::*;
use ratatui::widgets::*;

/// Engine metadata for display
struct EngineEntry {
    id: &'static str,
    name: &'static str,
    icon: &'static str,
    description: &'static str,
    category: &'static str,
}

const ENGINE_LIST: &[EngineEntry] = &[
    EngineEntry { id: "panchanga", name: "Panchanga", icon: "🗓", description: "Vedic calendar — tithi, nakshatra, yoga, karana", category: "Vedic" },
    EngineEntry { id: "numerology", name: "Numerology", icon: "🔢", description: "Pythagorean + Chaldean number systems", category: "Numbers" },
    EngineEntry { id: "biorhythm", name: "Biorhythm", icon: "📊", description: "Physical, Emotional, Intellectual cycles", category: "Cycles" },
    EngineEntry { id: "human-design", name: "Human Design", icon: "🧬", description: "Bodygraph — 64 gates, 9 centers, profile", category: "Design" },
    EngineEntry { id: "gene-keys", name: "Gene Keys", icon: "🧪", description: "Shadow → Gift → Siddhi activation sequences", category: "Keys" },
    EngineEntry { id: "vimshottari", name: "Vimshottari", icon: "⏳", description: "120-year nested dasha period analysis", category: "Vedic" },
    EngineEntry { id: "vedic-clock", name: "Vedic Clock", icon: "🕐", description: "TCM organ clock + Ayurvedic timing", category: "Timing" },
    EngineEntry { id: "biofield", name: "Biofield", icon: "🌀", description: "Chakra & biofield analysis from birth data", category: "Energy" },
    EngineEntry { id: "face-reading", name: "Face Reading", icon: "👁", description: "Physiognomy analysis", category: "Body" },
    EngineEntry { id: "nadabrahman", name: "Nada Brahman", icon: "🔔", description: "Sound consciousness engine", category: "Sound" },
    EngineEntry { id: "transits", name: "Transits", icon: "🪐", description: "Planetary transits, aspects & Sade Sati", category: "Vedic" },
    EngineEntry { id: "tarot", name: "Tarot", icon: "🃏", description: "Major & minor arcana readings", category: "Divination" },
    EngineEntry { id: "i-ching", name: "I Ching", icon: "☯", description: "Book of Changes — hexagram casting", category: "Divination" },
    EngineEntry { id: "enneagram", name: "Enneagram", icon: "🔮", description: "9-type personality system", category: "Personality" },
    EngineEntry { id: "sacred-geometry", name: "Sacred Geometry", icon: "📐", description: "Geometric consciousness patterns", category: "Geometry" },
    EngineEntry { id: "sigil-forge", name: "Sigil Forge", icon: "✡", description: "Consciousness sigil generation", category: "Creation" },
];

pub struct EnginePicker {
    pub selected: usize,
    pub loading: bool,
    pub filter: String,
}

impl EnginePicker {
    pub fn new() -> Self {
        Self {
            selected: 0,
            loading: false,
            filter: String::new(),
        }
    }

    fn filtered_engines(&self) -> Vec<(usize, &EngineEntry)> {
        if self.filter.is_empty() {
            ENGINE_LIST.iter().enumerate().collect()
        } else {
            let q = self.filter.to_lowercase();
            ENGINE_LIST
                .iter()
                .enumerate()
                .filter(|(_, e)| {
                    e.name.to_lowercase().contains(&q)
                        || e.category.to_lowercase().contains(&q)
                        || e.description.to_lowercase().contains(&q)
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
                Constraint::Min(8),   // List
                Constraint::Length(3), // Footer
            ])
            .split(area);

        // Title
        let title = Paragraph::new(Span::styled(
            "⚡ Select Engine",
            Style::default()
                .fg(Color::Cyan)
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
        let filter = Paragraph::new(format!("🔍 {}_", self.filter))
            .style(Style::default().fg(Color::Yellow))
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .title(" Filter ")
                    .border_style(Style::default().fg(Color::DarkGray)),
            );
        frame.render_widget(filter, chunks[1]);

        // Engine list
        let engines = self.filtered_engines();
        let items: Vec<ListItem> = engines
            .iter()
            .enumerate()
            .map(|(display_idx, (_, entry))| {
                let is_selected = display_idx == self.selected;
                let style = if is_selected {
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(Color::White)
                };

                let prefix = if is_selected { "▸ " } else { "  " };
                let content = Line::from(vec![
                    Span::styled(
                        format!("{}{} {}", prefix, entry.icon, entry.name),
                        style,
                    ),
                    Span::styled(
                        format!("  [{}]", entry.category),
                        Style::default().fg(Color::DarkGray),
                    ),
                    Span::styled(
                        format!("  {}", entry.description),
                        Style::default().fg(Color::DarkGray),
                    ),
                ]);

                ListItem::new(content)
            })
            .collect();

        let status = if self.loading {
            "⏳ Calculating..."
        } else {
            &format!("{} engines", engines.len())
        };

        let list = List::new(items).block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .title(format!(" Engines — {} ", status))
                .title_style(Style::default().fg(Color::Cyan))
                .border_style(Style::default().fg(Color::DarkGray)),
        );
        frame.render_widget(list, chunks[2]);

        // Footer
        let footer = Paragraph::new(Line::from(vec![
            Span::styled(" ↑↓ ", Style::default().fg(Color::Cyan)),
            Span::raw("Navigate  "),
            Span::styled(" Enter ", Style::default().fg(Color::Green)),
            Span::raw("Calculate  "),
            Span::styled(" / ", Style::default().fg(Color::Yellow)),
            Span::raw("Filter  "),
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
            KeyCode::Esc => {
                if !self.filter.is_empty() {
                    self.filter.clear();
                    self.selected = 0;
                    Action::None
                } else {
                    Action::Navigate(ActiveScreen::Welcome)
                }
            }
            KeyCode::Up | KeyCode::Char('k') => {
                let count = self.filtered_engines().len();
                if count > 0 && self.selected > 0 {
                    self.selected -= 1;
                }
                Action::None
            }
            KeyCode::Down | KeyCode::Char('j') => {
                let count = self.filtered_engines().len();
                if count > 0 && self.selected < count - 1 {
                    self.selected += 1;
                }
                Action::None
            }
            KeyCode::Enter => {
                let engines = self.filtered_engines();
                if let Some((_, entry)) = engines.get(self.selected) {
                    if let (Some(client), Some(profile)) = (client, profile) {
                        let engine_id = entry.id.to_string();
                        let input = profile.to_engine_input();
                        self.loading = true;

                        match client.calculate(&engine_id, input).await {
                            Ok(output) => {
                                self.loading = false;
                                return Action::ShowEngineResult {
                                    engine_id,
                                    output,
                                };
                            }
                            Err(e) => {
                                self.loading = false;
                                tracing::error!("Engine calculation failed: {}", e);
                                // Stay on screen — error will be shown via app.show_error
                            }
                        }
                    }
                }
                Action::None
            }
            KeyCode::Backspace => {
                self.filter.pop();
                self.selected = 0;
                Action::None
            }
            KeyCode::Char(c) => {
                // Don't filter on vim keys when not in filter mode
                if c == '/' {
                    // Already in filter mode implicitly
                    Action::None
                } else if !self.filter.is_empty() || !matches!(c, 'j' | 'k') {
                    self.filter.push(c);
                    self.selected = 0;
                    Action::None
                } else {
                    Action::None
                }
            }
            _ => Action::None,
        }
    }
}
