
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

> cd ./entity/src/ && sea-orm-cli generate entity --lib && cd ../..
> sea-orm-cli migrate up

export PATH=$PATH:/root/.cargo/bin
sea-orm-cli migrate up

## 서버 재기동

> ps -ef | grep ./target/release/rin_agent
> kill -9 (process 번호)

## 서버 구동

> cargo build --release
> nohup ./target/release/rin_agent &

## Reference for Develop

### Gemini

> <https://ai.google.dev/gemini-api/docs?hl=ko>

#### TODO

```shell
# Use a temporary file to hold the base64 encoded image data
TEMP_B64=$(mktemp)
trap 'rm -f "$TEMP_B64"' EXIT
base64 $B64FLAGS $IMG_PATH > "$TEMP_B64"

# Use a temporary file to hold the JSON payload
TEMP_JSON=$(mktemp)
trap 'rm -f "$TEMP_JSON"' EXIT

cat > "$TEMP_JSON" << EOF
{
  "contents": [
    {
      "parts": [
        {
          "text": "Tell me about this instrument"
        },
        {
          "inline_data": {
            "mime_type": "image/jpeg",
            "data": "$(cat "$TEMP_B64")"
          }
        }
      ]
    }
  ]
}
EOF

curl "https://generativelanguage.googleapis.com/v1beta/models/gemini-2.0-flash:generateContent?key=$GEMINI_API_KEY" \
  -H 'Content-Type: application/json' \
  -X POST \
  -d "@$TEMP_JSON"
```

## SeaORM 방안

- SeaORM-Cli를 사용함.
- root에서 `sea-orm-cli migrate up`를 실행
