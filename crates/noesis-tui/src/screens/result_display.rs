//! Result Display — Styled engine output with witness prompt, consciousness level, export

use crate::app::{Action, ActiveScreen};
use crate::utils::format_name;
use crossterm::event::{KeyCode, KeyEvent};
use noesis_sdk::{consciousness, EngineOutput, MarkdownRenderer, ReportFormat, WorkflowResult};
use ratatui::prelude::*;
use ratatui::widgets::*;

/// What kind of result we're displaying
enum ResultKind {
    Engine {
        engine_id: String,
        output: EngineOutput,
    },
    Workflow {
        workflow_id: String,
        result: WorkflowResult,
    },
    None,
}

pub struct ResultDisplay {
    result: ResultKind,
    scroll_offset: u16,
    rendered_text: String,
    pub export_message: Option<String>,
}

impl ResultDisplay {
    pub fn new() -> Self {
        Self {
            result: ResultKind::None,
            scroll_offset: 0,
            rendered_text: String::new(),
            export_message: None,
        }
    }

    pub fn set_engine_result(&mut self, engine_id: String, output: EngineOutput) {
        let renderer = MarkdownRenderer::new();
        self.rendered_text = renderer.render_engine_output(&output);
        self.result = ResultKind::Engine { engine_id, output };
        self.scroll_offset = 0;
        self.export_message = None;
    }

    pub fn set_workflow_result(&mut self, workflow_id: String, result: WorkflowResult) {
        let renderer = MarkdownRenderer::new();
        self.rendered_text = renderer.render_workflow_result(&result);
        self.result = ResultKind::Workflow {
            workflow_id,
            result,
        };
        self.scroll_offset = 0;
        self.export_message = None;
    }

