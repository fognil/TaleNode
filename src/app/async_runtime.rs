use serde::{Deserialize, Serialize};

/// Results delivered from async tasks back to the main egui thread.
#[allow(dead_code)]
pub enum AsyncResult {
    /// DeepL translation batch completed.
    TranslationDone {
        locale: String,
        /// (string_key, translated_text) pairs.
        translations: Vec<(String, String)>,
    },
    /// Translation task failed.
    TranslationError(String),
    /// ElevenLabs voice list loaded.
    VoiceListLoaded(Vec<VoiceInfo>),
    /// Voice generation completed for a single node.
    VoiceGenerated {
        node_id: uuid::Uuid,
        audio_path: String,
    },
    /// Voice operation failed.
    VoiceError(String),
    /// Collaboration message received from WebSocket.
    CollabMessage(String),
    /// Collaboration error.
    CollabError(String),
}

/// Metadata about an ElevenLabs voice.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VoiceInfo {
    pub voice_id: String,
    pub name: String,
    #[serde(default)]
    pub category: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn voice_info_serialization() {
        let v = VoiceInfo {
            voice_id: "abc123".to_string(),
            name: "Rachel".to_string(),
            category: "premade".to_string(),
        };
        let json = serde_json::to_string(&v).unwrap();
        let loaded: VoiceInfo = serde_json::from_str(&json).unwrap();
        assert_eq!(loaded.voice_id, "abc123");
        assert_eq!(loaded.name, "Rachel");
    }

    #[test]
    fn voice_info_default_category() {
        let json = r#"{"voice_id":"x","name":"Test"}"#;
        let loaded: VoiceInfo = serde_json::from_str(json).unwrap();
        assert!(loaded.category.is_empty());
    }
}
