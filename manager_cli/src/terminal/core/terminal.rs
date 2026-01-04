use anyhow::Result;
use crossterm::{
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    Terminal as RatatuiTerminal,
};
use std::io::{self, Stdout};

/// 터미널 타입 별칭
pub type Terminal = RatatuiTerminal<CrosstermBackend<Stdout>>;

/// 터미널 관리자
pub struct TerminalManager {
    terminal: Terminal,
}

impl TerminalManager {
    /// 새로운 TerminalManager 생성 및 초기화
    /// 
    /// raw mode를 활성화하고 alternate screen으로 전환합니다.
    pub fn new() -> Result<Self> {
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen)?;
        let backend = CrosstermBackend::new(stdout);
        let terminal = RatatuiTerminal::new(backend)?;
        
        Ok(Self { terminal })
    }

    /// 터미널 참조 반환
    pub fn terminal(&mut self) -> &mut Terminal {
        &mut self.terminal
    }

    /// 터미널 화면 클리어
    pub fn clear(&mut self) -> Result<()> {
        self.terminal.clear()?;
        Ok(())
    }

    /// 터미널 크기 가져오기
    pub fn size(&self) -> Result<(u16, u16)> {
        let size = self.terminal.size()?;
        Ok((size.width, size.height))
    }
}

impl Drop for TerminalManager {
    /// TerminalManager가 drop될 때 터미널을 원래 상태로 복구
    fn drop(&mut self) {
        let _ = disable_raw_mode();
        let _ = execute!(io::stdout(), LeaveAlternateScreen);
    }
}

/// 터미널 초기화 헬퍼 함수
pub fn setup_terminal() -> Result<TerminalManager> {
    TerminalManager::new()
}

/// 터미널 정리 헬퍼 함수
pub fn cleanup_terminal() -> Result<()> {
    disable_raw_mode()?;
    execute!(io::stdout(), LeaveAlternateScreen)?;
    Ok(())
}
