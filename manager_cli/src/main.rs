use anyhow::Result;
use contract::config::{EnvConfigBuilder, ManagerConfig};
use std::time::Duration;

mod command;
mod terminal;
mod connection;

use terminal::core::{input::InputHandler, terminal::TerminalManager, input::InputEvent};
use terminal::ui::{AppState, layout::landing, pages};
use connection::RedisManager;

#[tokio::main]
async fn main() -> Result<()> {
    // 환경 변수 로드
    let strategy = contract::config::parse_env_strategy_from_args();
    let dotenv_path = contract::config::parse_dotenv_path_from_args();
    
    let mut builder = EnvConfigBuilder::new()
        .strategy(strategy)
        .ignore_missing(true);
    
    if let Some(path) = dotenv_path {
        builder = builder.dotenv_path(path);
    }
    
    let env_ret = builder.load();

    if let Err(e) = env_ret {
        eprintln!("Failed to load environment variables: {}", e);
        std::process::exit(1);
    }
    
    // Redis 설정 가져오기
    let redis_url = std::env::var("REDIS_URL")
        .unwrap_or_else(|_| "redis://127.0.0.1:6379".to_string());

    // Redis Manager 초기화
    let redis_manager = RedisManager::new(&redis_url)?;

    // 애플리케이션 실행
    run_app(redis_manager).await
}

/// 메인 애플리케이션 루프
async fn run_app(redis_manager: RedisManager) -> Result<()> {
    // 터미널 초기화
    let mut terminal_manager = TerminalManager::new()?;
    let input_handler = InputHandler::new();

    // 앱 상태 초기화
    let mut app_state = AppState::new();

    // 메인 이벤트 루프
    loop {
        // UI 렌더링
        terminal_manager.terminal().draw(|f| {
            let area = f.area();
            
            match app_state.current_page {
                terminal::ui::app::Page::Landing => {
                    landing::render(f, &app_state, area);
                }
                terminal::ui::app::Page::RedisKeys => {
                    pages::redis_keys::render(f, &app_state, area);
                }
                terminal::ui::app::Page::RedisPubSub => {
                    pages::redis_pubsub::render(f, &app_state, area);
                }
                terminal::ui::app::Page::Settings => {
                    pages::settings::render(f, &app_state, area);
                }
            }
        })?;

        // 입력 처리
        if let Some(event) = input_handler.poll_event(Duration::from_millis(100))? {
            match event {
                InputEvent::CtrlC | InputEvent::CtrlQ => {
                    app_state.should_quit = true;
                }
                InputEvent::Char('q') if app_state.current_page == terminal::ui::app::Page::Landing => {
                    app_state.should_quit = true;
                }
                InputEvent::Up => {
                    if app_state.current_page == terminal::ui::app::Page::Landing {
                        app_state.menu_up();
                    }
                }
                InputEvent::Down => {
                    if app_state.current_page == terminal::ui::app::Page::Landing {
                        app_state.menu_down();
                    }
                }
                InputEvent::Enter => {
                    if app_state.current_page == terminal::ui::app::Page::Landing {
                        let selected = app_state.menu_items.get(app_state.selected_menu).cloned();
                        
                        if let Some(menu_item) = selected {
                            match menu_item.as_str() {
                                "Redis Connection" => {
                                    // Redis 연결 시도
                                    app_state.set_status("Connecting to Redis...".to_string());
                                    
                                    match redis_manager.connect().await {
                                        Ok(_) => {
                                            app_state.set_redis_connected(true);
                                        }
                                        Err(e) => {
                                            app_state.set_status(format!("Connection failed: {}", e));
                                        }
                                    }
                                }
                                "Redis Keys" => {
                                    // Redis 키 조회
                                    app_state.set_status("Loading keys...".to_string());
                                    
                                    match redis_manager.get_all_keys().await {
                                        Ok(keys) => {
                                            app_state.set_redis_keys(keys);
                                            app_state.current_page = terminal::ui::app::Page::RedisKeys;
                                        }
                                        Err(e) => {
                                            app_state.set_status(format!("Failed to load keys: {}", e));
                                        }
                                    }
                                }
                                _ => {
                                    app_state.execute_selected_menu();
                                }
                            }
                        }
                    }
                }
                InputEvent::Escape => {
                    if app_state.current_page != terminal::ui::app::Page::Landing {
                        app_state.go_back();
                    }
                }
                InputEvent::Char('r') => {
                    // 새로고침 (현재 페이지에서)
                    if app_state.current_page == terminal::ui::app::Page::RedisKeys {
                        match redis_manager.get_all_keys().await {
                            Ok(keys) => {
                                app_state.set_redis_keys(keys);
                                app_state.set_status("Keys refreshed".to_string());
                            }
                            Err(e) => {
                                app_state.set_status(format!("Refresh failed: {}", e));
                            }
                        }
                    }
                }
                _ => {}
            }
        }

        // 종료 체크
        if app_state.should_quit {
            break;
        }

        // CPU 사용률 절감을 위한 짧은 대기
        tokio::time::sleep(Duration::from_millis(10)).await;
    }

    Ok(())
}