# Manager CLI

Rust 기반 Redis 제어 TUI (Terminal User Interface) 애플리케이션

## 개요

이 프로젝트는 **ratatui**를 기반으로 하는 터미널 기반 Redis 관리 도구입니다. 사용자 친화적인 인터페이스를 통해 Redis 서버를 제어하고 모니터링할 수 있습니다.

## 주요 기능

- 🔌 **Redis 연결 관리**: Redis 서버 연결 및 상태 확인
- 🔑 **키 관리**: Redis 키 조회, 생성, 삭제
- 📡 **Pub/Sub 모니터링**: Redis Pub/Sub 채널 관리 (구현 예정)
- ⚙️ **설정**: 애플리케이션 설정 관리

## 프로젝트 구조

```
manager_cli/
├── src/
│   ├── main.rs                    # 메인 애플리케이션 진입점 및 이벤트 루프
│   ├── terminal/                  # 터미널 관련 모듈
│   │   ├── core/                  # 핵심 입출력 제어
│   │   │   ├── input.rs          # 키 입력 처리 (InputEvent, InputHandler)
│   │   │   ├── terminal.rs       # 터미널 제어 (TerminalManager)
│   │   │   └── mod.rs
│   │   └── ui/                    # UI 컴포넌트
│   │       ├── app.rs            # 앱 상태 관리 (AppState)
│   │       ├── layout/           # 레이아웃 구성
│   │       │   ├── landing.rs    # 랜딩 페이지 (메인 메뉴)
│   │       │   └── mod.rs
│   │       ├── pages/            # 개별 페이지
│   │       │   ├── redis_keys.rs    # Redis 키 목록 페이지
│   │       │   ├── redis_pubsub.rs  # Pub/Sub 페이지
│   │       │   ├── settings.rs      # 설정 페이지
│   │       │   └── mod.rs
│   │       └── mod.rs
│   └── connection/               # Redis 연결 모듈
│       ├── communication.rs      # Redis 연결 및 명령 실행 (RedisManager)
│       └── mod.rs
├── Cargo.toml
└── README.md
```

## 아키텍처

### 1. Terminal/Core 모듈

터미널의 핵심 입출력을 제어하는 모듈입니다.

#### `terminal/core/input.rs`
- **InputEvent**: 키보드 입력 이벤트 타입
  - `Char`, `Enter`, `Backspace`, `Escape`, `Arrow keys` 등
- **InputHandler**: 키 입력을 폴링하고 InputEvent로 변환

#### `terminal/core/terminal.rs`
- **TerminalManager**: 터미널 초기화 및 관리
  - Raw mode 활성화/비활성화
  - Alternate screen 전환
  - 터미널 정리 (Drop trait)

### 2. Terminal/UI 모듈

UI 창의 집합으로, 페이지를 렌더링하는 모듈입니다.

#### `terminal/ui/app.rs`
- **AppState**: 애플리케이션 전역 상태
  - 현재 페이지, 메뉴 선택, Redis 연결 상태
  - 메뉴 네비게이션 함수
  - 입력 버퍼 관리

#### `terminal/ui/layout/landing.rs`
- 메인 랜딩 페이지 렌더링
- 헤더, 메뉴, 정보 패널, 푸터 구성

#### `terminal/ui/pages/*`
- **redis_keys.rs**: Redis 키 목록 표시
- **redis_pubsub.rs**: Pub/Sub 모니터 (구현 예정)
- **settings.rs**: 설정 페이지 (구현 예정)

### 3. Connection 모듈

Redis 연결 및 제어 기능을 제공합니다.

#### `connection/communication.rs`
- **RedisManager**: Redis 서버와의 통신 관리
  - 연결/연결 해제
  - PING, INFO, DBSIZE 등 기본 명령
  - 키 조회, 설정, 삭제
  - TTL, EXPIRE 관리

## 기술 스택

- **ratatui**: 터미널 UI 프레임워크
- **crossterm**: 크로스 플랫폼 터미널 제어
- **redis**: Redis 클라이언트
- **tokio**: 비동기 런타임
- **anyhow**: 에러 처리

## 설치 및 실행

### 필수 요구사항

- Rust 1.70 이상
- Redis 서버 (로컬 또는 원격)

### 환경 설정

`.env` 파일을 생성하고 Redis 연결 정보를 설정합니다:

```env
REDIS_URL=redis://127.0.0.1:6379
```

### 빌드 및 실행

```bash
# 개발 모드
cd rin_agent/manager_cli
cargo run

# 릴리스 모드
cargo build --release
./target/release/manager_cli
```

## 사용법

### 키보드 단축키

#### 메인 메뉴
- `↑/↓`: 메뉴 네비게이션
- `Enter`: 메뉴 선택
- `q` 또는 `Ctrl+C`: 종료

#### Redis 키 페이지
- `ESC`: 메인 메뉴로 돌아가기
- `r`: 키 목록 새로고침

#### 공통
- `Ctrl+C` 또는 `Ctrl+Q`: 즉시 종료

## 개발 가이드

### 새 페이지 추가하기

1. `src/terminal/ui/pages/`에 새 파일 생성
2. `render` 함수 구현:
   ```rust
   pub fn render(f: &mut Frame, app: &AppState, area: Rect) {
       // 페이지 렌더링 로직
   }
   ```
3. `src/terminal/ui/pages/mod.rs`에 모듈 추가
4. `src/terminal/ui/app.rs`의 `Page` enum에 추가
5. `src/main.rs`의 렌더링 로직에 케이스 추가

### Redis 명령 추가하기

1. `src/connection/communication.rs`의 `RedisManager`에 메서드 추가:
   ```rust
   pub async fn your_command(&self) -> Result<YourType> {
       let mut conn_guard = self.connection.lock().await;
       let conn = conn_guard.as_mut().context("Not connected")?;
       
       // Redis 명령 실행
       conn.your_redis_command()
           .await
           .context("Failed to execute command")
   }
   ```

## 라이선스

이 프로젝트는 MIT 라이선스를 따릅니다.

## 기여

버그 리포트, 기능 요청, Pull Request는 언제나 환영합니다!
