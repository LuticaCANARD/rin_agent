# Unified Generation System - Quick Start Guide

## Overview

이 PR은 기존의 텍스트, 이미지, 음성 생성 모델을 하나의 통합된 인터페이스로 일원화했습니다.

## 주요 변경 사항

### 1. 통합된 생성 모달리티 (GenerationModality)

```rust
pub enum GenerationModality {
    Text,   // 텍스트 생성
    Image,  // 이미지 생성
    Audio,  // 음성/오디오 생성
}
```

### 2. 통합 생성 설정 (UnifiedGenerationConfig)

```rust
pub struct UnifiedGenerationConfig {
    pub modalities: Vec<GenerationModality>,  // 사용할 모달리티 목록
    pub model: String,                         // 사용할 모델
    pub max_output_tokens: Option<i32>,       // 최대 토큰 수
}
```

### 3. 통합 생성 함수 (unified_generate)

모든 생성 타입을 처리하는 단일 함수:

```rust
pub async fn unified_generate(
    prompt: String,
    config: UnifiedGenerationConfig,
    image_input: Option<GeminiImageInputType>,
) -> Result<GeminiActionResult, String>
```

## 사용 예제

### 이미지 생성

```rust
let config = UnifiedGenerationConfig {
    modalities: vec![GenerationModality::Text, GenerationModality::Image],
    model: "gemini-2.5-flash-image".to_string(),
    max_output_tokens: Some(2048),
};

let result = unified_generate(
    "미래적인 도시 풍경".to_string(),
    config,
    None
).await?;

if let Some(image_data) = result.image {
    // 이미지 데이터 처리
}
```

### 음성 생성

```rust
let config = UnifiedGenerationConfig {
    modalities: vec![GenerationModality::Text, GenerationModality::Audio],
    model: "gemini-2.5-flash-image".to_string(),
    max_output_tokens: Some(2048),
};

let result = unified_generate(
    "안녕하세요, 만나서 반갑습니다.".to_string(),
    config,
    None
).await?;

if let Some(audio_data) = result.audio {
    // 오디오 데이터 처리
}
```

### 멀티모달 생성 (텍스트 + 이미지)

```rust
let config = UnifiedGenerationConfig {
    modalities: vec![
        GenerationModality::Text,
        GenerationModality::Image
    ],
    model: "gemini-2.5-flash-image".to_string(),
    max_output_tokens: Some(2048),
};

let result = unified_generate(prompt, config, None).await?;

// result.result_message (텍스트)와 result.image 모두 사용 가능
```

## 새로운 도구

### audio_generate 도구

음성 생성을 위한 새로운 도구가 추가되었습니다:

```rust
// gemini_setting.rs에서 자동으로 등록됨
pub static GEMINI_BOT_TOOLS: LazyLock<...> = LazyLock::new(|| {
    load_gemini_tools!(
        set_alarm,
        discord_response,
        searching,
        web_connect,
        image_generate,
        audio_generate  // ← 새로 추가
    )
    ...
});
```

## 기존 코드와의 호환성

### image_generate 도구

기존의 `image_generate` 도구는 내부적으로 `unified_generate`를 사용하도록 리팩토링되었지만, 
외부 인터페이스는 동일하게 유지됩니다. 따라서 기존 코드는 수정 없이 동작합니다.

### 텍스트 생성

기존의 텍스트 생성 시스템 (`gemini_client.rs`)은 그대로 유지됩니다. 
`unified_generate`는 대안적인 인터페이스를 제공하며, 필요에 따라 선택적으로 사용할 수 있습니다.

## 장점

1. **일관성**: 모든 생성 타입에 대해 단일 API
2. **유연성**: 여러 모달리티를 쉽게 조합 가능
3. **유지보수성**: 중앙화된 생성 로직으로 코드 중복 감소
4. **확장성**: 향후 새로운 모달리티(예: 비디오) 추가 용이

## 테스트

모든 단위 테스트가 통과했습니다:

```
running 4 tests
test tests::test_unified_generation::tests::test_generation_modality_creation ... ok
test tests::test_unified_generation::tests::test_unified_generation_config_custom ... ok
test tests::test_unified_generation::tests::test_unified_generation_config_default ... ok
test tests::test_unified_generation::tests::test_unified_generation_config_multi_modal ... ok

test result: ok. 4 passed; 0 failed; 0 ignored
```

## 문서

자세한 문서는 다음 파일을 참조하세요:
- `docs/unified_generation.md` - 상세한 아키텍처 및 사용 가이드
- `src/gemini/unified_generation.rs` - 소스 코드 내 인라인 문서

## 다음 단계

이 시스템은 향후 다음과 같은 개선이 가능합니다:
- 비디오 생성 지원
- 오디오 스트리밍 생성
- 모달리티별 세밀한 파라미터 제어
- 캐싱 지원
