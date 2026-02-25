use std::path::PathBuf;
use uuid::Uuid;

use super::TaleNodeApp;
use crate::model::node::NodeType;

impl TaleNodeApp {
    pub(super) fn start_fetch_voices(&mut self) {
        let api_key = match &self.settings.elevenlabs_api_key {
            Some(k) if !k.is_empty() => k.clone(),
            _ => {
                self.status_message = Some((
                    "ElevenLabs API key not set — open Settings".to_string(),
                    std::time::Instant::now(),
                    true,
                ));
                return;
            }
        };

        let tx = self.async_tx.clone();
        self.tokio_runtime.spawn(async move {
            let client =
                crate::integrations::elevenlabs::ElevenLabsClient::new(api_key);
            match client.list_voices().await {
                Ok(voices) => {
                    let _ = tx.send(
                        crate::app::async_runtime::AsyncResult::VoiceListLoaded(
                            voices,
                        ),
                    );
                }
                Err(e) => {
                    let _ = tx.send(
                        crate::app::async_runtime::AsyncResult::VoiceError(e),
                    );
                }
            }
        });

        self.status_message = Some((
            "Fetching voice list...".to_string(),
            std::time::Instant::now(),
            false,
        ));
    }

    pub(super) fn start_generate_voice(&mut self, node_id: Uuid) {
        let Some(node) = self.graph.nodes.get(&node_id) else {
            return;
        };
        let NodeType::Dialogue(ref data) = node.node_type else {
            return;
        };

        let voice_id = data
            .speaker_id
            .and_then(|sid| self.graph.characters.iter().find(|c| c.id == sid))
            .and_then(|c| c.voice_id.clone());

        let Some(voice_id) = voice_id else {
            self.status_message = Some((
                "No voice assigned to character".to_string(),
                std::time::Instant::now(),
                true,
            ));
            return;
        };

        if data.text.trim().is_empty() {
            self.status_message = Some((
                "Dialogue text is empty".to_string(),
                std::time::Instant::now(),
                true,
            ));
            return;
        }

        let api_key = match &self.settings.elevenlabs_api_key {
            Some(k) if !k.is_empty() => k.clone(),
            _ => {
                self.status_message = Some((
                    "ElevenLabs API key not set — open Settings".to_string(),
                    std::time::Instant::now(),
                    true,
                ));
                return;
            }
        };

        let text = data.text.clone();
        let readable_id = format!("dlg_{}", &node_id.to_string()[..8]);
        let output_dir = self
            .project_path
            .as_ref()
            .and_then(|p| p.parent())
            .map(|d| d.join("voices"))
            .unwrap_or_else(|| PathBuf::from("voices"));
        let output_path = output_dir.join(format!("{readable_id}.mp3"));

        self.voice_generation_in_progress = true;
        let tx = self.async_tx.clone();

        self.tokio_runtime.spawn(async move {
            let client =
                crate::integrations::elevenlabs::ElevenLabsClient::new(api_key);
            match client.generate_voice(&voice_id, &text, &output_path).await {
                Ok(path) => {
                    let _ = tx.send(
                        crate::app::async_runtime::AsyncResult::VoiceGenerated {
                            node_id,
                            audio_path: path,
                        },
                    );
                }
                Err(e) => {
                    let _ = tx.send(
                        crate::app::async_runtime::AsyncResult::VoiceError(e),
                    );
                }
            }
        });

        self.status_message = Some((
            "Generating voice...".to_string(),
            std::time::Instant::now(),
            false,
        ));
    }

    pub(super) fn start_generate_all_voices(&mut self) {
        // Find first dialogue node that has a voice assigned but no audio
        let node_id = self.graph.nodes.values().find_map(|node| {
            if let NodeType::Dialogue(ref data) = node.node_type {
                if data.audio_clip.is_some() || data.text.trim().is_empty() {
                    return None;
                }
                let has_voice = data
                    .speaker_id
                    .and_then(|sid| {
                        self.graph.characters.iter().find(|c| c.id == sid)
                    })
                    .is_some_and(|c| c.voice_id.is_some());
                if has_voice {
                    return Some(node.id);
                }
            }
            None
        });

        if let Some(id) = node_id {
            self.start_generate_voice(id);
        } else {
            self.status_message = Some((
                "No pending voice generation".to_string(),
                std::time::Instant::now(),
                false,
            ));
        }
    }
}
