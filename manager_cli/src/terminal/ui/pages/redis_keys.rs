use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame,
};

use crate::terminal::ui::app::AppState;

/// Redis 키 목록 페이지 렌더링
pub fn render(f: &mut Frame, app: &AppState, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(0),
            Constraint::Length(3),
        ])
        .split(area);

    // 헤더
    let header = Paragraph::new(vec![Line::from(vec![Span::styled(
        " Redis Keys ",
        Style::default()
            .fg(Color::Yellow)
            .add_modifier(Modifier::BOLD),
    )])])
    .alignment(Alignment::Center)
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan)),
    );
    f.render_widget(header, chunks[0]);

    // 키 목록
    render_keys_list(f, app, chunks[1]);

    // 푸터
    let footer = Paragraph::new(vec![Line::from(vec![
        Span::styled("ESC", Style::default().fg(Color::Yellow)),
        Span::raw(" 뒤로 | "),
        Span::styled("r", Style::default().fg(Color::Yellow)),
        Span::raw(" 새로고침"),
    ])])
    .alignment(Alignment::Center)
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::White)),
    );
    f.render_widget(footer, chunks[2]);
}

fn render_keys_list(f: &mut Frame, app: &AppState, area: Rect) {
    if app.redis_keys.is_empty() {
        let empty_msg = Paragraph::new(vec![Line::from(Span::styled(
            "키가 없습니다.",
            Style::default().fg(Color::Gray),
        ))])
        .alignment(Alignment::Center)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Green)),
        );
        f.render_widget(empty_msg, area);
    } else {
        let items: Vec<ListItem> = app
            .redis_keys
            .iter()
            .map(|key| {
                ListItem::new(Line::from(Span::styled(
                    format!("• {}", key),
                    Style::default().fg(Color::White),
                )))
            })
            .collect();

        let list = List::new(items).block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Green))
                .title(format!(" 총 {} 개의 키 ", app.redis_keys.len())),
        );

        f.render_widget(list, area);
    }
}
