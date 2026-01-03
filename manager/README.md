# Manager Service

Manager 서비스는 RinAgent의 모니터링 및 관리를 담당하는 독립 서비스입니다.

## Goal

이 프로젝트는 rin_agent의 하위 프로젝트로, 아래 역할을 한다:

1. 임베디드 통신으로 오는 메시지들을 모아 Redis/Valkey로 연결
2. 서버의 관제 역할 수행
3. 시스템 헬스 체크 및 모니터링
4. MQTT를 통한 명령 수신 및 상태 보고

## 주요 기능

### 1. 벌크 로깅 시스템 (BulkLogger)

- **비동기 처리**: Tokio 기반 async/await 패턴
- **배치 처리**: 설정 가능한 배치 크기로 로그를 모아서 한 번에 Redis에 전송
- **자동 플러시**: 시간 간격마다 자동으로 버퍼 플러시
- **채널 기반 큐**: mpsc 채널을 사용한 안전한 멀티스레드 로깅

### 2. 시스템 모니터링

- **CPU 사용률**: 실시간 CPU 사용률 모니터링
- **메모리 사용률**: 시스템 메모리 사용 현황 추적
- **프로세스 감시**: 특정 프로세스의 실행 상태 모니터링
- **자동 알림**: MQTT를 통한 상태 보고

### 3. MQTT 통합

- **메시지 구독**: `manager/command`, `manager/health` 토픽 구독
- **상태 발행**: 시스템 헬스 체크 결과 발행
- **프로세스 알림**: 프로세스 상태 변화 알림

### 4. 데이터베이스 연동

- **SeaORM**: 타입 안전한 데이터베이스 접근
- **PostgreSQL**: 영구 데이터 저장

## 서버 콘솔 역할

이 프로세스를 실행되면, MQTT를 통해서 서버쪽으로 명령을 보낼 수 있다:

### 지원 명령

1. **프로세스 재부팅**: RestartCommand를 통한 프로세스 재시작
2. **헬스 체크**: 주기적인 시스템 상태 확인
3. **프로세스 모니터링**: 지정된 프로세스 감시

## 아키텍처 개선 사항

### Before (기존 구조)

```rust
// 문제점:
// 1. 동기 Redis Connection - 블로킹 작업
// 2. std::thread 사용 - Tokio 런타임과 불일치
// 3. 소유권 오류 - connection.err() 후 재사용 불가
// 4. 미완성 구현 - 실제 로직 없음
pub struct BulkLogger {
    redis_client: redis::Client,
    connection: Option<redis::Connection>, // 동기
}
```

### After (개선된 구조)

```rust
// 개선 사항:
// 1. 비동기 MultiplexedConnection - 논블로킹
// 2. tokio::spawn 사용 - 일관된 런타임
// 3. 올바른 소유권 처리 - ref 패턴 사용
// 4. 완전한 구현 - 배치 처리 + 자동 플러시
pub struct BulkLogger {
    redis_client: redis::Client,
    connection: Option<MultiplexedConnection>, // 비동기
}

pub struct BulkLoggerHandler {
    tx: mpsc::Sender<BulkLogPacket>,
}

// 채널 기반 큐 + tokio::select! 패턴
async fn process_logs(...) {
    loop {
        tokio::select! {
            Some(packet) = rx.recv() => {
                buffer.push(packet);
                if buffer.len() >= batch_size {
                    logger.bulk_push(...).await;
                }
            }
            _ = flush_timer.tick() => {
                if !buffer.is_empty() {
                    logger.bulk_push(...).await;
                }
            }
        }
    }
}
```

## 환경 변수

`.env` 파일에 다음 변수를 설정:

```bash
# 데이터베이스
DATABASE_URL=postgres://user:password@localhost:5432/rin_agent

# Redis
REDIS_URL=redis://127.0.0.1:6379

# MQTT
MQTT_HOST=localhost
MQTT_PORT=1883
MQTT_CLIENT_ID=rin_manager

# 로깅 설정
LOG_BATCH_SIZE=100
LOG_FLUSH_INTERVAL_SECS=5

# 모니터링 설정
HEALTH_CHECK_INTERVAL_SECS=30
MONITORED_PROCESS_NAME=rin_agent  # 선택사항
```

## 실행 방법

```bash
# 개발 모드
cd rin_agent/manager
cargo run

# 릴리스 모드
cargo run --release
```

## 모듈 구조

```
manager/
├── src/
│   ├── main.rs              # 엔트리 포인트
│   ├── libs/
│   │   ├── mod.rs           # 라이브러리 모듈
│   │   ├── bulk_logger.rs   # 벌크 로깅 시스템
│   │   └── find_server.rs   # 프로세스 검색
│   ├── command/
│   │   ├── mod.rs           # 명령어 모듈
│   │   └── restart.rs       # 프로세스 재시작
│   └── service/
│       ├── mod.rs           # 서비스 모듈
│       └── manager_service.rs  # 메인 서비스 로직
└── Cargo.toml
```

## 사용 예시

### 로깅

```rust
// 비동기 로깅
service.logger().log(BulkLogPacket {
    timestamp: chrono::Utc::now().timestamp() as u64,
    level: "INFO".to_string(),
    message: "작업 완료".to_string(),
}).await;

// 동기 컨텍스트에서 로깅
service.logger().log_sync(BulkLogPacket {
    timestamp: chrono::Utc::now().timestamp() as u64,
    level: "ERROR".to_string(),
    message: "오류 발생".to_string(),
});
```

### MQTT 메시지 발행

```rust
service.mqtt_client()
    .publish("my/topic", QoS::AtLeastOnce, false, "메시지")
    .await?;
```

### 프로세스 재시작

```rust
use command::restart::RestartCommand;

match RestartCommand::execute("rin_agent").await {
    Ok(msg) => println!("{}", msg),
    Err(e) => eprintln!("재시작 실패: {}", e),
}
```

## 주요 개선 포인트

1. **비동기 일관성**: 모든 I/O 작업을 async/await로 통일
2. **채널 기반 아키텍처**: 안전한 스레드 간 통신
3. **배치 처리**: 효율적인 Redis 작업
4. **자동 플러시**: 데이터 손실 방지
5. **타입 안전성**: Rust의 강력한 타입 시스템 활용
6. **에러 처리**: Result 타입과 match 패턴 사용
7. **설정 가능**: 환경 변수로 유연한 설정
