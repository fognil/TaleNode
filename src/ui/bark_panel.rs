use egui::Ui;
use uuid::Uuid;

use crate::model::graph::DialogueGraph;

/// Actions returned by the bark panel for the handler to apply.
pub enum BarkPanelAction {
    None,
    AddBark(Uuid),
    RemoveBark(Uuid, Uuid),
    EditBark,
}

/// Draw the bark/ambient dialogue editor panel.
/// Takes a mutable graph reference and the currently selected character.
pub fn show_bark_panel(
    ui: &mut Ui,
    graph: &mut DialogueGraph,
    selected_character: &mut Option<Uuid>,
) -> BarkPanelAction {
    let mut action = BarkPanelAction::None;

    if graph.characters.is_empty() {
        ui.label("Add characters in the Project panel first.");
        return action;
    }

    // Character selector dropdown
    let current_label = selected_character
        .and_then(|id| graph.characters.iter().find(|c| c.id == id))
        .map(|c| c.name.clone())
        .unwrap_or_else(|| "(select character)".to_string());

    ui.horizontal(|ui| {
        ui.label("Character:");
        egui::ComboBox::from_id_salt("bark_char_select")
            .selected_text(&current_label)
            .show_ui(ui, |ui| {
                for ch in &graph.characters {
                    let sel = *selected_character == Some(ch.id);
                    if ui.selectable_label(sel, &ch.name).clicked() {
                        *selected_character = Some(ch.id);
                    }
                }
            });
    });

    ui.separator();

    let Some(char_id) = *selected_character else {
        ui.label("Select a character to edit barks.");
        return action;
    };

    let barks = graph.barks.entry(char_id).or_default();

    // Bark line list
    let mut remove_bark_id = None;
    for bark in barks.iter_mut() {
        ui.group(|ui| {
            ui.horizontal(|ui| {
                ui.label("Text:");
                if ui
                    .text_edit_multiline(&mut bark.text)
                    .gained_focus()
                {
                    action = BarkPanelAction::EditBark;
                }
            });
            ui.horizontal(|ui| {
                ui.label("Weight:");
                if ui
                    .add(egui::DragValue::new(&mut bark.weight).speed(0.1).range(0.1..=10.0))
                    .drag_started()
                {
                    action = BarkPanelAction::EditBark;
                }
            });
            ui.horizontal(|ui| {
                ui.label("Condition:");
                let mut cond = bark.condition_variable.clone().unwrap_or_default();
                if ui
                    .add(
                        egui::TextEdit::singleline(&mut cond)
                            .hint_text("variable name (optional)")
                            .desired_width(150.0),
                    )
                    .changed()
                {
                    bark.condition_variable = if cond.is_empty() {
                        None
                    } else {
                        Some(cond)
                    };
                    action = BarkPanelAction::EditBark;
                }
            });
            if ui.small_button("Remove").clicked() {
                remove_bark_id = Some(bark.id);
            }
        });
        ui.add_space(2.0);
    }

    if let Some(bark_id) = remove_bark_id {
        action = BarkPanelAction::RemoveBark(char_id, bark_id);
    }

    if ui.button("+ Add Bark Line").clicked() {
        action = BarkPanelAction::AddBark(char_id);
    }

    action
}
