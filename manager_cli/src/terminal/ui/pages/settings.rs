use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use crate::terminal::ui::app::AppState;

/// 설정 페이지 렌더링
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
        " Settings ",
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

    // 컨텐츠
    let content = Paragraph::new(vec![
        Line::from(Span::styled(
            "애플리케이션 설정",
            Style::default().fg(Color::Green).add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from("• Redis 호스트: localhost"),
        Line::from("• Redis 포트: 6379"),
        Line::from("• 자동 새로고침: 활성화"),
        Line::from(""),
        Line::from("구현 예정..."),
    ])
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Green)),
    );
    f.render_widget(content, chunks[1]);

    // 푸터
    let footer = Paragraph::new(vec![Line::from(vec![
        Span::styled("ESC", Style::default().fg(Color::Yellow)),
        Span::raw(" 뒤로"),
    ])])
    .alignment(Alignment::Center)
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::White)),
    );
    f.render_widget(footer, chunks[2]);
}