    pub fn draw(&self, frame: &mut Frame, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Title bar
                Constraint::Min(8),   // Content
                Constraint::Length(3), // Footer
            ])
            .split(area);

        // Title bar
        // Render with consciousness level name
        let title_text = match &self.result {
            ResultKind::Engine { engine_id, output } => {
                let level = output.consciousness_level;
                let level_info = consciousness::get_level(level);
                format!(
                    "✦ {} — {} {} {}",
                    format_name(engine_id),
                    level_info.state,
                    level_info.dots,
                    format!("(Phase {}/5)", level),
                )
            }
            ResultKind::Workflow {
                workflow_id,
                result,
            } => {
                format!(
                    "🌊 {} — {:.0}ms",
                    format_name(workflow_id),
                    result.total_time_ms
                )
            }
            ResultKind::None => "No result".to_string(),
        };

        let title = Paragraph::new(Span::styled(
            title_text,
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

        // Content — render styled text
        let content_lines = self.style_markdown_lines();
        let total_lines = content_lines.len();
        let visible_line = (self.scroll_offset as usize).saturating_add(1).min(total_lines);
        let scroll_indicator = format!(" Line {}/{} ", visible_line, total_lines);
        let content = Paragraph::new(content_lines)
            .scroll((self.scroll_offset, 0))
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .title_bottom(Line::from(scroll_indicator).alignment(Alignment::Right))
                    .border_style(Style::default().fg(Color::DarkGray)),
            )
            .wrap(Wrap { trim: false });
        frame.render_widget(content, chunks[1]);

        // Footer
        let mut footer_spans = vec![
            Span::styled(" ↑↓ ", Style::default().fg(Color::Cyan)),
            Span::raw("Scroll  "),
            Span::styled(" e ", Style::default().fg(Color::Green)),
            Span::raw("Export MD  "),
            Span::styled(" J ", Style::default().fg(Color::Green)),
            Span::raw("Export JSON  "),
            Span::styled(" r ", Style::default().fg(Color::Yellow)),
            Span::raw("Run Again  "),
            Span::styled(" Esc ", Style::default().fg(Color::Red)),
            Span::raw("Back"),
        ];

        if let Some(ref msg) = self.export_message {
            footer_spans.push(Span::styled(
                format!("  ✓ {}", msg),
                Style::default().fg(Color::Green),
            ));
        }

        let footer = Paragraph::new(Line::from(footer_spans))
            .alignment(Alignment::Center)
            .block(
                Block::default()
                    .borders(Borders::TOP)
                    .border_style(Style::default().fg(Color::DarkGray)),
            );
        frame.render_widget(footer, chunks[2]);
    }

    /// Convert rendered Markdown to styled Lines for ratatui.
    fn style_markdown_lines(&self) -> Vec<Line<'_>> {
        self.rendered_text
            .lines()
            .map(|line| {
                if line.starts_with("# ") {
                    Line::from(Span::styled(
                        &line[2..],
                        Style::default()
                            .fg(Color::Cyan)
                            .add_modifier(Modifier::BOLD),
                    ))
                } else if line.starts_with("## ✦") {
                    Line::from(Span::styled(
                        line,
                        Style::default()
                            .fg(Color::Yellow)
                            .add_modifier(Modifier::BOLD),
                    ))
                } else if line.starts_with("## ") {
                    Line::from(Span::styled(
                        &line[3..],
                        Style::default()
                            .fg(Color::Magenta)
                            .add_modifier(Modifier::BOLD),
                    ))
                } else if line.starts_with("> ") {
                    Line::from(Span::styled(
                        &line[2..],
                        Style::default()
                            .fg(Color::Yellow)
                            .add_modifier(Modifier::ITALIC),
                    ))
                } else if line.starts_with("- **") {
                    // Key-value line
                    if let Some(colon_pos) = line.find("**: ") {
                        let key = &line[4..colon_pos];
                        let value = &line[colon_pos + 4..];
                        Line::from(vec![
                            Span::styled("  ", Style::default()),
                            Span::styled(
                                format!("{}:", key),
                                Style::default()
                                    .fg(Color::White)
                                    .add_modifier(Modifier::BOLD),
                            ),
                            Span::raw(format!(" {}", value)),
                        ])
                    } else {
                        Line::from(Span::raw(line))
                    }
                } else if line.starts_with("---") {
                    Line::from(Span::styled(
                        "────────────────────────────",
                        Style::default().fg(Color::DarkGray),
                    ))
                } else if line.starts_with("_") && line.ends_with("_") {
                    Line::from(Span::styled(
                        &line[1..line.len() - 1],
                        Style::default()
                            .fg(Color::DarkGray)
                            .add_modifier(Modifier::ITALIC),
                    ))
                } else if line.starts_with("Phase: **") {
                    Line::from(Span::styled(
                        line,
                        Style::default().fg(Color::Green),
                    ))
                } else {
                    Line::from(Span::raw(line))
                }
            })
            .collect()
    }

    pub fn handle_key(&mut self, key: KeyEvent) -> Action {
        match key.code {
            KeyCode::Esc => Action::Navigate(ActiveScreen::Welcome),
            KeyCode::Up | KeyCode::Char('k') => {
                self.scroll_offset = self.scroll_offset.saturating_sub(1);
                Action::None
            }
            KeyCode::Down | KeyCode::Char('j') => {
                self.scroll_offset = self.scroll_offset.saturating_add(1);
                Action::None
            }
            KeyCode::PageUp => {
                self.scroll_offset = self.scroll_offset.saturating_sub(10);
                Action::None
            }
            KeyCode::PageDown => {
                self.scroll_offset = self.scroll_offset.saturating_add(10);
                Action::None
            }
            KeyCode::Char('e') => {
                self.export(ReportFormat::Markdown);
                Action::None
            }
            KeyCode::Char('J') => {
                self.export(ReportFormat::Json);
                Action::None
            }
            KeyCode::Char('r') => match &self.result {
                ResultKind::Engine { .. } => Action::Navigate(ActiveScreen::EnginePicker),
                ResultKind::Workflow { .. } => Action::Navigate(ActiveScreen::WorkflowPicker),
                ResultKind::None => Action::None,
            },
            _ => Action::None,
        }
    }

    fn export(&mut self, format: ReportFormat) {
        let renderer = MarkdownRenderer::new();

        let (content, filename_hint) = match &self.result {
            ResultKind::Engine { engine_id, output } => {
                let content = match format {
                    ReportFormat::Markdown => renderer.render_engine_output(output),
                    ReportFormat::Json => {
                        serde_json::to_string_pretty(output).unwrap_or_default()
                    }
                    ReportFormat::Text => renderer.render_engine_output(output),
                };
                (content, engine_id.clone())
            }
            ResultKind::Workflow {
                workflow_id,
                result,
            } => {
                let content = match format {
                    ReportFormat::Markdown => renderer.render_workflow_result(result),
                    ReportFormat::Json => {
                        serde_json::to_string_pretty(result).unwrap_or_default()
                    }
                    ReportFormat::Text => renderer.render_workflow_result(result),
                };
                (content, workflow_id.clone())
            }
            ResultKind::None => return,
        };

        let ext = match format {
            ReportFormat::Markdown => "md",
            ReportFormat::Json => "json",
            ReportFormat::Text => "txt",
        };

        let dir = match noesis_sdk::render::reports_dir() {
            Ok(d) => d,
            Err(e) => {
                self.export_message = Some(format!("Failed to get reports dir: {}", e));
                return;
            }
        };

        let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S");
        let filename = format!("{}_{}.{}", filename_hint, timestamp, ext);
        let path = dir.join(&filename);

        match std::fs::write(&path, content) {
            Ok(()) => {
                self.export_message = Some(format!("Saved to {}", filename));
            }
            Err(e) => {
                self.export_message = Some(format!("Export failed: {}", e));
            }
        }
    }
}
