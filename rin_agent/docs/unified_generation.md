# Unified Generation System

## Overview

The unified generation system provides a single, consistent interface for generating content across multiple modalities:
- **Text Generation**: Generate text responses
- **Image Generation**: Create images from text prompts
- **Audio/Voice Generation**: Generate speech or audio from text

## Architecture

### Key Components

1. **GenerationModality Enum** (`src/gemini/types.rs`)
   - Defines the available generation modalities: `Text`, `Image`, `Audio`

2. **UnifiedGenerationConfig** (`src/gemini/types.rs`)
   - Configuration structure for specifying:
     - Which modalities to use
     - Which model to use
     - Token limits

3. **unified_generate Function** (`src/gemini/unified_generation.rs`)
   - Core function that handles all generation requests
   - Automatically routes requests to the appropriate Gemini API
   - Returns results in a unified format

### Tools

The system includes specialized tools that leverage the unified generation:

1. **image_generate** (`src/gemini/tools/image_generate.rs`)
   - Generates images from text prompts
   - Optionally uses an input image for image-to-image generation
   - Refactored to use `unified_generate`

2. **audio_generate** (`src/gemini/tools/audio_generate.rs`)
   - Generates audio/voice from text prompts
   - Uses the unified generation system

## Usage Examples

### Generating Images

```rust
use crate::gemini::types::{GenerationModality, UnifiedGenerationConfig};
use crate::gemini::unified_generation::unified_generate;

let config = UnifiedGenerationConfig {
    modalities: vec![GenerationModality::Text, GenerationModality::Image],
    model: "gemini-2.5-flash-image".to_string(),
    max_output_tokens: Some(2048),
};

let result = unified_generate(
    "A futuristic cityscape at sunset".to_string(), 
    config, 
    None
).await?;

if let Some(image_data) = result.image {
    // Handle image data
}
```

### Generating Audio

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
    // Handle audio data
}
```

### Multi-Modal Generation

```rust
// Generate both text and image
let config = UnifiedGenerationConfig {
    modalities: vec![
        GenerationModality::Text, 
        GenerationModality::Image
    ],
    model: "gemini-2.5-flash-image".to_string(),
    max_output_tokens: Some(2048),
};

let result = unified_generate(prompt, config, None).await?;

// Both result.result_message (text) and result.image may be present
```

## Benefits

1. **Consistency**: Single API for all generation types
2. **Flexibility**: Easy to combine multiple modalities
3. **Maintainability**: Centralized generation logic
4. **Extensibility**: Easy to add new modalities in the future

## Model Support

Currently, the system uses:
- `gemini-2.5-flash-image` (GEMINI_NANO_BANANA) for image and audio generation
- `gemini-flash-latest` (GEMINI_MODEL_FLASH) for text generation
- `gemini-3-pro-preview` (GEMINI_MODEL_PRO) for advanced text generation

The unified system allows any model that supports the multimodal API to be used.

## Future Enhancements

Potential future additions:
- Video generation support
- Streaming generation for audio
- Fine-grained control over generation parameters per modality
- Caching support for unified generation
