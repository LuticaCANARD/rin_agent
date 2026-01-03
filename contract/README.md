# Contract - RinAgent 프로세스 간 통신 규약

이 크레이트는 `rin_agent`와 `manager` 사이의 통신 프로토콜과 공유 타입을 정의합니다.

## 📋 목차

- [역할 분담](#역할-분담)
- [Feature Flags](#feature-flags)
- [통신 프로토콜](#통신-프로토콜)
- [메시지 타입](#메시지-타입)
- [환경변수 관리](#환경변수-관리)
- [사용 예시](#사용-예시)

## 역할 분담

### rin_agent (Main Application)
- ✅ Discord 봇 운영 (명령어, 음성, 이벤트 처리)
- ✅ Gemini AI 통합 및 대화 처리
- ✅ Rocket 웹 서버 (REST API, 정적 파일 제공)
- ✅ 사용자 상호작용 및 비즈니스 로직
- ✅ 알람 스케줄링 및 실행
- ✅ PostgreSQL 직접 접근 (사용자 데이터, AI 컨텍스트)

### manager (System Manager)
- ✅ 시스템 헬스 모니터링 (CPU, 메모리 사용률)
- ✅ 프로세스 감시 및 자동 재시작
- ✅ 벌크 로그 수집 및 Redis 저장
- ✅ MQTT 기반 명령 수신 및 상태 보고
- ✅ rin_agent 프로세스 생명주기 관리
- 📤 시스템 메트릭 발행
- 📥 외부에서 관리 명령 수신

## Feature Flags

contract 크레이트는 선택적으로 기능을 활성화/비활성화할 수 있는 feature flags를 제공합니다.

### 사용 가능한 Features

#### `mqtt` (default)

MQTT 관련 타입과 함수를 포함합니다:
- `topics` 모듈 (MQTT 토픽 상수)
- `ManagerCommand` enum
- `ManagerResponse` enum  
- `RinAgentCommand` enum (현재 비사용)
- `RinAgentStatus` enum (현재 비사용)
- `RestartResult` struct

### Cargo.toml 설정 예시

```toml
# Manager - MQTT 기능 필요 (default)
[dependencies]
contract = { path = "../contract" }
# 또는 명시적으로
contract = { path = "../contract", features = ["mqtt"] }

# RinAgent - MQTT 기능 불필요
[dependencies]
contract = { path = "../contract", default-features = false }
```

> **주의**: `mqtt` feature를 비활성화하면 MQTT 관련 타입이 컴파일되지 않아 바이너리 크기가 줄어들고 의존성이 감소합니다.

## 통신 프로토콜

### MQTT 토픽 구조

Manager만 MQTT를 사용합니다:

```
manager/
├── command              # Manager 명령 수신
├── health/report        # Manager 헬스 체크 보고
├── process/status       # 프로세스 상태 보고
└── process/alert        # 프로세스 이상 알림
```

> **참고**: RinAgent는 MQTT를 사용하지 않습니다. Discord 봇 이벤트와 Rocket 웹 서버만 사용합니다.

### Redis 데이터 구조

```
logs (LIST)              # 로그 패킷 큐 (JSON)
  - timestamp: u64
  - level: LogLevel
  - message: String
  - source: Option<String>
  - metadata: Option<Value>
```

## 메시지 타입

### LogPacket

Redis에 저장되는 로그 패킷 (모든 feature에서 사용 가능):

```rust
use contract::{LogPacket, LogLevel};

let log = LogPacket::new(LogLevel::Info, "서버 시작됨")
    .with_source("rin_agent")
    .with_metadata(serde_json::json!({
        "version": "1.0.0"
    }));
```

### ManagerCommand

Manager에게 보내는 명령 (⚠️ `mqtt` feature 필요):

```rust
use contract::ManagerCommand;

// 프로세스 재시작
let cmd = ManagerCommand::RestartProcess {
    process_name: "rin_agent".to_string(),
    force: Some(true),
};

// 헬스 체크
let cmd = ManagerCommand::HealthCheck;

// 프로세스 모니터링 시작
let cmd = ManagerCommand::StartMonitoring {
    process_name: "rin_agent".to_string(),
    interval_secs: 30,
};
```

### ManagerResponse

Manager의 응답:

```rust
use contract::ManagerResponse;

// 헬스 체크 응답
let response = ManagerResponse::HealthReport {
    cpu_usage: 15.5,
    memory_usage_percent: 45.2,
    total_memory_mb: 16384,
    used_memory_mb: 7403,
    timestamp: 1735891200,
};

// 프로세스 상태
let response = ManagerResponse::ProcessStatus {
    process_name: "rin_agent".to_string(),
    is_running: true,
    pid: Some(12345),
    timestamp: 1735891200,
};
```

### RinAgentCommand

RinAgent에게 보내는 명령:

```rust
use contract::RinAgentCommand;

// 알람 트리거
let cmd = RinAgentCommand::TriggerAlarm {
    alarm_id: 123,
    custom_message: Some("긴급 점검 알림".to_string()),
};

// 서비스 재시작
let cmd = RinAgentCommand::Restart {
    graceful: true,
};
```

### RinAgentStatus

RinAgent의 상태:

```rust
use contract::RinAgentStatus;

let status = RinAgentStatus::Running {
    uptime_secs: 86400,
    connected_guilds: 5,
    active_voice_sessions: 2,
    timestamp: 1735891200,
};
```

## 환경변수 관리

### 전략 선택

Contract는 4가지 환경변수 로드 전략을 제공합니다:

| 전략 | 설명 | 사용 시나리오 |
|------|------|---------------|
| `dotenv-only` | .env 파일만 사용 | 로컬 개발 환경 |
| `system-only` | 시스템 환경변수만 사용 | Docker/K8s 배포 |
| `dotenv-preferred` | .env 우선, 시스템 폴백 | 하이브리드 개발 |
| `system-preferred` | 시스템 우선, .env 폴백 | **프로덕션 권장** |

### CLI 사용법

```bash
# 시스템 환경변수 우선 (기본값)
cargo run

# .env 파일만 사용
cargo run -- --env-strategy dotenv-only

# 커스텀 .env 파일 지정
cargo run -- --env-strategy dotenv-only --env-file .env.production

# 시스템 환경변수만 사용
cargo run -- --env-strategy system-only
```

### 프로그램 내 사용

```rust
use contract::config::{EnvConfigBuilder, EnvLoadStrategy};

// 자동 전략 선택 (CLI 인자에서 파싱)
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let strategy = contract::config::parse_env_strategy_from_args();
    let dotenv_path = contract::config::parse_dotenv_path_from_args();
    
    let mut builder = EnvConfigBuilder::new()
        .strategy(strategy)
        .ignore_missing(true);
    
    if let Some(path) = dotenv_path {
        builder = builder.dotenv_path(path);
    }
    
    builder.load()?;
    
    // 환경변수 로드 완료, 설정 로드
    let config = contract::config::ManagerConfig::from_env()?;
    
    Ok(())
}
```

### 설정 구조체

**Manager 설정:**

```rust
use contract::config::ManagerConfig;

let config = ManagerConfig::from_env()?;
println!("Redis: {}", config.common.redis_url);
println!("MQTT: {}:{}", config.common.mqtt_host, config.common.mqtt_port);
```

**RinAgent 설정:**

```rust
use contract::config::RinAgentConfig;

let config = RinAgentConfig::from_env()?;
println!("Discord Token: {}", config.discord_token);
println!("Gemini API Key: {}", config.gemini_api_key);
```

## 환경변수 목록

### 공통 (Common)

| 변수 | 필수 | 기본값 | 설명 |
|------|------|--------|------|
| `REDIS_URL` | ❌ | `redis://127.0.0.1:6379` | Redis 서버 URL |
| `DATABASE_URL` | ✅ | - | PostgreSQL 연결 문자열 |
| `MQTT_HOST` | ❌ | `localhost` | MQTT 브로커 호스트 |
| `MQTT_PORT` | ❌ | `1883` | MQTT 브로커 포트 |

### Manager 전용

| 변수 | 필수 | 기본값 | 설명 |
|------|------|--------|------|
| `MQTT_CLIENT_ID` | ❌ | `rin_manager` | MQTT 클라이언트 ID |
| `LOG_BATCH_SIZE` | ❌ | `100` | 로그 배치 크기 |
| `LOG_FLUSH_INTERVAL_SECS` | ❌ | `5` | 로그 플러시 간격 (초) |
| `HEALTH_CHECK_INTERVAL_SECS` | ❌ | `30` | 헬스 체크 간격 (초) |
| `MONITORED_PROCESS_NAME` | ❌ | - | 감시할 프로세스 이름 |

### RinAgent 전용

| 변수 | 필수 | 기본값 | 설명 |
|------|------|--------|------|
| `DISCORD_TOKEN` | ✅ | - | Discord 봇 토큰 |
| `DISCORD_CLIENT_ID` | ✅ | - | Discord 클라이언트 ID |
| `GEMINI_API_KEY` | ✅ | - | Google Gemini API 키 |
| `MANAGER_ID` | ❌ | - | Manager Discord 사용자 ID |

## 사용 예시

### Manager에서 MQTT 메시지 발행

```rust
use contract::{ManagerResponse, topics};

let response = ManagerResponse::HealthReport {
    cpu_usage: 15.5,
    memory_usage_percent: 45.2,
    total_memory_mb: 16384,
    used_memory_mb: 7403,
    timestamp: chrono::Utc::now().timestamp() as u64,
};

let json = serde_json::to_string(&response)?;
mqtt_client.publish(
    topics::MANAGER_HEALTH_REPORT,
    rumqttc::QoS::AtLeastOnce,
    false,
    json
).await?;
```

### RinAgent에서 로그 전송

```rust
use contract::{LogPacket, LogLevel};

let log = LogPacket::new(LogLevel::Info, "Discord bot connected")
    .with_source("rin_agent");

// BulkLogger를 통해 Redis로 전송
logger.log(log).await;
```

### Manager 명령 수신 처리

```rust
use contract::{ManagerCommand, topics};

mqtt_client.subscribe(topics::MANAGER_COMMAND, QoS::AtLeastOnce).await?;

// 이벤트 루프에서
if let Event::Incoming(Packet::Publish(publish)) = event {
    if publish.topic == topics::MANAGER_COMMAND {
        let cmd: ManagerCommand = serde_json::from_slice(&publish.payload)?;
        
        match cmd {
            ManagerCommand::RestartProcess { process_name, force } => {
                // 프로세스 재시작 로직
            }
            ManagerCommand::HealthCheck => {
                // 헬스 체크 수행
            }
            _ => {}
        }
    }
}
```

## 버전 관리

Contract 버전: `1.0.0`

호환성:
- `rin_agent` >= 0.1.0
- `manager` >= 0.1.0

메시지 프로토콜 변경 시 `CONTRACT_VERSION` 상수를 업데이트하세요.
