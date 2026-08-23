use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph, Wrap},
    Frame,
};

use crate::model::NoteType;

use super::app::App;

pub fn render(f: &mut Frame, app: &mut App) {
    let size = f.area();

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(0),
            Constraint::Length(3),
        ])
        .split(size);

    let title = Paragraph::new("Sticky Notes Manager")
        .style(
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )
        .alignment(Alignment::Center)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(
                    Style::default().fg(Color::Yellow),
                ),
        );

    f.render_widget(title, chunks[0]);

    let items: Vec<ListItem> = app
        .notes
        .iter()
        .map(|note| {
            let type_str = match note.note_type {
                NoteType::Text => "TEXT",
                NoteType::List => "LIST",
                NoteType::Markdown => "MD ",
            };

            let type_color = match note.note_type {
                NoteType::Text => Color::Green,
                NoteType::List => Color::Blue,
                NoteType::Markdown => Color::Magenta,
            };

            ListItem::new(Line::from(vec![
                Span::styled(
                    format!("[{}] ", type_str),
                    Style::default()
                        .fg(type_color)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    &note.title,
                    Style::default().fg(Color::White),
                ),
                Span::styled(
                    format!(
                        " ({})",
                        note.id.chars().take(8).collect::<String>()
                    ),
                    Style::default().fg(Color::DarkGray),
                ),
            ]))
        })
        .collect();

    let notes_list = List::new(items)
        .block(
            Block::default()
                .title(format!(" Notes ({}) ", app.notes.len()))
                .borders(Borders::ALL)
                .border_style(
                    Style::default().fg(Color::Cyan),
                ),
        )
        .highlight_style(
            Style::default()
                .bg(Color::Yellow)
                .fg(Color::Black)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("► ");

    f.render_stateful_widget(
        notes_list,
        chunks[1],
        &mut app.list_state,
    );

    let help = Paragraph::new(
        Line::from(vec![
            Span::styled(
                " n ",
                Style::default()
                    .fg(Color::Yellow)
                    .bg(Color::Black),
            ),
            Span::raw(" New Text  "),
            Span::styled(
                " N ",
                Style::default()
                    .fg(Color::Yellow)
                    .bg(Color::Black),
            ),
            Span::raw(" New List  "),
            Span::styled(
                " m ",
                Style::default()
                    .fg(Color::Yellow)
                    .bg(Color::Black),
            ),
            Span::raw(" New MD  "),
            Span::styled(
                " o/Enter ",
                Style::default()
                    .fg(Color::Yellow)
                    .bg(Color::Black),
            ),
            Span::raw(" Open  "),
            Span::styled(
                " d ",
                Style::default()
                    .fg(Color::Yellow)
                    .bg(Color::Black),
            ),
            Span::raw(" Delete  "),
            Span::styled(
                " r ",
                Style::default()
                    .fg(Color::Yellow)
                    .bg(Color::Black),
            ),
            Span::raw(" Refresh  "),
            Span::styled(
                " q/Esc ",
                Style::default()
                    .fg(Color::Yellow)
                    .bg(Color::Black),
            ),
            Span::raw(" Quit  "),
            Span::styled(
                " ↑/↓ or j/k ",
                Style::default()
                    .fg(Color::Yellow)
                    .bg(Color::Black),
            ),
            Span::raw(" Navigate"),
        ]),
    )
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(
                Style::default().fg(Color::DarkGray),
            ),
    )
    .alignment(Alignment::Center)
    .wrap(Wrap { trim: true });

    f.render_widget(help, chunks[2]);

    if let Some(msg) = &app.status_message {
        let status_area = Rect {
            x: chunks[1].x + 2,
            y: chunks[1]
                .y
                .saturating_add(chunks[1].height.saturating_sub(3)),
            width: chunks[1]
                .width
                .saturating_sub(4)
                .min(msg.len() as u16 + 4),
            height: 3,
        };

        let status = Paragraph::new(msg.as_str())
            .style(
                Style::default()
                    .fg(Color::Black)
                    .bg(Color::Yellow),
            )
            .block(
                Block::default()
                    .borders(Borders::ALL),
            );

        f.render_widget(status, status_area);
    }
}