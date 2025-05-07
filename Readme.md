
# CanarinAgent

- 자기관리등을 위한 Gen AI 서버 대리 프로시저.

```txt
src/
├── main.rs                # 진입점: 전체 애플리케이션 초기화
├── lib.rs                 # 공통 로직 및 모듈 정의
├── api/                   # 웹 API 관련 코드
│   ├── mod.rs             # API 모듈 정의
│   ├── routes.rs          # API 라우트 정의
│   └── handlers.rs        # API 요청 핸들러
├── discord/               # Discord 봇 관련 코드
│   ├── mod.rs             # Discord 모듈 정의
│   └── bot.rs             # Discord 봇 로직
├── gemini/                # Gemini 호출 관련 코드
│   ├── mod.rs             # Gemini 모듈 정의
│   └── client.rs          # Gemini 클라이언트 로직
├── db/                    # 데이터베이스 관련 코드
│   ├── mod.rs             # DB 모듈 정의
│   └── models.rs          # DB 모델 정의
└── config.rs              # 설정 및 환경 변수 로드

```

## DB migration

> cd .\entity\src\ && sea-orm-cli generate entity --lib && cd ../..

## 서버 재기동

> ps -ef | grep ./target/release/rin_agent
> kill -9 (process 번호)

## 서버 구동

> cargo build --release
> nohup ./target/release/rin_agent &
