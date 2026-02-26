use uuid::Uuid;

use super::TaleNodeApp;
use crate::model::node::NodeType;

impl TaleNodeApp {
    pub(super) fn render_writing_assistant_tab(&mut self, ui: &mut egui::Ui) {
        let action = crate::ui::writing_panel::show_writing_panel(
            ui,
            &self.graph,
            &self.selected_nodes,
            self.writing_in_progress,
            &self.writing_suggestions,
            &self.writing_tone_report,
            &mut self.writing_instruction,
            &mut self.writing_choice_count,
        );
        match action {
            crate::ui::writing_panel::WritingAction::SuggestDialogue {
                node_id,
                instruction,
            } => {
                self.start_suggest_dialogue(node_id, instruction);
            }
            crate::ui::writing_panel::WritingAction::GenerateChoices {
                node_id,
                count,
            } => {
                self.start_generate_choices(node_id, count);
            }
            crate::ui::writing_panel::WritingAction::CheckTone { node_id } => {
                self.start_check_tone(node_id);
            }
            crate::ui::writing_panel::WritingAction::ApplyDialogue { node_id, text } => {
                self.snapshot();
                if let Some(node) = self.graph.nodes.get_mut(&node_id) {
                    if let NodeType::Dialogue(ref mut d) = node.node_type {
                        d.text = text;
                    }
                }
            }
            crate::ui::writing_panel::WritingAction::ApplyChoices { node_id, choices } => {
                self.apply_generated_choices(node_id, choices);
            }
            crate::ui::writing_panel::WritingAction::None => {}
        }
    }

    fn start_suggest_dialogue(&mut self, node_id: Uuid, instruction: String) {
        let Some(node) = self.graph.nodes.get(&node_id) else {
            return;
        };
        let NodeType::Dialogue(ref data) = node.node_type else {
            return;
        };
        let key = match &self.settings.ai_api_key {
            Some(k) if !k.is_empty() => k.clone(),
            _ => {
                self.status_message = Some((
                    "AI API key not set — open Settings".to_string(),
                    std::time::Instant::now(),
                    true,
                ));
                return;
            }
        };

        let speaker = data.speaker_name.clone();
        let emotion = data.emotion.clone();
        let text = data.text.clone();
        let provider = self.settings.ai_provider;
        let base_url = self.settings.ai_base_url.clone();
        let model = self.settings.ai_model.clone();

        self.writing_in_progress = true;
        let tx = self.async_tx.clone();

        self.tokio_runtime.spawn(async move {
            let client = crate::integrations::ai_writing::AIWritingClient::new(
                provider, key, base_url, model,
            );
            match client
                .suggest_dialogue(&speaker, &emotion, &text, &instruction)
                .await
            {
                Ok(suggestions) => {
                    let _ = tx.send(
                        crate::app::async_runtime::AsyncResult::WritingSuggestionsReady {
                            node_id,
                            suggestions,
                        },
                    );
                }
                Err(e) => {
                    let _ = tx.send(
                        crate::app::async_runtime::AsyncResult::WritingError(e),
                    );
                }
            }
        });

        self.status_message = Some((
            "Generating suggestions...".to_string(),
            std::time::Instant::now(),
            false,
        ));
    }

    fn start_generate_choices(&mut self, node_id: Uuid, count: usize) {
        let Some(node) = self.graph.nodes.get(&node_id) else {
            return;
        };
        let NodeType::Choice(ref data) = node.node_type else {
            return;
        };
        let key = match &self.settings.ai_api_key {
            Some(k) if !k.is_empty() => k.clone(),
            _ => {
                self.status_message = Some((
                    "AI API key not set — open Settings".to_string(),
                    std::time::Instant::now(),
                    true,
                ));
                return;
            }
        };

        let prompt = data.prompt.clone();
        let context = data
            .choices
            .iter()
            .map(|c| c.text.as_str())
            .collect::<Vec<_>>()
            .join(", ");
        let provider = self.settings.ai_provider;
        let base_url = self.settings.ai_base_url.clone();
        let model = self.settings.ai_model.clone();

        self.writing_in_progress = true;
        let tx = self.async_tx.clone();

        self.tokio_runtime.spawn(async move {
            let client = crate::integrations::ai_writing::AIWritingClient::new(
                provider, key, base_url, model,
            );
            match client.generate_choices(&prompt, &context, count).await {
                Ok(suggestions) => {
                    let _ = tx.send(
                        crate::app::async_runtime::AsyncResult::WritingSuggestionsReady {
                            node_id,
                            suggestions,
                        },
                    );
                }
                Err(e) => {
                    let _ = tx.send(
                        crate::app::async_runtime::AsyncResult::WritingError(e),
                    );
                }
            }
        });

        self.status_message = Some((
            "Generating choice options...".to_string(),
            std::time::Instant::now(),
            false,
        ));
    }

    fn start_check_tone(&mut self, node_id: Uuid) {
        let Some(node) = self.graph.nodes.get(&node_id) else {
            return;
        };
        let NodeType::Dialogue(ref data) = node.node_type else {
            return;
        };
        let key = match &self.settings.ai_api_key {
            Some(k) if !k.is_empty() => k.clone(),
            _ => {
                self.status_message = Some((
                    "AI API key not set — open Settings".to_string(),
                    std::time::Instant::now(),
                    true,
                ));
                return;
            }
        };

        let speaker = data.speaker_name.clone();
        let emotion = data.emotion.clone();
        let text = data.text.clone();
        let provider = self.settings.ai_provider;
        let base_url = self.settings.ai_base_url.clone();
        let model = self.settings.ai_model.clone();

        self.writing_in_progress = true;
        let tx = self.async_tx.clone();

        self.tokio_runtime.spawn(async move {
            let client = crate::integrations::ai_writing::AIWritingClient::new(
                provider, key, base_url, model,
            );
            match client.check_tone(&speaker, &emotion, &text).await {
                Ok(report) => {
                    let _ = tx.send(
                        crate::app::async_runtime::AsyncResult::ToneCheckReady {
                            node_id,
                            report,
                        },
                    );
                }
                Err(e) => {
                    let _ = tx.send(
                        crate::app::async_runtime::AsyncResult::WritingError(e),
                    );
                }
            }
        });

        self.status_message = Some((
            "Checking tone...".to_string(),
            std::time::Instant::now(),
            false,
        ));
    }

    fn apply_generated_choices(&mut self, node_id: Uuid, choices: Vec<String>) {
        self.snapshot();
        let Some(node) = self.graph.nodes.get_mut(&node_id) else {
            return;
        };
        let NodeType::Choice(ref mut data) = node.node_type else {
            return;
        };

        // Update existing choice texts and add/remove as needed
        for (i, text) in choices.iter().enumerate() {
            if i < data.choices.len() {
                data.choices[i].text = text.clone();
                if i < node.outputs.len() {
                    node.outputs[i].label = text.clone();
                }
            }
        }

        // Add new choices if AI generated more than existing
        for text in choices.iter().skip(data.choices.len()) {
            let option = crate::model::node::ChoiceOption {
                id: uuid::Uuid::new_v4(),
                text: text.clone(),
                condition: None,
            };
            data.choices.push(option);
            node.outputs
                .push(crate::model::port::Port::output_with_label(text));
        }
    }
}
