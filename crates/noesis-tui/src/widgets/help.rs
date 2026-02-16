//! Help Overlay — Keyboard shortcuts and usage guide

use ratatui::prelude::*;
use ratatui::widgets::*;

pub struct HelpOverlay;

impl HelpOverlay {
    pub fn new() -> Self {
        Self
    }

    pub fn draw(&self, frame: &mut Frame, area: Rect) {
        // Semi-transparent background via a clear block
        let block = Block::default()
            .style(Style::default().bg(Color::Black));
        frame.render_widget(block, area);

        // Center the help panel
        let help_area = centered_rect(60, 70, area);

        let help_lines = vec![
            Line::from(Span::styled(
                "✦ Noesis Engine — Keyboard Shortcuts",
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            )),
            Line::from(""),
            section_header("Global"),
            shortcut("Ctrl+Q / Ctrl+C", "Quit application"),
            shortcut("?", "Toggle this help"),
            shortcut("Esc", "Go back / Close"),
            Line::from(""),
            section_header("Welcome Screen"),
            shortcut("e", "Open engine picker"),
            shortcut("w", "Open workflow picker"),
            shortcut("h", "Open history"),
            shortcut("p", "Open profile editor"),
            Line::from(""),
            section_header("Engine / Workflow Picker"),
            shortcut("↑↓ / k/j", "Navigate list"),
            shortcut("Enter", "Calculate / Execute"),
            shortcut("Type text", "Filter engines"),
            shortcut("Esc", "Clear filter / Go back"),
            Line::from(""),
            section_header("Result Display"),
            shortcut("↑↓ / k/j", "Scroll up/down"),
            shortcut("PgUp/PgDn", "Page up/down"),
            shortcut("e", "Export as Markdown"),
            shortcut("Shift+J", "Export as JSON"),
            Line::from(""),
            section_header("Profile Editor"),
            shortcut("↑↓", "Select field"),
            shortcut("Enter", "Edit field"),
            shortcut("Esc", "Cancel edit"),
            Line::from(""),
            section_header("History"),
            shortcut("r", "Refresh readings"),
            Line::from(""),
            Line::from(Span::styled(
                "Press ? or Esc to close",
                Style::default().fg(Color::DarkGray),
            )),
        ];

        let help = Paragraph::new(help_lines)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Double)
                    .title(" Help ")
                    .title_style(Style::default().fg(Color::Cyan))
                    .border_style(Style::default().fg(Color::Cyan))
                    .style(Style::default().bg(Color::Black)),
            )
            .wrap(Wrap { trim: false });

        frame.render_widget(help, help_area);
    }
}

fn section_header(label: &str) -> Line<'_> {
    Line::from(Span::styled(
        format!("── {} ──", label),
        Style::default()
            .fg(Color::Magenta)
            .add_modifier(Modifier::BOLD),
    ))
}

fn shortcut<'a>(key: &'a str, desc: &'a str) -> Line<'a> {
    Line::from(vec![
        Span::styled(
            format!("  {:<16}", key),
            Style::default().fg(Color::Green),
        ),
        Span::raw(desc),
    ])
}

/// Create a centered rectangle
fn centered_rect(percent_x: u16, percent_y: u16, area: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(area);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}
