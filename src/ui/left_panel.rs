use egui::Ui;
use uuid::Uuid;

use crate::model::character::Character;
use crate::model::graph::DialogueGraph;
use crate::model::node::VariableValue;
use crate::model::variable::{Variable, VariableType};

pub fn show_left_panel(ui: &mut Ui, graph: &mut DialogueGraph) {
    // --- Variables section ---
    ui.heading("Variables");
    ui.separator();

    let mut remove_var = None;
    for (i, var) in graph.variables.iter_mut().enumerate() {
        ui.horizontal(|ui| {
            ui.label(&var.name);
            if ui.small_button("X").clicked() {
                remove_var = Some(i);
            }
        });

        ui.indent(var.id, |ui| {
            ui.horizontal(|ui| {
                ui.label("Name:");
                ui.text_edit_singleline(&mut var.name);
            });

            // Type selector
            let type_labels = ["Bool", "Int", "Float", "Text"];
            let current = match var.var_type {
                VariableType::Bool => 0,
                VariableType::Int => 1,
                VariableType::Float => 2,
                VariableType::Text => 3,
            };
            let mut selected = current;
            egui::ComboBox::from_id_salt(format!("var_type_{}", var.id))
                .selected_text(type_labels[selected])
                .show_ui(ui, |ui| {
                    for (idx, label) in type_labels.iter().enumerate() {
                        if ui.selectable_label(selected == idx, *label).clicked() {
                            selected = idx;
                        }
                    }
                });

            if selected != current {
                let (new_type, new_val) = match selected {
                    0 => (VariableType::Bool, VariableValue::Bool(false)),
                    1 => (VariableType::Int, VariableValue::Int(0)),
                    2 => (VariableType::Float, VariableValue::Float(0.0)),
                    _ => (VariableType::Text, VariableValue::Text(String::new())),
                };
                var.var_type = new_type;
                var.default_value = new_val;
            }

            // Default value
            ui.horizontal(|ui| {
                ui.label("Default:");
                match &mut var.default_value {
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
            });
        });

        ui.add_space(4.0);
    }

    if let Some(idx) = remove_var {
        graph.variables.remove(idx);
    }

    if ui.button("+ Add Variable").clicked() {
        graph.variables.push(Variable {
            id: Uuid::new_v4(),
            name: format!("var_{}", graph.variables.len() + 1),
            var_type: VariableType::Bool,
            default_value: VariableValue::Bool(false),
        });
    }

    ui.add_space(16.0);

    // --- Characters section ---
    ui.heading("Characters");
    ui.separator();

    let mut remove_char = None;
    for (i, ch) in graph.characters.iter_mut().enumerate() {
        ui.horizontal(|ui| {
            // Color indicator
            let color = egui::Color32::from_rgba_premultiplied(
                ch.color[0], ch.color[1], ch.color[2], ch.color[3],
            );
            let (rect, _) = ui.allocate_exact_size(
                egui::Vec2::new(12.0, 12.0),
                egui::Sense::hover(),
            );
            ui.painter().rect_filled(rect, 2.0, color);

            ui.label(&ch.name);
            if ui.small_button("X").clicked() {
                remove_char = Some(i);
            }
        });

        ui.indent(ch.id, |ui| {
            ui.horizontal(|ui| {
                ui.label("Name:");
                ui.text_edit_singleline(&mut ch.name);
            });

            ui.horizontal(|ui| {
                ui.label("Color:");
                let mut color_arr = [ch.color[0], ch.color[1], ch.color[2]];
                if ui.color_edit_button_srgb(&mut color_arr).changed() {
                    ch.color[0] = color_arr[0];
                    ch.color[1] = color_arr[1];
                    ch.color[2] = color_arr[2];
                }
            });

            ui.horizontal(|ui| {
                ui.label("Portrait:");
                ui.text_edit_singleline(&mut ch.portrait_path);
            });
        });

        ui.add_space(4.0);
    }

    if let Some(idx) = remove_char {
        graph.characters.remove(idx);
    }

    if ui.button("+ Add Character").clicked() {
        graph.characters.push(Character::new(format!(
            "Character {}",
            graph.characters.len() + 1
        )));
    }

    // --- Groups section ---
    if !graph.groups.is_empty() {
        ui.add_space(16.0);
        ui.heading("Groups");
        ui.separator();

        let mut remove_group = None;
        for (i, group) in graph.groups.iter_mut().enumerate() {
            ui.horizontal(|ui| {
                let color = egui::Color32::from_rgba_premultiplied(
                    group.color[0], group.color[1], group.color[2], group.color[3].max(100),
                );
                let (rect, _) = ui.allocate_exact_size(
                    egui::Vec2::new(12.0, 12.0),
                    egui::Sense::hover(),
                );
                ui.painter().rect_filled(rect, 2.0, color);

                ui.label(format!("{} ({})", group.name, group.node_ids.len()));
                if ui.small_button("X").clicked() {
                    remove_group = Some(i);
                }
            });

            ui.indent(group.id, |ui| {
                ui.horizontal(|ui| {
                    ui.label("Name:");
                    ui.text_edit_singleline(&mut group.name);
                });
                ui.horizontal(|ui| {
                    ui.label("Color:");
                    let mut color_arr = [group.color[0], group.color[1], group.color[2]];
                    if ui.color_edit_button_srgb(&mut color_arr).changed() {
                        group.color[0] = color_arr[0];
                        group.color[1] = color_arr[1];
                        group.color[2] = color_arr[2];
                    }
                });
            });

            ui.add_space(4.0);
        }

        if let Some(idx) = remove_group {
            graph.groups.remove(idx);
        }
    }
}
