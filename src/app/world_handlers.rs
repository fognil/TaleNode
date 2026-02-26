use super::TaleNodeApp;
use crate::model::world::{EntityCategory, EntityProperty, WorldEntity};

impl TaleNodeApp {
    pub(super) fn render_world_database_tab(&mut self, ui: &mut egui::Ui) {
        let pre_graph = self.graph.clone();
        egui::ScrollArea::vertical().show(ui, |ui| {
            let action = crate::ui::world_panel::show_world_panel(
                ui,
                &mut self.graph,
                &mut self.world_category_filter,
            );
            match action {
                crate::ui::world_panel::WorldPanelAction::AddEntity => {
                    self.history.push_undo(pre_graph);
                    let entity = WorldEntity::new(
                        format!("Entity {}", self.graph.world_entities.len() + 1),
                        EntityCategory::Item,
                    );
                    self.graph.world_entities.push(entity);
                }
                crate::ui::world_panel::WorldPanelAction::RemoveEntity(id) => {
                    self.history.push_undo(pre_graph);
                    self.graph.world_entities.retain(|e| e.id != id);
                }
                crate::ui::world_panel::WorldPanelAction::EditEntity => {
                    self.history.push_undo(pre_graph);
                }
                crate::ui::world_panel::WorldPanelAction::AddProperty(entity_id) => {
                    self.history.push_undo(pre_graph);
                    if let Some(e) =
                        self.graph.world_entities.iter_mut().find(|e| e.id == entity_id)
                    {
                        e.properties.push(EntityProperty {
                            key: String::new(),
                            value: String::new(),
                        });
                    }
                }
                crate::ui::world_panel::WorldPanelAction::RemoveProperty(entity_id, idx) => {
                    self.history.push_undo(pre_graph);
                    if let Some(e) =
                        self.graph.world_entities.iter_mut().find(|e| e.id == entity_id)
                    {
                        if idx < e.properties.len() {
                            e.properties.remove(idx);
                        }
                    }
                }
                crate::ui::world_panel::WorldPanelAction::None => {}
            }
        });
    }
}
