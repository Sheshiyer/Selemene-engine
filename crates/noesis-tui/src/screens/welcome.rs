//! Welcome Screen — Landing page with navigation options

use crate::app::{Action, ActiveScreen};
use crossterm::event::{KeyCode, KeyEvent};
use noesis_sdk::consciousness;
use noesis_sdk::LocalProfile;
use ratatui::prelude::*;
use ratatui::widgets::*;

pub struct WelcomeScreen {
    pub selected: usize,
    /// Set to true when an API client is connected. Wire in app.rs.
    pub connected: bool,
}

const MENU_ITEMS: &[(&str, &str)] = &[
    (
        "⚡ Run Engine",
        "Calculate with a single consciousness engine",
    ),
    ("🌊 Run Workflow", "Execute a multi-engine workflow"),
    ("📜 History", "Browse past readings"),
    ("👤 Profile", "Edit your birth data and preferences"),
    ("❓ Help", "Keyboard shortcuts and usage guide"),
];

impl WelcomeScreen {
    pub fn new() -> Self {
        Self {
            selected: 0,
            connected: false,
        }
    }

    pub fn draw(&self, frame: &mut Frame, area: Rect, profile: &Option<LocalProfile>) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(10), // Header (with consciousness level)
                Constraint::Min(10),    // Menu
                Constraint::Length(3),  // Footer
            ])
            .split(area);

        // Header
        self.draw_header(frame, chunks[0], profile);

        // Menu
        self.draw_menu(frame, chunks[1]);

        // Footer
        self.draw_footer(frame, chunks[2]);
    }

    fn draw_header(&self, frame: &mut Frame, area: Rect, profile: &Option<LocalProfile>) {
        let title = vec![
            Line::from(Span::styled(
                "╔═══════════════════════════════════════╗",
                Style::default().fg(Color::Cyan),
            )),
            Line::from(Span::styled(
                "║       ✦  NOESIS ENGINE  ✦            ║",
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            )),
            Line::from(Span::styled(
                "║   Consciousness Calculation Engine    ║",
                Style::default().fg(Color::DarkGray),
            )),
            Line::from(Span::styled(
                "╚═══════════════════════════════════════╝",
                Style::default().fg(Color::Cyan),
            )),
            Line::from(Span::styled(
                format!("v{}", env!("CARGO_PKG_VERSION")),
                Style::default().fg(Color::DarkGray),
            )),
        ];

        let greeting = if let Some(ref p) = profile {
            format!("Welcome back, {}", p.name)
        } else {
            "Welcome, Witness".to_string()
        };

        let mut lines = title;
        lines.push(Line::from(Span::styled(
            greeting,
            Style::default().fg(Color::Yellow),
        )));

        // Consciousness level indicator
        if let Some(ref p) = profile {
            let level_info = consciousness::get_level(p.consciousness_level);
            lines.push(Line::from(vec![
                Span::styled(
                    format!("{} ", level_info.dots),
                    Style::default().fg(Color::Cyan),
                ),
                Span::styled(
                    level_info.state,
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::ITALIC),
                ),
            ]));
        }

        // Connection status indicator
        let (indicator, style) = if self.connected {
            ("● Connected", Style::default().fg(Color::Green))
        } else {
            ("● Offline", Style::default().fg(Color::Red))
        };
        lines.push(Line::from(Span::styled(indicator, style)));

        let header = Paragraph::new(lines).alignment(Alignment::Center);
        frame.render_widget(header, area);
    }

    fn draw_menu(&self, frame: &mut Frame, area: Rect) {
        let items: Vec<ListItem> = MENU_ITEMS
            .iter()
            .enumerate()
            .map(|(i, (label, desc))| {
                let style = if i == self.selected {
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(Color::White)
                };

                let prefix = if i == self.selected { "▸ " } else { "  " };
                let content = Line::from(vec![
                    Span::styled(format!("{}{}", prefix, label), style),
                    Span::styled(
                        format!("  — {}", desc),
                        Style::default().fg(Color::DarkGray),
                    ),
                ]);

                ListItem::new(content)
            })
            .collect();

        let menu = List::new(items)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .title(" Navigate ")
                    .title_style(Style::default().fg(Color::Cyan))
                    .border_style(Style::default().fg(Color::DarkGray)),
            )
            .highlight_style(Style::default().bg(Color::DarkGray));

        frame.render_widget(menu, area);
    }

    fn draw_footer(&self, frame: &mut Frame, area: Rect) {
        let footer = Paragraph::new(Line::from(vec![
            Span::styled(" ↑↓ ", Style::default().fg(Color::Cyan)),
            Span::raw("Navigate  "),
            Span::styled(" Enter ", Style::default().fg(Color::Cyan)),
            Span::raw("Select  "),
            Span::styled(" ? ", Style::default().fg(Color::Cyan)),
            Span::raw("Help  "),
            Span::styled(" Ctrl+Q ", Style::default().fg(Color::Red)),
            Span::raw("Quit"),
        ]))
        .alignment(Alignment::Center)
        .block(
            Block::default()
                .borders(Borders::TOP)
                .border_style(Style::default().fg(Color::DarkGray)),
        );

        frame.render_widget(footer, area);
    }

    pub fn handle_key(&mut self, key: KeyEvent) -> Action {
        match key.code {
            KeyCode::Up | KeyCode::Char('k') => {
                if self.selected > 0 {
                    self.selected -= 1;
                }
                Action::None
            }
            KeyCode::Down | KeyCode::Char('j') => {
                if self.selected < MENU_ITEMS.len() - 1 {
                    self.selected += 1;
                }
                Action::None
            }
            KeyCode::Enter => match self.selected {
                0 => Action::Navigate(ActiveScreen::EnginePicker),
                1 => Action::Navigate(ActiveScreen::WorkflowPicker),
                2 => Action::Navigate(ActiveScreen::History),
                3 => Action::Navigate(ActiveScreen::ProfileEditor),
                4 => Action::ToggleHelp,
                _ => Action::None,
            },
            KeyCode::Char('e') => Action::Navigate(ActiveScreen::EnginePicker),
            KeyCode::Char('w') => Action::Navigate(ActiveScreen::WorkflowPicker),
            KeyCode::Char('h') => Action::Navigate(ActiveScreen::History),
            KeyCode::Char('p') => Action::Navigate(ActiveScreen::ProfileEditor),
            KeyCode::Char('q') => Action::Quit,
            _ => Action::None,
        }
    }
}
