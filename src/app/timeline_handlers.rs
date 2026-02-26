use super::TaleNodeApp;
use crate::model::timeline::{Timeline, TimelineAction, TimelineStep};

impl TaleNodeApp {
    pub(super) fn render_timeline_tab(&mut self, ui: &mut egui::Ui) {
        let pre_graph = self.graph.clone();
        egui::ScrollArea::vertical().show(ui, |ui| {
            let action = crate::ui::timeline_panel::show_timeline_panel(ui, &mut self.graph);
            match action {
                crate::ui::timeline_panel::TimelinePanelAction::AddTimeline => {
                    self.history.push_undo(pre_graph);
                    let tl = Timeline::new(format!(
                        "Timeline {}",
                        self.graph.timelines.len() + 1
                    ));
                    self.graph.timelines.push(tl);
                }
                crate::ui::timeline_panel::TimelinePanelAction::RemoveTimeline(id) => {
                    self.history.push_undo(pre_graph);
                    self.graph.timelines.retain(|t| t.id != id);
                }
                crate::ui::timeline_panel::TimelinePanelAction::EditTimeline => {
                    self.history.push_undo(pre_graph);
                }
                crate::ui::timeline_panel::TimelinePanelAction::AddStep(tl_id) => {
                    self.history.push_undo(pre_graph);
                    if let Some(tl) = self.graph.timelines.iter_mut().find(|t| t.id == tl_id) {
                        tl.steps.push(TimelineStep::new(TimelineAction::Wait {
                            seconds: 1.0,
                        }));
                    }
                }
                crate::ui::timeline_panel::TimelinePanelAction::RemoveStep(tl_id, step_id) => {
                    self.history.push_undo(pre_graph);
                    if let Some(tl) = self.graph.timelines.iter_mut().find(|t| t.id == tl_id) {
                        tl.steps.retain(|s| s.id != step_id);
                    }
                }
                crate::ui::timeline_panel::TimelinePanelAction::MoveStepUp(tl_id, idx) => {
                    self.history.push_undo(pre_graph);
                    if let Some(tl) = self.graph.timelines.iter_mut().find(|t| t.id == tl_id) {
                        if idx > 0 && idx < tl.steps.len() {
                            tl.steps.swap(idx, idx - 1);
                        }
                    }
                }
                crate::ui::timeline_panel::TimelinePanelAction::MoveStepDown(tl_id, idx) => {
                    self.history.push_undo(pre_graph);
                    if let Some(tl) = self.graph.timelines.iter_mut().find(|t| t.id == tl_id) {
                        if idx + 1 < tl.steps.len() {
                            tl.steps.swap(idx, idx + 1);
                        }
                    }
                }
                crate::ui::timeline_panel::TimelinePanelAction::None => {}
            }
        });
    }
}
