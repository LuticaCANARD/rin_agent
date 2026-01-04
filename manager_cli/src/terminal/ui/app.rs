use anyhow::Result;
use std::time::Duration;

/// 애플리케이션 페이지 타입
#[derive(Debug, Clone, PartialEq)]
pub enum Page {
    /// 랜딩 페이지 (메인 메뉴)
    Landing,
    /// Redis 키 목록 페이지
    RedisKeys,
    /// Redis Pub/Sub 페이지
    RedisPubSub,
    /// 설정 페이지
    Settings,
}

/// 애플리케이션 상태
pub struct AppState {
    /// 현재 페이지
    pub current_page: Page,
    /// 종료 플래그
    pub should_quit: bool,
    /// 메뉴 아이템
    pub menu_items: Vec<String>,
    /// 선택된 메뉴 인덱스
    pub selected_menu: usize,
    /// Redis 연결 상태
    pub redis_connected: bool,
    /// Redis 키 목록
    pub redis_keys: Vec<String>,
    /// 상태 메시지
    pub status_message: String,
    /// 입력 버퍼
    pub input_buffer: String,
}

impl AppState {
    /// 새로운 AppState 생성
    pub fn new() -> Self {
        Self {
            current_page: Page::Landing,
            should_quit: false,
            menu_items: vec![
                "Redis Connection".to_string(),
                "Redis Keys".to_string(),
                "Redis Pubsub".to_string(),
                "Settings".to_string(),
                "Exit".to_string(),
            ],
            selected_menu: 0,
            redis_connected: false,
            redis_keys: Vec::new(),
            status_message: "Ready".to_string(),
            input_buffer: String::new(),
        }
    }

    /// 메뉴 위로 이동
    pub fn menu_up(&mut self) {
        if self.selected_menu > 0 {
            self.selected_menu -= 1;
        }
    }

    /// 메뉴 아래로 이동
    pub fn menu_down(&mut self) {
        if self.selected_menu < self.menu_items.len() - 1 {
            self.selected_menu += 1;
        }
    }

    /// 현재 선택된 메뉴 아이템 실행
    pub fn execute_selected_menu(&mut self) {
        let selected = self.menu_items.get(self.selected_menu);
        
        match selected.map(|s| s.as_str()) {
            Some("Redis Connection") => {
                self.status_message = "Connecting to Redis...".to_string();
                self.current_page = Page::Landing;
            }
            Some("Redis Keys") => {
                self.current_page = Page::RedisKeys;
                self.status_message = "Loading keys...".to_string();
            }
            Some("Redis Pubsub") => {
                self.current_page = Page::RedisPubSub;
                self.status_message = "Pub/Sub monitor".to_string();
            }
            Some("Settings") => {
                self.current_page = Page::Settings;
                self.status_message = "Settings".to_string();
            }
            Some("Exit") => {
                self.should_quit = true;
            }
            _ => {}
        }
    }

    /// 이전 페이지로 돌아가기
    pub fn go_back(&mut self) {
        self.current_page = Page::Landing;
        self.status_message = "Back to main menu".to_string();
    }

    /// 입력 버퍼에 문자 추가
    pub fn input_char(&mut self, c: char) {
        self.input_buffer.push(c);
    }

    /// 입력 버퍼에서 문자 삭제
    pub fn input_backspace(&mut self) {
        self.input_buffer.pop();
    }

    /// 입력 버퍼 초기화
    pub fn clear_input(&mut self) {
        self.input_buffer.clear();
    }

    /// 상태 메시지 설정
    pub fn set_status(&mut self, message: String) {
        self.status_message = message;
    }

    /// Redis 연결 상태 업데이트
    pub fn set_redis_connected(&mut self, connected: bool) {
        self.redis_connected = connected;
        self.status_message = if connected {
            "Redis connected".to_string()
        } else {
            "Redis disconnected".to_string()
        };
    }

    /// Redis 키 목록 업데이트
    pub fn set_redis_keys(&mut self, keys: Vec<String>) {
        self.redis_keys = keys;
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}
