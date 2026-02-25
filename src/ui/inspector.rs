use egui::Ui;
use uuid::Uuid;

use crate::model::graph::DialogueGraph;
use crate::model::node::NodeType;
use crate::model::review::ReviewStatus;

use super::inspector_widgets::{
    show_condition_inspector, show_dialogue_inspector, show_event_inspector,
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
/// Returns `true` when an undo-worthy event starts (caller should snapshot).
pub fn show_inspector(
    ui: &mut Ui,
    graph: &mut DialogueGraph,
    selected: Uuid,
    new_tag_text: &mut String,
) -> bool {
    let mut deferred = DeferredAction::None;
    let mut snapshot_needed = false;

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
                let characters = graph.characters.clone();
                if show_dialogue_inspector(ui, data, &characters) {
                    snapshot_needed = true;
                }
            }

            NodeType::Choice(data) => {
                let (action, changed) = show_choice_inspector(ui, data);
                deferred = action;
                if changed {
                    snapshot_needed = true;
                }
            }

            NodeType::Condition(data) => {
                if show_condition_inspector(ui, data) {
                    snapshot_needed = true;
                }
            }

            NodeType::Event(data) => {
                if show_event_inspector(ui, data) {
                    snapshot_needed = true;
                }
            }

            NodeType::Random(data) => {
                let (action, changed) = show_random_inspector(ui, data);
                deferred = action;
                if changed {
                    snapshot_needed = true;
                }
            }

            NodeType::End(data) => {
                if ui.text_edit_singleline(&mut data.tag).gained_focus() {
                    snapshot_needed = true;
                }
            }

            NodeType::SubGraph(data) => {
                ui.label("Name:");
                if ui.text_edit_singleline(&mut data.name).gained_focus() {
                    snapshot_needed = true;
                }
                ui.add_space(8.0);
                ui.label(format!(
                    "Child nodes: {}  Connections: {}",
                    data.child_graph.nodes.len(),
                    data.child_graph.connections.len(),
                ));
                ui.add_space(4.0);
                ui.label("Double-click to enter sub-graph.");
            }
        }
    }

    // Second pass: apply deferred mutations that need &mut Node (not &mut NodeType)
    match deferred {
        DeferredAction::AddChoice => {
            snapshot_needed = true;
            if let Some(node) = graph.nodes.get_mut(&selected) {
                node.add_choice();
                sync_choice_labels(node);
            }
        }
        DeferredAction::RemoveChoice(idx) => {
            snapshot_needed = true;
            if let Some(node) = graph.nodes.get_mut(&selected) {
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
            snapshot_needed = true;
            if let Some(node) = graph.nodes.get_mut(&selected) {
                node.add_random_branch();
                sync_random_labels(node);
            }
        }
        DeferredAction::RemoveRandomBranch(idx) => {
            snapshot_needed = true;
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

    // Tags section
    ui.add_space(8.0);
    ui.separator();
    ui.heading("Tags");
    let tags = graph.get_tags(selected).to_vec();
    ui.horizontal_wrapped(|ui| {
        let mut tag_to_remove = None;
        for tag in &tags {
            ui.label(tag.as_str());
            if ui.small_button("x").clicked() {
                tag_to_remove = Some(tag.clone());
                snapshot_needed = true;
            }
        }
        if let Some(tag) = tag_to_remove {
            graph.remove_tag(selected, &tag);
        }
    });
    ui.horizontal(|ui| {
        ui.text_edit_singleline(new_tag_text);
        if ui.button("+").clicked() && !new_tag_text.trim().is_empty() {
            graph.add_tag(selected, new_tag_text.trim().to_string());
            *new_tag_text = String::new();
            snapshot_needed = true;
        }
    });

    // Review section
    ui.add_space(8.0);
    ui.separator();
    ui.heading("Review");

    let current_status = graph.get_review_status(selected);
    let mut selected_idx = ReviewStatus::all()
        .iter()
        .position(|s| *s == current_status)
        .unwrap_or(0);
    egui::ComboBox::from_id_salt("review_status_combo")
        .selected_text(current_status.label())
        .show_ui(ui, |ui| {
            for (i, status) in ReviewStatus::all().iter().enumerate() {
                if ui
                    .selectable_label(selected_idx == i, status.label())
                    .clicked()
                {
                    selected_idx = i;
                    snapshot_needed = true;
                }
            }
        });
    let new_status = ReviewStatus::all()[selected_idx];
    if new_status != current_status {
        graph.set_review_status(selected, new_status);
    }

    let comment_count = graph
        .comments
        .iter()
        .filter(|c| c.node_id == selected)
        .count();
    ui.label(format!("Comments: {comment_count}"));

    snapshot_needed
}

fn show_choice_inspector(
    ui: &mut Ui,
    data: &mut crate::model::node::ChoiceData,
) -> (DeferredAction, bool) {
    let mut snapshot_needed = false;

    ui.label("Prompt:");
    if ui.text_edit_singleline(&mut data.prompt).gained_focus() {
        snapshot_needed = true;
    }

    ui.add_space(8.0);
    ui.label("Choices:");
    ui.separator();

    let mut remove_idx = None;
    let choice_count = data.choices.len();
    for (i, choice) in data.choices.iter_mut().enumerate() {
        ui.horizontal(|ui| {
            ui.label(format!("{}.", i + 1));
            if ui.text_edit_singleline(&mut choice.text).gained_focus() {
                snapshot_needed = true;
            }
            if choice_count > 1 && ui.small_button("X").clicked() {
                remove_idx = Some(i);
            }
        });
    }

    if let Some(idx) = remove_idx {
        return (DeferredAction::RemoveChoice(idx), true);
    }

    if ui.button("+ Add Choice").clicked() {
        return (DeferredAction::AddChoice, true);
    }

    (DeferredAction::None, snapshot_needed)
}

fn show_random_inspector(
    ui: &mut Ui,
    data: &mut crate::model::node::RandomData,
) -> (DeferredAction, bool) {
    let mut snapshot_needed = false;

    ui.label("Branches:");
    ui.separator();

    let mut remove_idx = None;
    let branch_count = data.branches.len();
    for (i, branch) in data.branches.iter_mut().enumerate() {
        ui.horizontal(|ui| {
            let mut pct = branch.weight * 100.0;
            ui.label(format!("{}.", i + 1));
            let resp =
                ui.add(egui::DragValue::new(&mut pct).range(0.0..=100.0).suffix("%"));
            if resp.drag_started() {
                snapshot_needed = true;
            }
            if resp.changed() {
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
        return (DeferredAction::RemoveRandomBranch(idx), true);
    }

    if ui.button("+ Add Branch").clicked() {
        return (DeferredAction::AddRandomBranch, true);
    }

    (DeferredAction::None, snapshot_needed)
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
