use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph, Wrap},
    Frame,
};

use crate::terminal::ui::app::AppState;

/// 랜딩 페이지 렌더링
pub fn render(f: &mut Frame, app: &AppState, area: Rect) {
    // 메인 레이아웃: 헤더, 컨텐츠, 푸터
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(0),
            Constraint::Length(3),
        ])
        .split(area);

    // 헤더
    render_header(f, chunks[0]);

    // 컨텐츠 영역
    render_content(f, app, chunks[1]);

    // 푸터
    render_footer(f, app, chunks[2]);
}

/// 헤더 렌더링
fn render_header(f: &mut Frame, area: Rect) {
    let title = Paragraph::new(vec![Line::from(vec![
        Span::styled(
            "███ Rin Manager CLI ███",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ),
    ])])
    .alignment(Alignment::Center)
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan)),
    );
    f.render_widget(title, area);
}

/// 컨텐츠 렌더링
fn render_content(f: &mut Frame, app: &AppState, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(30), Constraint::Percentage(70)])
        .split(area);

    // 왼쪽: 메뉴 리스트
    render_menu(f, app, chunks[0]);

    // 오른쪽: 정보 패널
    render_info_panel(f, app, chunks[1]);
}

/// 메뉴 리스트 렌더링
fn render_menu(f: &mut Frame, app: &AppState, area: Rect) {
    let menu_items: Vec<ListItem> = app
        .menu_items
        .iter()
        .enumerate()
        .map(|(i, item)| {
            let style = if i == app.selected_menu {
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::White)
            };

            let content = if i == app.selected_menu {
                format!(">> {}", item)
            } else {
                format!("   {}", item)
            };

            ListItem::new(Line::from(Span::styled(content, style)))
        })
        .collect();

    let menu = List::new(menu_items).block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Green))
            .title(" Menu "),
    );

    f.render_widget(menu, area);
}

/// 정보 패널 렌더링
fn render_info_panel(f: &mut Frame, app: &AppState, area: Rect) {
    let binding = String::new();
    let selected_item = app.menu_items.get(app.selected_menu).unwrap_or(&binding);

    let info_text = match selected_item.as_str() {
        "Redis Connection" => vec![
            Line::from(Span::styled(
                "Redis 연결 관리",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            )),
            Line::from(""),
            Line::from("Redis 서버에 연결하고 연결 상태를 확인합니다."),
            Line::from(""),
            Line::from(Span::styled(
                "상태:",
                Style::default().add_modifier(Modifier::BOLD),
            )),
            Line::from(if app.redis_connected {
                Span::styled("✓ 연결됨", Style::default().fg(Color::Green))
            } else {
                Span::styled("✗ 연결 안됨", Style::default().fg(Color::Red))
            }),
        ],
        "Redis Keys" => vec![
            Line::from(Span::styled(
                "Redis 키 목록",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            )),
            Line::from(""),
            Line::from("Redis에 저장된 모든 키를 조회합니다."),
            Line::from(""),
            Line::from(Span::styled(
                "현재 키 갯수:",
                Style::default().add_modifier(Modifier::BOLD),
            )),
            Line::from(format!("{}", app.redis_keys.len())),
        ],
        "Redis Pubsub" => vec![
            Line::from(Span::styled(
                "Redis Pub/Sub",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            )),
            Line::from(""),
            Line::from("Redis Pub/Sub 채널을 관리하고 메시지를 모니터링합니다."),
            Line::from(""),
            Line::from("구독 중인 채널: 0"),
        ],
        "Settings" => vec![
            Line::from(Span::styled(
                "설정",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            )),
            Line::from(""),
            Line::from("애플리케이션 설정을 변경합니다."),
        ],
        "Exit" => vec![
            Line::from(Span::styled(
                "종료",
                Style::default()
                    .fg(Color::Red)
                    .add_modifier(Modifier::BOLD),
            )),
            Line::from(""),
            Line::from("애플리케이션을 종료합니다."),
        ],
        _ => vec![Line::from("정보 없음")],
    };

    let info = Paragraph::new(info_text)
        .wrap(Wrap { trim: true })
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Magenta))
                .title(" Info "),
        );

    f.render_widget(info, area);
}

/// 푸터 렌더링
fn render_footer(f: &mut Frame, app: &AppState, area: Rect) {
    let footer_text = vec![Line::from(vec![
        Span::styled("↑↓", Style::default().fg(Color::Yellow)),
        Span::raw(" 네비게이션 | "),
        Span::styled("Enter", Style::default().fg(Color::Yellow)),
        Span::raw(" 선택 | "),
        Span::styled("q/Ctrl+C", Style::default().fg(Color::Yellow)),
        Span::raw(" 종료 | "),
        Span::styled(&app.status_message, Style::default().fg(Color::Cyan)),
    ])];

    let footer = Paragraph::new(footer_text)
        .alignment(Alignment::Center)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::White)),
        );

    f.render_widget(footer, area);
}
