use super::TaleNodeApp;

impl TaleNodeApp {
    pub(super) fn render_barks_tab(&mut self, ui: &mut egui::Ui) {
        let pre_graph = self.graph.clone();
        egui::ScrollArea::vertical().show(ui, |ui| {
            let action = crate::ui::bark_panel::show_bark_panel(
                ui,
                &mut self.graph,
                &mut self.bark_selected_character,
            );
            match action {
                crate::ui::bark_panel::BarkPanelAction::AddBark(char_id) => {
                    self.history.push_undo(pre_graph);
                    let bark = crate::model::bark::BarkLine::new(format!(
                        "Bark {}",
                        self.graph
                            .barks
                            .get(&char_id)
                            .map_or(1, |b| b.len() + 1)
                    ));
                    self.graph.barks.entry(char_id).or_default().push(bark);
                }
                crate::ui::bark_panel::BarkPanelAction::RemoveBark(char_id, bark_id) => {
                    self.history.push_undo(pre_graph);
                    if let Some(barks) = self.graph.barks.get_mut(&char_id) {
                        barks.retain(|b| b.id != bark_id);
                    }
                }
                crate::ui::bark_panel::BarkPanelAction::EditBark => {
                    self.history.push_undo(pre_graph);
                }
                crate::ui::bark_panel::BarkPanelAction::None => {}
            }
        });
    }
}
