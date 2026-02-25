use std::time::Instant;

use super::async_runtime::AsyncResult;
use super::TaleNodeApp;

impl TaleNodeApp {
    /// Poll the async result channel and apply any completed results.
    /// Called once per frame from update(). Non-blocking via try_recv().
    pub(super) fn poll_async_results(&mut self) {
        while let Ok(result) = self.async_rx.try_recv() {
            match result {
                AsyncResult::TranslationDone {
                    locale,
                    translations,
                } => {
                    self.snapshot();
                    for (key, text) in translations {
                        self.graph
                            .locale
                            .set_translation(key, locale.clone(), text);
                    }
                    self.translation_in_progress = false;
                    self.status_message = Some((
                        format!("Translation to '{locale}' complete"),
                        Instant::now(),
                        false,
                    ));
                }
                AsyncResult::TranslationError(e) => {
                    self.translation_in_progress = false;
                    self.status_message =
                        Some((format!("Translation error: {e}"), Instant::now(), true));
                }
                AsyncResult::VoiceListLoaded(voices) => {
                    self.available_voices = voices;
                    self.status_message = Some((
                        "Voice list loaded".to_string(),
                        Instant::now(),
                        false,
                    ));
                }
                AsyncResult::VoiceGenerated {
                    node_id,
                    audio_path,
                } => {
                    self.snapshot();
                    if let Some(node) = self.graph.nodes.get_mut(&node_id) {
                        if let crate::model::node::NodeType::Dialogue(ref mut d) =
                            node.node_type
                        {
                            d.audio_clip = Some(audio_path);
                        }
                    }
                    self.voice_generation_in_progress = false;
                    self.status_message = Some((
                        "Voice generated".to_string(),
                        Instant::now(),
                        false,
                    ));
                }
                AsyncResult::VoiceError(e) => {
                    self.voice_generation_in_progress = false;
                    self.status_message =
                        Some((format!("Voice error: {e}"), Instant::now(), true));
                }
                AsyncResult::CollabMessage(_msg) => {
                    // Handled by collab module (commit 6)
                }
                AsyncResult::CollabError(e) => {
                    self.status_message =
                        Some((format!("Collab error: {e}"), Instant::now(), true));
                }
            }
        }
    }
}
