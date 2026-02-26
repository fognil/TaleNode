use egui::Ui;
use uuid::Uuid;

use crate::model::graph::DialogueGraph;
use crate::model::timeline::TimelineAction;

/// Actions returned by the timeline panel.
pub enum TimelinePanelAction {
    None,
    AddTimeline,
    RemoveTimeline(Uuid),
    EditTimeline,
    AddStep(Uuid),
    RemoveStep(Uuid, Uuid),
    MoveStepUp(Uuid, usize),
    MoveStepDown(Uuid, usize),
}

/// Draw the timeline/cutscene sequencer panel.
pub fn show_timeline_panel(ui: &mut Ui, graph: &mut DialogueGraph) -> TimelinePanelAction {
    let mut action = TimelinePanelAction::None;

    if graph.timelines.is_empty() {
        ui.label("No timelines. Add one to get started.");
    }

    let mut remove_timeline = None;
    let timeline_ids: Vec<Uuid> = graph.timelines.iter().map(|t| t.id).collect();
    for (ti, timeline_id) in timeline_ids.iter().enumerate() {
        let Some(timeline) = graph.timelines.get_mut(ti) else { continue };
        let header = if timeline.name.is_empty() {
            format!("Timeline {}", ti + 1)
        } else {
            timeline.name.clone()
        };

        egui::CollapsingHeader::new(&header)
            .id_salt(timeline_id)
            .default_open(true)
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label("Name:");
                    if ui.text_edit_singleline(&mut timeline.name).gained_focus() {
                        action = TimelinePanelAction::EditTimeline;
                    }
                });
                ui.horizontal(|ui| {
                    ui.label("Description:");
                    if ui.text_edit_singleline(&mut timeline.description).gained_focus() {
                        action = TimelinePanelAction::EditTimeline;
                    }
                });
                ui.checkbox(&mut timeline.loop_playback, "Loop");

                // Steps
                ui.label("Steps:");
                let step_count = timeline.steps.len();
                let mut remove_step = None;
                let mut move_up = None;
                let mut move_down = None;
                for (si, step) in timeline.steps.iter_mut().enumerate() {
                    ui.horizontal(|ui| {
                        ui.label(format!("{}.", si + 1));
                        let current_label = step.action.label().to_string();
                        egui::ComboBox::from_id_salt(format!("step_type_{timeline_id}_{si}"))
                            .selected_text(&current_label)
                            .width(90.0)
                            .show_ui(ui, |ui| {
                                for &label in TimelineAction::LABELS {
                                    if ui.selectable_label(current_label == label, label).clicked()
                                        && current_label != label
                                    {
                                        step.action = TimelineAction::from_label(label);
                                        action = TimelinePanelAction::EditTimeline;
                                    }
                                }
                            });
                        show_action_fields(ui, &mut step.action, &mut action);
                        ui.add(
                            egui::DragValue::new(&mut step.delay)
                                .range(0.0..=60.0)
                                .speed(0.1)
                                .prefix("delay: "),
                        );
                        if si > 0 && ui.small_button("^").clicked() {
                            move_up = Some(si);
                        }
                        if si + 1 < step_count && ui.small_button("v").clicked() {
                            move_down = Some(si);
                        }
                        if ui.small_button("X").clicked() {
                            remove_step = Some(step.id);
                        }
                    });
                }
                if let Some(step_id) = remove_step {
                    action = TimelinePanelAction::RemoveStep(*timeline_id, step_id);
                }
                if let Some(idx) = move_up {
                    action = TimelinePanelAction::MoveStepUp(*timeline_id, idx);
                }
                if let Some(idx) = move_down {
                    action = TimelinePanelAction::MoveStepDown(*timeline_id, idx);
                }
                if ui.small_button("+ Step").clicked() {
                    action = TimelinePanelAction::AddStep(*timeline_id);
                }

                ui.separator();
                if ui.small_button("Delete Timeline").clicked() {
                    remove_timeline = Some(ti);
                }
            });
    }

    if let Some(ti) = remove_timeline {
        action = TimelinePanelAction::RemoveTimeline(graph.timelines[ti].id);
    }

    ui.add_space(4.0);
    if ui.button("+ Add Timeline").clicked() {
        action = TimelinePanelAction::AddTimeline;
    }

    action
}

fn show_action_fields(
    ui: &mut Ui,
    action: &mut TimelineAction,
    panel_action: &mut TimelinePanelAction,
) {
    match action {
        TimelineAction::Camera { target, duration } => {
            if ui.add(egui::TextEdit::singleline(target).desired_width(60.0)).gained_focus() {
                *panel_action = TimelinePanelAction::EditTimeline;
            }
            ui.add(egui::DragValue::new(duration).range(0.0..=30.0).speed(0.1).prefix("dur: "));
        }
        TimelineAction::Animation { target, clip } => {
            if ui.add(egui::TextEdit::singleline(target).desired_width(60.0)).gained_focus() {
                *panel_action = TimelinePanelAction::EditTimeline;
            }
            if ui.add(egui::TextEdit::singleline(clip).desired_width(60.0)).gained_focus() {
                *panel_action = TimelinePanelAction::EditTimeline;
            }
        }
        TimelineAction::Audio { clip, volume } => {
            if ui.add(egui::TextEdit::singleline(clip).desired_width(80.0)).gained_focus() {
                *panel_action = TimelinePanelAction::EditTimeline;
            }
            ui.add(egui::DragValue::new(volume).range(0.0..=1.0).speed(0.01).prefix("vol: "));
        }
        TimelineAction::Wait { seconds } => {
            ui.add(egui::DragValue::new(seconds).range(0.0..=60.0).speed(0.1).prefix("sec: "));
        }
        TimelineAction::SetVariable { key, value } => {
            if ui.add(egui::TextEdit::singleline(key).desired_width(60.0)).gained_focus() {
                *panel_action = TimelinePanelAction::EditTimeline;
            }
            if ui.add(egui::TextEdit::singleline(value).desired_width(60.0)).gained_focus() {
                *panel_action = TimelinePanelAction::EditTimeline;
            }
        }
        TimelineAction::Custom { action_type, data } => {
            if ui.add(egui::TextEdit::singleline(action_type).desired_width(60.0)).gained_focus() {
                *panel_action = TimelinePanelAction::EditTimeline;
            }
            if ui.add(egui::TextEdit::singleline(data).desired_width(60.0)).gained_focus() {
                *panel_action = TimelinePanelAction::EditTimeline;
            }
        }
        TimelineAction::Dialogue { .. } => {
            ui.label("(linked node)");
        }
    }
}
