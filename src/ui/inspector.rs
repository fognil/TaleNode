use egui::Ui;
use uuid::Uuid;

use crate::model::graph::DialogueGraph;
use crate::model::locale::LocaleSettings;
use crate::model::node::NodeType;
use crate::model::review::ReviewStatus;

use super::inspector_widgets::{
    show_condition_inspector, show_dialogue_inspector, show_event_inspector,
    sync_choice_labels, sync_random_labels,
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
#[allow(clippy::too_many_arguments)]
pub fn show_inspector(
    ui: &mut Ui,
    graph: &mut DialogueGraph,
    selected: Uuid,
    new_tag_text: &mut String,
    active_locale: &mut Option<String>,
) -> bool {
    let mut deferred = DeferredAction::None;
    let mut snapshot_needed = false;

    // First pass: draw UI and collect deferred actions
    let locale = graph.locale.clone();
    if let Some(node) = graph.nodes.get_mut(&selected) {
        ui.heading(node.title());
        ui.separator();

        ui.horizontal(|ui| {
            ui.label("ID:");
            ui.monospace(node.id.to_string().chars().take(8).collect::<String>());
        });

        // Locale switcher (only if extra locales exist)
        if locale.has_extra_locales() {
            show_locale_switcher(ui, &locale, active_locale);
        }
        ui.add_space(8.0);

        let uuid8 = node.id.to_string()[..8].to_string();
        let editing_locale = active_locale.clone();

        match &mut node.node_type {
            NodeType::Start => {
                ui.label("Start node — entry point of the dialogue.");
            }

            NodeType::Dialogue(data) => {
                let characters = graph.characters.clone();
                if show_dialogue_inspector(ui, data, &characters, &editing_locale, &locale, &uuid8)
                {
                    snapshot_needed = true;
                }
            }

            NodeType::Choice(data) => {
                let (action, changed) =
                    show_choice_inspector(ui, data, &editing_locale, &locale, &uuid8);
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

    // Apply locale translations from inspector widgets back to graph
    if let Some(ref loc) = active_locale {
        apply_locale_edits(ui, &mut graph.locale, loc, &mut snapshot_needed);
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
            if ui.small_button("x").on_hover_text("Remove tag").clicked() {
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
        if ui.button("+").on_hover_text("Add tag").clicked() && !new_tag_text.trim().is_empty() {
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

fn show_locale_switcher(
    ui: &mut Ui,
    locale: &LocaleSettings,
    active_locale: &mut Option<String>,
) {
    ui.horizontal(|ui| {
        ui.label("Locale:");
        let label = active_locale
            .as_deref()
            .map_or_else(|| format!("Default ({})", locale.default_locale), String::from);
        egui::ComboBox::from_id_salt("inspector_locale")
            .selected_text(&label)
            .show_ui(ui, |ui| {
                let is_default = active_locale.is_none();
                if ui
                    .selectable_label(is_default, format!("Default ({})", locale.default_locale))
                    .clicked()
                {
                    *active_locale = None;
                }
                for loc in &locale.extra_locales {
                    let selected = active_locale.as_deref() == Some(loc.as_str());
                    if ui.selectable_label(selected, loc).clicked() {
                        *active_locale = Some(loc.clone());
                    }
                }
            });
    });
}

fn apply_locale_edits(
    ui: &mut Ui,
    locale: &mut LocaleSettings,
    loc: &str,
    snapshot_needed: &mut bool,
) {
    // Read locale edits stored in egui memory by locale_text_field
    let edits: Vec<(String, String)> = ui.ctx().memory_mut(|mem| {
        mem.data
            .remove_temp::<Vec<(String, String)>>(egui::Id::new("locale_edits"))
            .unwrap_or_default()
    });
    for (key, text) in edits {
        *snapshot_needed = true;
        locale.set_translation(key, loc.to_string(), text);
    }
}

fn show_choice_inspector(
    ui: &mut Ui,
    data: &mut crate::model::node::ChoiceData,
    editing_locale: &Option<String>,
    locale: &LocaleSettings,
    uuid8: &str,
) -> (DeferredAction, bool) {
    let mut snapshot_needed = false;

    ui.label("Prompt:");
    if editing_locale.is_some() {
        super::inspector_widgets::locale_text_field(
            ui,
            &data.prompt,
            &format!("choice_{uuid8}"),
            editing_locale,
            locale,
            false,
        );
    }
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
            if choice_count > 1 && ui.small_button("X").on_hover_text("Remove choice").clicked() {
                remove_idx = Some(i);
            }
        });
        if editing_locale.is_some() {
            super::inspector_widgets::locale_text_field(
                ui,
                &choice.text,
                &format!("opt_{uuid8}_{i}"),
                editing_locale,
                locale,
                false,
            );
        }
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
            if branch_count > 1 && ui.small_button("X").on_hover_text("Remove branch").clicked() {
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
