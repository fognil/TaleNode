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
    /// AI writing suggestions ready.
    WritingSuggestionsReady {
        node_id: uuid::Uuid,
        suggestions: Vec<String>,
    },
    /// AI tone check completed.
    ToneCheckReady {
        node_id: uuid::Uuid,
        report: String,
    },
    /// AI writing operation failed.
    WritingError(String),
    /// AI model list fetched from provider API.
    ModelsLoaded(Vec<String>),
    /// Failed to fetch AI model list.
    ModelsError(String),
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

    #[test]
    fn async_channel_translation_done() {
        let (tx, rx) = std::sync::mpsc::channel();
        tx.send(AsyncResult::TranslationDone {
            locale: "fr".to_string(),
            translations: vec![
                ("dlg_abc".to_string(), "Bonjour".to_string()),
                ("dlg_def".to_string(), "Au revoir".to_string()),
            ],
        })
        .unwrap();
        let result = rx.try_recv().unwrap();
        if let AsyncResult::TranslationDone { locale, translations } = result {
            assert_eq!(locale, "fr");
            assert_eq!(translations.len(), 2);
            assert_eq!(translations[0], ("dlg_abc".to_string(), "Bonjour".to_string()));
        } else {
            panic!("Wrong variant");
        }
    }

    #[test]
    fn async_channel_voice_generated() {
        let (tx, rx) = std::sync::mpsc::channel();
        let node_id = uuid::Uuid::new_v4();
        tx.send(AsyncResult::VoiceGenerated {
            node_id,
            audio_path: "voices/dlg_abc.mp3".to_string(),
        })
        .unwrap();
        let result = rx.try_recv().unwrap();
        if let AsyncResult::VoiceGenerated { node_id: id, audio_path } = result {
            assert_eq!(id, node_id);
            assert_eq!(audio_path, "voices/dlg_abc.mp3");
        } else {
            panic!("Wrong variant");
        }
    }

    #[test]
    fn async_channel_empty_returns_none() {
        let (_tx, rx) = std::sync::mpsc::channel::<AsyncResult>();
        assert!(rx.try_recv().is_err());
    }
}
