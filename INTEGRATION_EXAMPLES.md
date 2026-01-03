# Contract 통합 예시

## 개요

rin_agent와 manager 모두 contract를 사용하여 환경변수와 메시지 프로토콜을 관리합니다.

### 주요 개선 사항

1. **중복 코드 제거**: 환경변수 로드가 각 모듈에서 중앙 집중화됨
2. **타입 안전성**: `RinAgentConfig`, `ManagerConfig`로 컴파일 타임 검증
3. **실행 시점 전략 선택**: CLI 인자로 dotenv/시스템 환경변수 선택 가능
4. **일관된 메시지 형식**: MQTT/Redis 메시지가 contract 타입 사용

## Manager 실행

```bash
# 시스템 환경변수 우선 (프로덕션 권장)
cargo run -p manager

# .env 파일만 사용 (로컬 개발)
cargo run -p manager -- --env-strategy dotenv-only

# 커스텀 .env 파일 지정
cargo run -p manager -- --env-strategy dotenv-only --env-file .env.production

# 시스템 환경변수만 사용 (Docker/K8s)
cargo run -p manager -- --env-strategy system-only
```

## RinAgent 실행

```bash
# 시스템 환경변수 우선 (프로덕션 권장)
cargo run -p rin_agent

# .env 파일만 사용 (로컬 개발)
cargo run -p rin_agent -- --env-strategy dotenv-only

# 하이브리드: .env 우선, 시스템 폴백
cargo run -p rin_agent -- --env-strategy dotenv-preferred

# 커스텀 .env 파일
cargo run -p rin_agent -- --env-file .env.test
```

## 환경변수 설정 예시

### .env (로컬 개발용)

```bash
# ========================================
# 공통 설정 (CommonConfig)
# ========================================
DATABASE_URL=postgres://rin:rin123@localhost:5432/rin_agent
REDIS_URL=redis://127.0.0.1:6379
MQTT_HOST=localhost
MQTT_PORT=1883

# ========================================
# Manager 전용 설정 (ManagerConfig)
# ========================================
MQTT_CLIENT_ID=rin_manager
LOG_BATCH_SIZE=100
LOG_FLUSH_INTERVAL_SECS=5
HEALTH_CHECK_INTERVAL_SECS=30
MONITORED_PROCESS_NAME=rin_agent

# ========================================
# RinAgent 전용 설정 (RinAgentConfig)
# ========================================
DISCORD_TOKEN=your_discord_token_here
DISCORD_CLIENT_ID=your_client_id_here
GEMINI_API_KEY=your_gemini_api_key_here
MANAGER_ID=discord_user_id_optional
```

### Docker/K8s (시스템 환경변수)

```yaml
apiVersion: v1
kind: ConfigMap
metadata:
  name: rin-agent-config
data:
  DATABASE_URL: "postgres://..."
  REDIS_URL: "redis://..."
  MQTT_HOST: "mqtt-service"
  MQTT_PORT: "1883"
```

## MQTT 메시지 예시

### Manager에게 명령 보내기

```bash
# 헬스 체크 요청
mosquitto_pub -t "manager/command" -m '{"type":"health_check"}'

# 프로세스 재시작
mosquitto_pub -t "manager/command" -m '{"type":"restart_process","process_name":"rin_agent","force":true}'

# 프로세스 모니터링 시작
mosquitto_pub -t "manager/command" -m '{"type":"start_monitoring","process_name":"rin_agent","interval_secs":30}'
```

### Manager 상태 구독

```bash
# 헬스 리포트 구독
mosquitto_sub -t "manager/health/report"

# 프로세스 상태 구독
mosquitto_sub -t "manager/process/status"

# 프로세스 알림 구독
mosquitto_sub -t "manager/process/alert"
```
