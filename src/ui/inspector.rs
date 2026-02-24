use egui::Ui;
use uuid::Uuid;

use crate::model::graph::DialogueGraph;
use crate::model::node::{
    CompareOp, EventAction, EventActionType, NodeType, VariableValue,
};

/// Deferred mutation to apply after the inspector UI pass.
enum DeferredAction {
    None,
    AddChoice,
    RemoveChoice(usize),
    AddRandomBranch,
    RemoveRandomBranch(usize),
}

/// Draw the inspector panel for the currently selected node.
pub fn show_inspector(ui: &mut Ui, graph: &mut DialogueGraph, selected: Uuid) {
    let mut deferred = DeferredAction::None;

    // First pass: draw UI and collect deferred actions
    if let Some(node) = graph.nodes.get_mut(&selected) {
        ui.heading(node.title());
        ui.separator();

        ui.horizontal(|ui| {
            ui.label("ID:");
            ui.monospace(node.id.to_string().chars().take(8).collect::<String>());
        });
        ui.add_space(8.0);

        match &mut node.node_type {
            NodeType::Start => {
                ui.label("Start node — entry point of the dialogue.");
            }

            NodeType::Dialogue(data) => {
                show_dialogue_inspector(ui, data);
            }

            NodeType::Choice(data) => {
                deferred = show_choice_inspector(ui, data);
            }

            NodeType::Condition(data) => {
                show_condition_inspector(ui, data);
            }

            NodeType::Event(data) => {
                show_event_inspector(ui, data);
            }

            NodeType::Random(data) => {
                deferred = show_random_inspector(ui, data);
            }

            NodeType::End(data) => {
                ui.label("Tag:");
                ui.text_edit_singleline(&mut data.tag);
                ui.add_space(4.0);
                ui.colored_label(
                    egui::Color32::from_rgb(140, 140, 140),
                    "Common: good_ending, bad_ending, continue",
                );
            }
        }
    }

    // Second pass: apply deferred mutations that need &mut Node (not &mut NodeType)
    match deferred {
        DeferredAction::AddChoice => {
            if let Some(node) = graph.nodes.get_mut(&selected) {
                node.add_choice();
                // Sync port label with choice text
                sync_choice_labels(node);
            }
        }
        DeferredAction::RemoveChoice(idx) => {
            if let Some(node) = graph.nodes.get_mut(&selected) {
                // Remove the connection for this port before removing the port
                if let Some(port_id) = node.outputs.get(idx).map(|p| p.id) {
                    graph.connections.retain(|c| c.from_port != port_id);
                }
                if let Some(node) = graph.nodes.get_mut(&selected) {
                    node.remove_choice(idx);
                    sync_choice_labels(node);
                }
            }
        }
        DeferredAction::AddRandomBranch => {
            if let Some(node) = graph.nodes.get_mut(&selected) {
                node.add_random_branch();
                sync_random_labels(node);
            }
        }
        DeferredAction::RemoveRandomBranch(idx) => {
            if let Some(node) = graph.nodes.get_mut(&selected) {
                if let Some(port_id) = node.outputs.get(idx).map(|p| p.id) {
                    graph.connections.retain(|c| c.from_port != port_id);
                }
                if let Some(node) = graph.nodes.get_mut(&selected) {
                    node.remove_random_branch(idx);
                    sync_random_labels(node);
                }
            }
        }
        DeferredAction::None => {}
    }
}

fn show_dialogue_inspector(
    ui: &mut Ui,
    data: &mut crate::model::node::DialogueData,
) {
    ui.label("Speaker:");
    ui.text_edit_singleline(&mut data.speaker_name);

    ui.add_space(4.0);
    ui.label("Text:");
    ui.add(egui::TextEdit::multiline(&mut data.text).desired_rows(4));

    ui.add_space(4.0);
    ui.label("Emotion:");
    let emotions = ["neutral", "happy", "sad", "angry", "surprised", "scared"];
    egui::ComboBox::from_id_salt("emotion_combo")
        .selected_text(&data.emotion)
        .show_ui(ui, |ui| {
            for e in &emotions {
                if ui.selectable_label(data.emotion == *e, *e).clicked() {
                    data.emotion = e.to_string();
                }
            }
        });

    ui.add_space(4.0);
    ui.label("Audio clip:");
    let mut audio = data.audio_clip.clone().unwrap_or_default();
    if ui.text_edit_singleline(&mut audio).changed() {
        data.audio_clip = if audio.is_empty() { None } else { Some(audio) };
    }
}

fn show_choice_inspector(
    ui: &mut Ui,
    data: &mut crate::model::node::ChoiceData,
) -> DeferredAction {
    ui.label("Prompt:");
    ui.text_edit_singleline(&mut data.prompt);

    ui.add_space(8.0);
    ui.label("Choices:");
    ui.separator();

    let mut remove_idx = None;
    let choice_count = data.choices.len();
    for (i, choice) in data.choices.iter_mut().enumerate() {
        ui.horizontal(|ui| {
            ui.label(format!("{}.", i + 1));
            ui.text_edit_singleline(&mut choice.text);
            if choice_count > 1 && ui.small_button("X").clicked() {
                remove_idx = Some(i);
            }
        });
    }

    if let Some(idx) = remove_idx {
        return DeferredAction::RemoveChoice(idx);
    }

    if ui.button("+ Add Choice").clicked() {
        return DeferredAction::AddChoice;
    }

    DeferredAction::None
}

