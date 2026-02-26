use egui::Ui;
use uuid::Uuid;

use crate::model::graph::DialogueGraph;
use crate::model::quest::QuestStatus;

/// Actions returned by the quest panel.
pub enum QuestPanelAction {
    None,
    AddQuest,
    RemoveQuest(Uuid),
    AddObjective(Uuid),
    RemoveObjective(Uuid, Uuid),
    EditQuest,
}

/// Draw the quest/journal editor panel.
pub fn show_quest_panel(ui: &mut Ui, graph: &mut DialogueGraph) -> QuestPanelAction {
    let mut action = QuestPanelAction::None;

    if graph.quests.is_empty() {
        ui.label("No quests defined. Add one to get started.");
    }

    let mut remove_quest = None;
    let quest_ids: Vec<Uuid> = graph.quests.iter().map(|q| q.id).collect();
    for (qi, quest_id) in quest_ids.iter().enumerate() {
        let Some(quest) = graph.quests.get_mut(qi) else { continue };
        let header_text = if quest.name.is_empty() {
            format!("Quest {}", qi + 1)
        } else {
            quest.name.clone()
        };
        egui::CollapsingHeader::new(&header_text)
            .id_salt(quest_id)
            .default_open(true)
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label("Name:");
                    if ui.text_edit_singleline(&mut quest.name).gained_focus() {
                        action = QuestPanelAction::EditQuest;
                    }
                });
                ui.horizontal(|ui| {
                    ui.label("Description:");
                    if ui.text_edit_multiline(&mut quest.description).gained_focus() {
                        action = QuestPanelAction::EditQuest;
                    }
                });
                ui.horizontal(|ui| {
                    ui.label("Status:");
                    egui::ComboBox::from_id_salt(format!("quest_status_{quest_id}"))
                        .selected_text(quest.status.label())
                        .show_ui(ui, |ui| {
                            for s in QuestStatus::ALL {
                                if ui.selectable_value(&mut quest.status, s, s.label()).changed() {
                                    action = QuestPanelAction::EditQuest;
                                }
                            }
                        });
                });

                // Objectives
                ui.label("Objectives:");
                let mut remove_obj = None;
                for obj in quest.objectives.iter_mut() {
                    ui.horizontal(|ui| {
                        ui.checkbox(&mut obj.completed, "");
                        if ui.add(egui::TextEdit::singleline(&mut obj.text)
                            .desired_width(180.0)).gained_focus() {
                            action = QuestPanelAction::EditQuest;
                        }
                        ui.checkbox(&mut obj.optional, "Optional");
                        if ui.small_button("X").clicked() {
                            remove_obj = Some(obj.id);
                        }
                    });
                }
                if let Some(obj_id) = remove_obj {
                    action = QuestPanelAction::RemoveObjective(*quest_id, obj_id);
                }
                if ui.small_button("+ Objective").clicked() {
                    action = QuestPanelAction::AddObjective(*quest_id);
                }

                ui.separator();
                if ui.small_button("Delete Quest").clicked() {
                    remove_quest = Some(qi);
                }
            });
    }

    if let Some(qi) = remove_quest {
        action = QuestPanelAction::RemoveQuest(graph.quests[qi].id);
    }

    ui.add_space(4.0);
    if ui.button("+ Add Quest").clicked() {
        action = QuestPanelAction::AddQuest;
    }

    action
}
