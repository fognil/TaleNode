use egui::{Color32, Ui};
use uuid::Uuid;

use crate::model::template::TemplateLibrary;

/// Action returned by the template panel for app to handle.
pub enum TemplatePanelAction {
    None,
    Insert(Uuid),
    Delete(Uuid),
    SaveSelection(String),
}

/// Draw the template library panel. Returns an action for the caller to process.
pub fn show_template_panel(
    ui: &mut Ui,
    library: &TemplateLibrary,
    new_name: &mut String,
    has_selection: bool,
) -> TemplatePanelAction {
    let mut action = TemplatePanelAction::None;

    ui.heading("Template Library");
    ui.separator();

    // Save selection as template
    if has_selection {
        ui.horizontal(|ui| {
            ui.label("Name:");
            let resp = ui.text_edit_singleline(new_name);
            let enter = resp.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter));
            if (enter || ui.button("Save Selection").on_hover_text("Save selected nodes as template").clicked()) && !new_name.trim().is_empty() {
                action = TemplatePanelAction::SaveSelection(new_name.trim().to_string());
                *new_name = String::new();
            }
        });
        ui.separator();
    } else {
        ui.colored_label(
            Color32::from_rgb(140, 140, 140),
            "Select nodes to save as template.",
        );
        ui.separator();
    }

    // Template list
    let panel_height = ui.available_height();
    egui::ScrollArea::vertical()
        .max_height(panel_height.max(60.0))
        .show(ui, |ui| {
            if library.templates.is_empty() {
                ui.colored_label(
                    Color32::from_rgb(140, 140, 140),
                    "No templates yet.",
                );
                return;
            }

            for template in &library.templates {
                ui.horizontal(|ui| {
                    // Template name + node count
                    let label = if template.is_builtin {
                        format!(
                            "{} ({} nodes) [builtin]",
                            template.name,
                            template.node_count()
                        )
                    } else {
                        format!("{} ({} nodes)", template.name, template.node_count())
                    };
                    ui.label(&label);

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        // Delete button (user-saved only)
                        if !template.is_builtin && ui.small_button("Delete").on_hover_text("Delete template").clicked() {
                            action = TemplatePanelAction::Delete(template.id);
                        }
                        // Insert button
                        if ui.small_button("Insert").on_hover_text("Insert template into canvas").clicked() {
                            action = TemplatePanelAction::Insert(template.id);
                        }
                    });
                });

                if !template.description.is_empty() {
                    ui.colored_label(
                        Color32::from_rgb(140, 140, 140),
                        &template.description,
                    );
                }
                ui.add_space(2.0);
            }
        });

    action
}