fn show_condition_inspector(
    ui: &mut Ui,
    data: &mut crate::model::node::ConditionData,
) {
    ui.label("Variable:");
    ui.text_edit_singleline(&mut data.variable_name);

    ui.add_space(4.0);
    ui.label("Operator:");
    let ops = [
        (CompareOp::Eq, "=="),
        (CompareOp::Neq, "!="),
        (CompareOp::Gt, ">"),
        (CompareOp::Lt, "<"),
        (CompareOp::Gte, ">="),
        (CompareOp::Lte, "<="),
        (CompareOp::Contains, "contains"),
    ];
    let current_label = ops
        .iter()
        .find(|(op, _)| *op == data.operator)
        .map(|(_, l)| *l)
        .unwrap_or("==");
    egui::ComboBox::from_id_salt("op_combo")
        .selected_text(current_label)
        .show_ui(ui, |ui| {
            for (op, label) in &ops {
                if ui.selectable_label(data.operator == *op, *label).clicked() {
                    data.operator = *op;
                }
            }
        });

    ui.add_space(4.0);
    ui.label("Value:");
    show_variable_value_editor(ui, &mut data.value, "cond_val");
}

fn show_event_inspector(
    ui: &mut Ui,
    data: &mut crate::model::node::EventData,
) {
    ui.label("Actions:");
    ui.separator();

    let mut remove_idx = None;
    for (i, action) in data.actions.iter_mut().enumerate() {
        ui.group(|ui| {
            ui.horizontal(|ui| {
                ui.label(format!("#{}", i + 1));
                if ui.small_button("X").clicked() {
                    remove_idx = Some(i);
                }
            });
            ui.label("Key:");
            ui.text_edit_singleline(&mut action.key);
            ui.label("Value:");
            show_variable_value_editor(ui, &mut action.value, &format!("evt_val_{i}"));
        });
    }

    if let Some(idx) = remove_idx {
        data.actions.remove(idx);
    }

    if ui.button("+ Add Action").clicked() {
        data.actions.push(EventAction {
            action_type: EventActionType::SetVariable,
            key: String::new(),
            value: VariableValue::Bool(false),
        });
    }
}

fn show_random_inspector(
    ui: &mut Ui,
    data: &mut crate::model::node::RandomData,
) -> DeferredAction {
    ui.label("Branches:");
    ui.separator();

    let mut remove_idx = None;
    let branch_count = data.branches.len();
    for (i, branch) in data.branches.iter_mut().enumerate() {
        ui.horizontal(|ui| {
            ui.label(format!("{}.", i + 1));
            let mut pct = branch.weight * 100.0;
            if ui
                .add(egui::DragValue::new(&mut pct).range(0.0..=100.0).suffix("%"))
                .changed()
            {
                branch.weight = pct / 100.0;
            }
            if branch_count > 1 && ui.small_button("X").clicked() {
                remove_idx = Some(i);
            }
        });
    }

    // Total weight indicator
    let total: f32 = data.branches.iter().map(|b| b.weight).sum();
    let total_pct = total * 100.0;
    if (total_pct - 100.0).abs() > 0.1 {
        ui.colored_label(
            egui::Color32::from_rgb(255, 100, 100),
            format!("Total: {total_pct:.0}% (should be 100%)"),
        );
    } else {
        ui.label(format!("Total: {total_pct:.0}%"));
    }

    if let Some(idx) = remove_idx {
        return DeferredAction::RemoveRandomBranch(idx);
    }

    if ui.button("+ Add Branch").clicked() {
        return DeferredAction::AddRandomBranch;
    }

    DeferredAction::None
}

/// Editor widget for a VariableValue (bool/int/float/text selector + value).
fn show_variable_value_editor(ui: &mut Ui, value: &mut VariableValue, id: &str) {
    let type_labels = ["Bool", "Int", "Float", "Text"];
    let current_type = match value {
        VariableValue::Bool(_) => 0,
        VariableValue::Int(_) => 1,
        VariableValue::Float(_) => 2,
        VariableValue::Text(_) => 3,
    };

    let mut selected = current_type;
    egui::ComboBox::from_id_salt(format!("{id}_type"))
        .selected_text(type_labels[selected])
        .show_ui(ui, |ui| {
            for (i, label) in type_labels.iter().enumerate() {
                if ui.selectable_label(selected == i, *label).clicked() {
                    selected = i;
                }
            }
        });

    if selected != current_type {
        *value = match selected {
            0 => VariableValue::Bool(false),
            1 => VariableValue::Int(0),
            2 => VariableValue::Float(0.0),
            _ => VariableValue::Text(String::new()),
        };
    }

    match value {
        VariableValue::Bool(b) => {
            ui.checkbox(b, "");
        }
        VariableValue::Int(i) => {
            ui.add(egui::DragValue::new(i));
        }
        VariableValue::Float(f) => {
            ui.add(egui::DragValue::new(f).speed(0.1));
        }
        VariableValue::Text(s) => {
            ui.text_edit_singleline(s);
        }
    }
}

/// Sync output port labels with choice text.
fn sync_choice_labels(node: &mut crate::model::node::Node) {
    if let NodeType::Choice(data) = &node.node_type {
        for (i, choice) in data.choices.iter().enumerate() {
            if let Some(port) = node.outputs.get_mut(i) {
                port.label = choice.text.clone();
            }
        }
    }
}

/// Sync output port labels with random branch weights.
fn sync_random_labels(node: &mut crate::model::node::Node) {
    if let NodeType::Random(data) = &node.node_type {
        for (i, branch) in data.branches.iter().enumerate() {
            if let Some(port) = node.outputs.get_mut(i) {
                port.label = format!("{:.0}%", branch.weight * 100.0);
            }
        }
    }
}
