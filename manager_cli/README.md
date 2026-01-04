# Manager CLI

Manager 서비스와 통신하는 독립 CLI 클라이언트입니다.

## 목적

- Manager 데몬 서비스에 MQTT를 통해 명령 전송
- 실시간 상태 모니터링
- 원격/로컬 모두 지원

## 실행 방법

```bash
# Manager 서비스 먼저 실행 (백그라운드)
cargo run -p manager

# 별도 터미널에서 CLI 실행
cargo run -p manager_cli
```

## 사용 가능한 명령어

```
manager> help                          # 도움말
manager> status                        # 시스템 상태 확인
manager> info                          # 시스템 정보 조회
manager> restart <process_name>        # 프로세스 재시작
manager> monitor <process_name> [10]   # 프로세스 모니터링 시작
manager> unmonitor <process_name>      # 프로세스 모니터링 중지
manager> exit                          # 종료
```

## 특징

- **비동기 응답**: MQTT를 통한 실시간 응답 수신
- **컬러 출력**: colored 크레이트로 가독성 향상
- **독립 실행**: Manager 재시작 없이 CLI만 껐다 켤 수 있음
- **다중 접속**: 여러 터미널에서 동시 접속 가능

## 환경 설정

`.env` 파일에서 MQTT 설정:

```env
MQTT_HOST=localhost
MQTT_PORT=1883
```
