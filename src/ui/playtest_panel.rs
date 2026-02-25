use egui::Color32;
use uuid::Uuid;

use crate::model::graph::DialogueGraph;
use crate::model::node::NodeType;
use crate::scripting::evaluate_condition_expr;
use crate::scripting::interpolate::interpolate_text;

use super::playtest::PlaytestState;
use super::playtest_checkpoint::{show_checkpoints_ui, CheckpointAction};

/// Show the playtest panel UI.
pub fn show_playtest_panel(
    ui: &mut egui::Ui,
    state: &mut PlaytestState,
    graph: &DialogueGraph,
    selected_nodes: &mut Vec<Uuid>,
) {
    ui.horizontal(|ui| {
        ui.heading("Playtest");
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if state.running {
                if ui.button("Stop").clicked() {
                    state.stop();
                }
                if ui.button("Restart").clicked() {
                    state.start(graph);
                }
            } else if ui.button("Start").clicked() {
                state.start(graph);
            }
        });
    });
    ui.separator();

    if !state.running {
        ui.label("Press Start to begin playtest from the Start node.");
        return;
    }

    // Checkpoint UI
    let in_subgraph = state.in_subgraph();
    let mut cp_action = None;
    show_checkpoints_ui(
        ui,
        &mut state.checkpoints,
        &mut cp_action,
        state.running,
        in_subgraph,
    );
    match cp_action {
        Some(CheckpointAction::Save) => {
            let label = state.current_node_label(graph);
            state.save_checkpoint(label);
        }
        Some(CheckpointAction::Load(id)) => {
            state.load_checkpoint(id);
        }
        Some(CheckpointAction::Delete(id)) => {
            state.delete_checkpoint(id);
        }
        None => {}
    }
    ui.separator();

    // Highlight current node on canvas
    if let Some(id) = state.current_node {
        if !selected_nodes.contains(&id) {
            selected_nodes.clear();
            selected_nodes.push(id);
        }
    }

    // Dialogue log
    egui::ScrollArea::vertical()
        .max_height(200.0)
        .stick_to_bottom(true)
        .show(ui, |ui| {
            for entry in &state.log {
                ui.horizontal(|ui| {
                    ui.colored_label(Color32::from_rgb(150, 200, 255), &entry.speaker);
                    ui.label(&entry.text);
                });
            }
        });

    ui.separator();

    // Current node interaction
    let Some(node_id) = state.current_node else {
        ui.colored_label(Color32::from_rgb(255, 200, 100), "End of dialogue reached.");
        show_variables_section(ui, state);
        return;
    };

    let active = state.active_graph(graph);
    let Some(node) = active.nodes.get(&node_id) else {
        ui.colored_label(Color32::from_rgb(255, 100, 100), "Error: node not found.");
        return;
    };

    match &node.node_type {
        NodeType::Dialogue(data) => {
            let speaker = if data.speaker_name.is_empty() {
                "???".to_string()
            } else {
                data.speaker_name.clone()
            };
            let text = interpolate_text(&data.text, &state.variables);
            ui.colored_label(Color32::from_rgb(150, 200, 255), &speaker);
            ui.label(&text);
            if !data.emotion.is_empty() {
                ui.colored_label(
                    Color32::from_rgb(180, 180, 180),
                    format!("[{}]", data.emotion),
                );
            }
            ui.add_space(4.0);
            let mut should_advance = false;
            if ui.button("Continue >>").clicked() {
                should_advance = true;
            }
            if should_advance {
                state.advance_dialogue(graph);
            }
        }
        NodeType::Choice(data) => {
            if !data.prompt.is_empty() {
                let prompt = interpolate_text(&data.prompt, &state.variables);
                ui.label(&prompt);
            }
            ui.add_space(4.0);
            let mut chosen = None;
            for (i, choice) in data.choices.iter().enumerate() {
                let text = interpolate_text(&choice.text, &state.variables);
                let available = choice
                    .condition
                    .as_ref()
                    .map(|cond| evaluate_condition_expr(&state.variables, cond))
                    .unwrap_or(true);

                if available {
                    if ui.button(&text).clicked() {
                        chosen = Some(i);
                    }
                } else {
                    ui.add_enabled(false, egui::Button::new(&text));
                }
            }
            if let Some(idx) = chosen {
                state.make_choice(graph, idx);
            }
        }
        NodeType::End(data) => {
            let tag = if data.tag.is_empty() {
                "default"
            } else {
                &data.tag
            };
            ui.colored_label(
                Color32::from_rgb(255, 200, 100),
                format!("Dialogue ended: [{tag}]"),
            );
        }
        _ => {
            ui.label("Unexpected node type in playtest.");
        }
    }

    show_variables_section(ui, state);
}

/// Show current variable values in a collapsible section.
fn show_variables_section(ui: &mut egui::Ui, state: &PlaytestState) {
    let vars = state.variables.all_vars();
    if vars.is_empty() {
        return;
    }
    ui.add_space(8.0);
    egui::CollapsingHeader::new("Variables")
        .default_open(false)
        .show(ui, |ui| {
            egui::Grid::new("playtest_vars_grid")
                .num_columns(2)
                .spacing([8.0, 2.0])
                .show(ui, |ui| {
                    for (name, value) in &vars {
                        ui.colored_label(Color32::from_rgb(180, 220, 180), *name);
                        let display = crate::scripting::eval::eval_to_string(value);
                        ui.monospace(&display);
                        ui.end_row();
                    }
                });
        });
}
