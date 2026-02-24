use egui::{Color32, Ui};

use crate::model::project::VersionSnapshot;

/// Action returned by the version panel for app to handle.
pub enum VersionPanelAction {
    None,
    CreateVersion(String),
    RestoreVersion(usize),
}

/// Draw the version history panel. Returns an action for the caller to process.
pub fn show_version_panel(
    ui: &mut Ui,
    versions: &[VersionSnapshot],
    new_desc_text: &mut String,
) -> VersionPanelAction {
    let mut action = VersionPanelAction::None;

    ui.heading("Version History");
    ui.separator();

    // Version list
    let panel_height = ui.available_height() - 30.0;
    egui::ScrollArea::vertical()
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
                    ui.label(
                        egui::RichText::new(format!("#{}", version.id)).strong(),
                    );
                    ui.label(&version.timestamp);
                    ui.label("-");
                    ui.label(&version.description);
                    if ui.small_button("Restore").clicked() {
                        action = VersionPanelAction::RestoreVersion(version.id);
                    }
                });
            }
        });

    // Bottom: save new version
    ui.separator();
    ui.horizontal(|ui| {
        ui.label("Description:");
        let resp = ui.text_edit_singleline(new_desc_text);
        let enter = resp.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter));
        if (enter || ui.button("Save Version").clicked())
            && !new_desc_text.trim().is_empty()
        {
            let desc = new_desc_text.trim().to_string();
            *new_desc_text = String::new();
            action = VersionPanelAction::CreateVersion(desc);
        }
    });

    action
}
