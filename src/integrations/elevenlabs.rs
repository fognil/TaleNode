use serde::Deserialize;
use std::path::PathBuf;

use crate::app::async_runtime::VoiceInfo;

const API_BASE: &str = "https://api.elevenlabs.io";

pub struct ElevenLabsClient {
    api_key: String,
    client: reqwest::Client,
}

#[derive(Deserialize)]
struct VoicesResponse {
    voices: Vec<VoiceEntry>,
}

#[derive(Deserialize)]
struct VoiceEntry {
    voice_id: String,
    name: String,
    #[serde(default)]
    category: String,
}

impl ElevenLabsClient {
    pub fn new(api_key: String) -> Self {
        Self {
            api_key,
            client: reqwest::Client::new(),
        }
    }

    /// Fetch available voices from ElevenLabs.
    pub async fn list_voices(&self) -> Result<Vec<VoiceInfo>, String> {
        let url = format!("{API_BASE}/v1/voices");
        let resp = self
            .client
            .get(&url)
            .header("xi-api-key", &self.api_key)
            .send()
            .await
            .map_err(|e| format!("ElevenLabs request failed: {e}"))?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            return Err(format!("ElevenLabs API error {status}: {body}"));
        }

        let data: VoicesResponse = resp
            .json()
            .await
            .map_err(|e| format!("ElevenLabs parse error: {e}"))?;

        Ok(data
            .voices
            .into_iter()
            .map(|v| VoiceInfo {
                voice_id: v.voice_id,
                name: v.name,
                category: v.category,
            })
            .collect())
    }

    /// Generate speech for the given text and save to the output path.
    /// Returns the output path on success.
    pub async fn generate_voice(
        &self,
        voice_id: &str,
        text: &str,
        output_path: &PathBuf,
    ) -> Result<String, String> {
        let url = format!("{API_BASE}/v1/text-to-speech/{voice_id}");

        let body = serde_json::json!({
            "text": text,
            "model_id": "eleven_multilingual_v2",
        });

        let resp = self
            .client
            .post(&url)
            .header("xi-api-key", &self.api_key)
            .header("Accept", "audio/mpeg")
            .json(&body)
            .send()
            .await
            .map_err(|e| format!("ElevenLabs TTS request failed: {e}"))?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body_text = resp.text().await.unwrap_or_default();
            return Err(format!("ElevenLabs TTS error {status}: {body_text}"));
        }

        let bytes = resp
            .bytes()
            .await
            .map_err(|e| format!("ElevenLabs read error: {e}"))?;

        // Ensure parent directory exists
        if let Some(parent) = output_path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| format!("Failed to create voices dir: {e}"))?;
        }

        std::fs::write(output_path, &bytes)
            .map_err(|e| format!("Failed to write audio file: {e}"))?;

        Ok(output_path.to_string_lossy().to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn api_base_url() {
        assert_eq!(API_BASE, "https://api.elevenlabs.io");
    }

    #[test]
    fn deserialize_voices_response() {
        let json = r#"{"voices":[{"voice_id":"abc","name":"Rachel","category":"premade"}]}"#;
        let resp: VoicesResponse = serde_json::from_str(json).unwrap();
        assert_eq!(resp.voices.len(), 1);
        assert_eq!(resp.voices[0].name, "Rachel");
    }

    #[test]
    fn deserialize_voice_missing_category() {
        let json = r#"{"voices":[{"voice_id":"x","name":"Test"}]}"#;
        let resp: VoicesResponse = serde_json::from_str(json).unwrap();
        assert!(resp.voices[0].category.is_empty());
    }
}
