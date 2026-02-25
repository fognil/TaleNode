use uuid::Uuid;

use crate::model::node::VariableValue;

use super::playtest::PlaytestLogEntry;

/// Maximum number of saved checkpoints per session.
pub const MAX_CHECKPOINTS: usize = 20;

/// A saved snapshot of playtest runtime state.
pub struct PlaytestCheckpoint {
    pub id: usize,
    pub label: String,
    pub current_node: Option<Uuid>,
    pub log: Vec<PlaytestLogEntry>,
    pub variables: Vec<(String, VariableValue)>,
}

/// Show the checkpoint management UI section.
pub fn show_checkpoints_ui(
    ui: &mut egui::Ui,
    checkpoints: &mut [PlaytestCheckpoint],
    action: &mut Option<CheckpointAction>,
    can_save: bool,
    in_subgraph: bool,
) {
    ui.horizontal(|ui| {
        let save_enabled = can_save && !in_subgraph && checkpoints.len() < MAX_CHECKPOINTS;
        let save_btn = ui.add_enabled(save_enabled, egui::Button::new("Save Checkpoint"));
        if save_btn.clicked() {
            *action = Some(CheckpointAction::Save);
        }
        if in_subgraph {
            save_btn.on_disabled_hover_text("Cannot save checkpoints inside a SubGraph");
        } else if checkpoints.len() >= MAX_CHECKPOINTS {
            save_btn.on_disabled_hover_text(
                format!("Maximum {MAX_CHECKPOINTS} checkpoints reached"),
            );
        }
    });

    if checkpoints.is_empty() {
        return;
    }

    egui::CollapsingHeader::new(format!("Checkpoints ({})", checkpoints.len()))
        .default_open(true)
        .show(ui, |ui| {
            let mut to_load = None;
            let mut to_delete = None;

            for cp in checkpoints.iter() {
                ui.horizontal(|ui| {
                    ui.label(format!("#{}: {}", cp.id, cp.label));
                    if ui.small_button("Load").clicked() {
                        to_load = Some(cp.id);
                    }
                    if ui.small_button("Delete").clicked() {
                        to_delete = Some(cp.id);
                    }
                });
            }

            if let Some(id) = to_load {
                *action = Some(CheckpointAction::Load(id));
            }
            if let Some(id) = to_delete {
                *action = Some(CheckpointAction::Delete(id));
            }
        });
}

/// Actions returned from the checkpoint UI.
pub enum CheckpointAction {
    Save,
    Load(usize),
    Delete(usize),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn checkpoint_max_constant() {
        assert_eq!(MAX_CHECKPOINTS, 20);
    }

    #[test]
    fn checkpoint_struct_fields() {
        let cp = PlaytestCheckpoint {
            id: 1,
            label: "Before boss fight".to_string(),
            current_node: Some(Uuid::new_v4()),
            log: vec![PlaytestLogEntry {
                speaker: "Guard".to_string(),
                text: "Hello!".to_string(),
            }],
            variables: vec![("gold".to_string(), VariableValue::Int(50))],
        };
        assert_eq!(cp.id, 1);
        assert_eq!(cp.label, "Before boss fight");
        assert!(cp.current_node.is_some());
        assert_eq!(cp.log.len(), 1);
        assert_eq!(cp.variables.len(), 1);
    }
}
