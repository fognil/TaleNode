use egui::{Color32, Ui};

use crate::model::graph_diff::GraphDiff;
use crate::model::project::VersionSnapshot;

/// Action returned by the version panel for app to handle.
pub enum VersionPanelAction {
    None,
    CreateVersion(String),
    RestoreVersion(usize),
    CompareVersions(usize, usize),
}

/// Draw the version history panel. Returns an action for the caller to process.
pub fn show_version_panel(
    ui: &mut Ui,
    versions: &[VersionSnapshot],
    new_desc_text: &mut String,
    compare_selection: &mut [Option<usize>; 2],
    diff_result: Option<&GraphDiff>,
) -> VersionPanelAction {
    let mut action = VersionPanelAction::None;

    ui.heading("Version History");
    ui.separator();

    // Version list
    let panel_height = if diff_result.is_some() {
        (ui.available_height() - 30.0) * 0.5
    } else {
        ui.available_height() - 30.0
    };
    egui::ScrollArea::vertical()
        .id_salt("version_list")
        .max_height(panel_height.max(60.0))
        .show(ui, |ui| {
            if versions.is_empty() {
                ui.colored_label(
                    Color32::from_rgb(140, 140, 140),
                    "No versions saved yet.",
                );
                return;
            }

            // Show newest first
            for version in versions.iter().rev() {
                ui.horizontal(|ui| {
                    // Checkbox for compare selection
                    let mut checked = compare_selection.contains(&Some(version.id));
                    if ui.checkbox(&mut checked, "").changed() {
                        if checked {
                            // Add to selection (fill first empty slot)
                            if compare_selection[0].is_none() {
                                compare_selection[0] = Some(version.id);
                            } else if compare_selection[1].is_none() {
                                compare_selection[1] = Some(version.id);
                            } else {
                                // Both slots full, replace second
                                compare_selection[1] = Some(version.id);
                            }
                        } else {
                            // Remove from selection
                            if compare_selection[0] == Some(version.id) {
                                compare_selection[0] = None;
                            }
                            if compare_selection[1] == Some(version.id) {
                                compare_selection[1] = None;
                            }
                        }
                    }

                    ui.label(
                        egui::RichText::new(format!("#{}", version.id)).strong(),
                    );
                    ui.label(&version.timestamp);
                    ui.label("-");
                    ui.label(&version.description);
                    if ui.small_button("Restore").on_hover_text("Restore this version").clicked() {
                        action = VersionPanelAction::RestoreVersion(version.id);
                    }
                });
            }
        });

    // Compare button
    ui.horizontal(|ui| {
        let both_selected =
            compare_selection[0].is_some() && compare_selection[1].is_some();
        if ui
            .add_enabled(both_selected, egui::Button::new("Compare Selected"))
            .on_hover_text("Compare two selected versions")
            .clicked()
        {
            if let (Some(a), Some(b)) = (compare_selection[0], compare_selection[1]) {
                action = VersionPanelAction::CompareVersions(a, b);
            }
        }
        if both_selected {
            ui.label(format!(
                "Comparing #{} vs #{}",
                compare_selection[0].unwrap_or(0),
                compare_selection[1].unwrap_or(0),
            ));
        } else {
            ui.colored_label(
                Color32::from_rgb(140, 140, 140),
                "Select 2 versions to compare",
            );
        }
    });

    // Diff result display
    if let Some(diff) = diff_result {
        ui.separator();
        show_diff_summary(ui, diff);
    }

    // Bottom: save new version
    ui.separator();
    ui.horizontal(|ui| {
        ui.label("Description:");
        let resp = ui.text_edit_singleline(new_desc_text);
        let enter = resp.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter));
        if (enter || ui.button("Save Version").on_hover_text("Save current state as version").clicked())
            && !new_desc_text.trim().is_empty()
        {
            let desc = new_desc_text.trim().to_string();
            *new_desc_text = String::new();
            action = VersionPanelAction::CreateVersion(desc);
        }
    });

    action
}

fn show_diff_summary(ui: &mut Ui, diff: &GraphDiff) {
    if diff.is_empty() {
        ui.colored_label(Color32::from_rgb(140, 140, 140), "No differences found.");
        return;
    }

    let green = Color32::from_rgb(80, 200, 80);
    let red = Color32::from_rgb(220, 80, 80);
    let yellow = Color32::from_rgb(220, 200, 60);

    ui.label(egui::RichText::new("Diff Summary").strong());

    // Node changes
    ui.horizontal(|ui| {
        if !diff.added_nodes.is_empty() {
            ui.colored_label(green, format!("+{} nodes", diff.added_nodes.len()));
        }
        if !diff.removed_nodes.is_empty() {
            ui.colored_label(red, format!("-{} nodes", diff.removed_nodes.len()));
        }
        if !diff.modified_nodes.is_empty() {
            ui.colored_label(yellow, format!("~{} modified", diff.modified_nodes.len()));
        }
    });

    // Connection changes
    if diff.added_connections > 0 || diff.removed_connections > 0 {
        ui.horizontal(|ui| {
            if diff.added_connections > 0 {
                ui.colored_label(green, format!("+{} connections", diff.added_connections));
            }
            if diff.removed_connections > 0 {
                ui.colored_label(red, format!("-{} connections", diff.removed_connections));
            }
        });
    }

    // Variable changes
    if !diff.added_variables.is_empty() || !diff.removed_variables.is_empty() {
        ui.horizontal(|ui| {
            for name in &diff.added_variables {
                ui.colored_label(green, format!("+var:{name}"));
            }
            for name in &diff.removed_variables {
                ui.colored_label(red, format!("-var:{name}"));
            }
        });
    }

    // Character changes
    if !diff.added_characters.is_empty() || !diff.removed_characters.is_empty() {
        ui.horizontal(|ui| {
            for name in &diff.added_characters {
                ui.colored_label(green, format!("+char:{name}"));
            }
            for name in &diff.removed_characters {
                ui.colored_label(red, format!("-char:{name}"));
            }
        });
    }
}
