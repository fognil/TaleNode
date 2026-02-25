use egui::{Color32, Ui};

/// Action returned by the script panel for app to handle.
pub enum ScriptPanelAction {
    None,
    /// User committed text edits — parse and apply to graph.
    Commit(String),
    /// User requested a refresh from the current graph.
    Refresh,
    /// Text was modified (for dirty-tracking).
    TextChanged,
}

/// Draw the script editor panel. Returns an action for the caller to process.
pub fn show_script_panel(
    ui: &mut Ui,
    text: &mut String,
    dirty: bool,
    stale: bool,
) -> ScriptPanelAction {
    let mut action = ScriptPanelAction::None;

    ui.heading("Script Editor");
    ui.separator();

    // Toolbar
    ui.horizontal(|ui| {
        if dirty {
            if ui.button("Commit").clicked() {
                action = ScriptPanelAction::Commit(text.clone());
            }
            if ui.button("Discard").clicked() {
                action = ScriptPanelAction::Refresh;
            }
        }

        if ui.button("Refresh").clicked() {
            action = ScriptPanelAction::Refresh;
        }

        // Status indicator
        if dirty && stale {
            ui.colored_label(
                Color32::from_rgb(255, 180, 60),
                "Modified + Graph changed",
            );
        } else if dirty {
            ui.colored_label(Color32::from_rgb(255, 200, 80), "Modified");
        } else if stale {
            ui.colored_label(
                Color32::from_rgb(180, 180, 255),
                "Graph changed — click Refresh",
            );
        } else {
            ui.colored_label(Color32::from_rgb(120, 200, 120), "Synced");
        }
    });

    ui.separator();

    // Multiline text editor with monospace font
    let available = ui.available_size();
    egui::ScrollArea::vertical()
        .max_height(available.y)
        .show(ui, |ui| {
            let resp = ui.add(
                egui::TextEdit::multiline(text)
                    .font(egui::TextStyle::Monospace)
                    .desired_width(available.x)
                    .desired_rows(30),
            );
            if resp.changed() && !matches!(action, ScriptPanelAction::Commit(_)) {
                action = ScriptPanelAction::TextChanged;
            }
        });

    action
}
