#[cfg(test)]
mod tests {
    use crate::gemini::types::{GenerationModality, UnifiedGenerationConfig};

    #[test]
    fn test_generation_modality_creation() {
        let text_modality = GenerationModality::Text;
        let image_modality = GenerationModality::Image;
        let audio_modality = GenerationModality::Audio;

        // Test that modalities can be created and compared
        assert_eq!(text_modality, GenerationModality::Text);
        assert_eq!(image_modality, GenerationModality::Image);
        assert_eq!(audio_modality, GenerationModality::Audio);
    }

    #[test]
    fn test_unified_generation_config_default() {
        let config = UnifiedGenerationConfig::default();
        
        // Test default values
        assert_eq!(config.modalities.len(), 1);
        assert_eq!(config.modalities[0], GenerationModality::Text);
        assert_eq!(config.model, "gemini-flash-latest");
        assert_eq!(config.max_output_tokens, Some(2048));
    }

    #[test]
    fn test_unified_generation_config_custom() {
        let config = UnifiedGenerationConfig {
            modalities: vec![
                GenerationModality::Text,
                GenerationModality::Image,
            ],
            model: "gemini-2.5-flash-image".to_string(),
            max_output_tokens: Some(4096),
        };

        // Test custom configuration
        assert_eq!(config.modalities.len(), 2);
        assert_eq!(config.modalities[0], GenerationModality::Text);
        assert_eq!(config.modalities[1], GenerationModality::Image);
        assert_eq!(config.model, "gemini-2.5-flash-image");
        assert_eq!(config.max_output_tokens, Some(4096));
    }

    #[test]
    fn test_unified_generation_config_multi_modal() {
        let config = UnifiedGenerationConfig {
            modalities: vec![
                GenerationModality::Text,
                GenerationModality::Image,
                GenerationModality::Audio,
            ],
            model: "gemini-2.5-flash-image".to_string(),
            max_output_tokens: Some(2048),
        };

        // Test multi-modal configuration
        assert_eq!(config.modalities.len(), 3);
        assert!(config.modalities.contains(&GenerationModality::Text));
        assert!(config.modalities.contains(&GenerationModality::Image));
        assert!(config.modalities.contains(&GenerationModality::Audio));
    }
}
