use anyhow::Result;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyModifiers};
use std::time::Duration;

/// 사용자 입력 이벤트 타입
#[derive(Debug, Clone, PartialEq)]
pub enum InputEvent {
    /// 문자 입력
    Char(char),
    /// Enter 키
    Enter,
    /// Backspace 키
    Backspace,
    /// Tab 키
    Tab,
    /// Escape 키
    Escape,
    /// 위 화살표
    Up,
    /// 아래 화살표
    Down,
    /// 왼쪽 화살표
    Left,
    /// 오른쪽 화살표
    Right,
    /// 페이지 업
    PageUp,
    /// 페이지 다운
    PageDown,
    /// Home 키
    Home,
    /// End 키
    End,
    /// Delete 키
    Delete,
    /// Ctrl+C (종료)
    CtrlC,
    /// Ctrl+Q (종료)
    CtrlQ,
    /// 알 수 없는 이벤트
    Unknown,
}

/// 입력 핸들러
pub struct InputHandler;

impl InputHandler {
    /// 새로운 InputHandler 생성
    pub fn new() -> Self {
        Self
    }

    /// 키 이벤트를 폴링하여 InputEvent로 변환
    /// 
    /// # Arguments
    /// * `timeout` - 대기 시간 (밀리초)
    /// 
    /// # Returns
    /// * `Ok(Some(InputEvent))` - 이벤트 발생
    /// * `Ok(None)` - 타임아웃
    /// * `Err` - 에러 발생
    pub fn poll_event(&self, timeout: Duration) -> Result<Option<InputEvent>> {
        if event::poll(timeout)? {
            if let Event::Key(key_event) = event::read()? {
                Ok(Some(Self::convert_key_event(key_event)))
            } else {
                Ok(None)
            }
        } else {
            Ok(None)
        }
    }

    /// KeyEvent를 InputEvent로 변환
    fn convert_key_event(key_event: KeyEvent) -> InputEvent {
        match key_event {
            KeyEvent {
                code: KeyCode::Char('c'),
                modifiers: KeyModifiers::CONTROL,
                ..
            } => InputEvent::CtrlC,
            KeyEvent {
                code: KeyCode::Char('q'),
                modifiers: KeyModifiers::CONTROL,
                ..
            } => InputEvent::CtrlQ,
            KeyEvent {
                code: KeyCode::Char(c),
                ..
            } => InputEvent::Char(c),
            KeyEvent {
                code: KeyCode::Enter,
                ..
            } => InputEvent::Enter,
            KeyEvent {
                code: KeyCode::Backspace,
                ..
            } => InputEvent::Backspace,
            KeyEvent {
                code: KeyCode::Tab,
                ..
            } => InputEvent::Tab,
            KeyEvent {
                code: KeyCode::Esc,
                ..
            } => InputEvent::Escape,
            KeyEvent {
                code: KeyCode::Up,
                ..
            } => InputEvent::Up,
            KeyEvent {
                code: KeyCode::Down,
                ..
            } => InputEvent::Down,
            KeyEvent {
                code: KeyCode::Left,
                ..
            } => InputEvent::Left,
            KeyEvent {
                code: KeyCode::Right,
                ..
            } => InputEvent::Right,
            KeyEvent {
                code: KeyCode::PageUp,
                ..
            } => InputEvent::PageUp,
            KeyEvent {
                code: KeyCode::PageDown,
                ..
            } => InputEvent::PageDown,
            KeyEvent {
                code: KeyCode::Home,
                ..
            } => InputEvent::Home,
            KeyEvent {
                code: KeyCode::End,
                ..
            } => InputEvent::End,
            KeyEvent {
                code: KeyCode::Delete,
                ..
            } => InputEvent::Delete,
            _ => InputEvent::Unknown,
        }
    }
}

impl Default for InputHandler {
    fn default() -> Self {
        Self::new()
    }
}
