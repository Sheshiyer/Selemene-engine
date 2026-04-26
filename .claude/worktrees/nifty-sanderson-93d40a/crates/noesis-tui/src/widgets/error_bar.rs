//! Error Bar Widget — Transient error display at bottom of screen

use ratatui::prelude::*;
use ratatui::widgets::*;

/// Draw a red error bar at the bottom of the screen.
pub fn draw_error_bar(frame: &mut Frame, area: Rect, message: &str) {
    let bar_area = Rect {
        x: area.x,
        y: area.y + area.height.saturating_sub(2),
        width: area.width,
        height: 1,
    };

    let bar = Paragraph::new(Span::styled(
        format!(" ⚠ {} ", message),
        Style::default()
            .fg(Color::White)
            .bg(Color::Red)
            .add_modifier(Modifier::BOLD),
    ));

    frame.render_widget(bar, bar_area);
}
