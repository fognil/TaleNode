use super::TaleNodeApp;

impl TaleNodeApp {
    pub(super) fn render_quests_tab(&mut self, ui: &mut egui::Ui) {
        let pre_graph = self.graph.clone();
        egui::ScrollArea::vertical().show(ui, |ui| {
            let action = crate::ui::quest_panel::show_quest_panel(ui, &mut self.graph);
            match action {
                crate::ui::quest_panel::QuestPanelAction::AddQuest => {
                    self.history.push_undo(pre_graph);
                    let quest = crate::model::quest::Quest::new(format!(
                        "Quest {}",
                        self.graph.quests.len() + 1
                    ));
                    self.graph.quests.push(quest);
                }
                crate::ui::quest_panel::QuestPanelAction::RemoveQuest(id) => {
                    self.history.push_undo(pre_graph);
                    self.graph.quests.retain(|q| q.id != id);
                }
                crate::ui::quest_panel::QuestPanelAction::AddObjective(quest_id) => {
                    self.history.push_undo(pre_graph);
                    if let Some(q) = self.graph.quests.iter_mut().find(|q| q.id == quest_id) {
                        let obj = crate::model::quest::Objective::new(format!(
                            "Objective {}",
                            q.objectives.len() + 1
                        ));
                        q.objectives.push(obj);
                    }
                }
                crate::ui::quest_panel::QuestPanelAction::RemoveObjective(quest_id, obj_id) => {
                    self.history.push_undo(pre_graph);
                    if let Some(q) = self.graph.quests.iter_mut().find(|q| q.id == quest_id) {
                        q.objectives.retain(|o| o.id != obj_id);
                    }
                }
                crate::ui::quest_panel::QuestPanelAction::EditQuest => {
                    self.history.push_undo(pre_graph);
                }
                crate::ui::quest_panel::QuestPanelAction::None => {}
            }
        });
    }
}
