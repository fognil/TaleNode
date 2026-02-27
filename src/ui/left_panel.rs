use egui::Ui;
use uuid::Uuid;

use crate::app::async_runtime::VoiceInfo;
use crate::model::character::Character;
use crate::model::graph::DialogueGraph;
use crate::model::node::VariableValue;
use crate::model::variable::{Variable, VariableType};

/// Draw the left panel (variables, characters, groups).
/// Returns `true` when an undo-worthy event starts (caller should snapshot).
pub fn show_left_panel(
    ui: &mut Ui,
    graph: &mut DialogueGraph,
    available_voices: &[VoiceInfo],
    portrait_cache: &mut crate::ui::portrait_cache::PortraitCache,
    project_dir: Option<&std::path::Path>,
) -> bool {
    let mut snapshot_needed = false;

    // --- Variables section ---
    egui::CollapsingHeader::new("Variables")
        .default_open(true)
        .show(ui, |ui| {
            let mut remove_var = None;
            for (i, var) in graph.variables.iter_mut().enumerate() {
                ui.group(|ui| {
                    ui.horizontal(|ui| {
                        ui.strong(&var.name);
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if ui.small_button("X").on_hover_text("Delete variable").clicked() {
                                remove_var = Some(i);
                                snapshot_needed = true;
                            }
                        });
                    });
                    egui::Grid::new(format!("var_{}", var.id))
                        .num_columns(2)
                        .spacing([8.0, 4.0])
                        .min_col_width(60.0)
                        .show(ui, |ui| {
                            ui.label("Name:");
                            if ui.text_edit_singleline(&mut var.name).gained_focus() {
                                snapshot_needed = true;
                            }
                            ui.end_row();

                            ui.label("Type:");
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
                                            snapshot_needed = true;
                                        }
                                    }
                                });
                            ui.end_row();

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

                            ui.label("Default:");
                            match &mut var.default_value {
                                VariableValue::Bool(b) => {
                                    if ui.checkbox(b, "").changed() { snapshot_needed = true; }
                                }
                                VariableValue::Int(n) => {
                                    if ui.add(egui::DragValue::new(n)).drag_started() {
                                        snapshot_needed = true;
                                    }
                                }
                                VariableValue::Float(f) => {
                                    if ui.add(egui::DragValue::new(f).speed(0.1)).drag_started() {
                                        snapshot_needed = true;
                                    }
                                }
                                VariableValue::Text(s) => {
                                    if ui.text_edit_singleline(s).gained_focus() {
                                        snapshot_needed = true;
                                    }
                                }
                            }
                            ui.end_row();
                        });
                });
            }
            if let Some(idx) = remove_var {
                graph.variables.remove(idx);
            }
            if ui.button("+ Add Variable").clicked() {
                snapshot_needed = true;
                graph.variables.push(Variable {
                    id: Uuid::new_v4(),
                    name: format!("var_{}", graph.variables.len() + 1),
                    var_type: VariableType::Bool,
                    default_value: VariableValue::Bool(false),
                });
            }
        });

    // --- Characters section ---
    egui::CollapsingHeader::new("Characters")
        .default_open(true)
        .show(ui, |ui| {
            let mut remove_char = None;
            for (i, ch) in graph.characters.iter_mut().enumerate() {
                ui.group(|ui| {
                    ui.horizontal(|ui| {
                        let color = egui::Color32::from_rgba_premultiplied(
                            ch.color[0], ch.color[1], ch.color[2], ch.color[3],
                        );
                        let (rect, _) = ui.allocate_exact_size(
                            egui::Vec2::new(12.0, 12.0),
                            egui::Sense::hover(),
                        );
                        ui.painter().rect_filled(rect, 2.0, color);
                        ui.strong(&ch.name);
                        ui.with_layout(
                            egui::Layout::right_to_left(egui::Align::Center),
                            |ui| {
                                if ui.small_button("X").on_hover_text("Delete character").clicked()
                                {
                                    remove_char = Some(i);
                                    snapshot_needed = true;
                                }
                            },
                        );
                    });
                    egui::Grid::new(format!("char_{}", ch.id))
                        .num_columns(2)
                        .spacing([8.0, 4.0])
                        .min_col_width(60.0)
                        .show(ui, |ui| {
                            ui.label("Name:");
                            if ui.text_edit_singleline(&mut ch.name).gained_focus() {
                                snapshot_needed = true;
                            }
                            ui.end_row();

                            ui.label("Color:");
                            let mut color_arr = [ch.color[0], ch.color[1], ch.color[2]];
                            if ui.color_edit_button_srgb(&mut color_arr).changed() {
                                snapshot_needed = true;
                            }
                            ch.color[0] = color_arr[0];
                            ch.color[1] = color_arr[1];
                            ch.color[2] = color_arr[2];
                            ui.end_row();

                            ui.label("Portrait:");
                            ui.horizontal(|ui| {
                                let tex_id = portrait_cache
                                    .get_or_load(ui.ctx(), &ch.portrait_path, project_dir)
                                    .map(|h| h.id());
                                if let Some(id) = tex_id {
                                    ui.image(egui::load::SizedTexture::new(id, [20.0, 20.0]));
                                }
                                let field_w = (ui.available_width() - 35.0).max(60.0);
                                if ui
                                    .add(
                                        egui::TextEdit::singleline(&mut ch.portrait_path)
                                            .desired_width(field_w),
                                    )
                                    .gained_focus()
                                {
                                    snapshot_needed = true;
                                }
                                if ui
                                    .small_button("[...]")
                                    .on_hover_text("Browse for image")
                                    .clicked()
                                {
                                    snapshot_needed = true;
                                    if let Some(path) = rfd::FileDialog::new()
                                        .add_filter("Images", &["png", "jpg", "jpeg", "bmp", "gif"])
                                        .pick_file()
                                    {
                                        ch.portrait_path =
                                            crate::ui::portrait_cache::make_relative_path(
                                                &path,
                                                project_dir,
                                            );
                                    }
                                }
                            });
                            ui.end_row();

                            if !available_voices.is_empty() {
                                ui.label("Voice:");
                                let current_label = ch
                                    .voice_id
                                    .as_ref()
                                    .and_then(|vid| {
                                        available_voices
                                            .iter()
                                            .find(|v| &v.voice_id == vid)
                                            .map(|v| v.name.as_str())
                                    })
                                    .unwrap_or("(none)");
                                egui::ComboBox::from_id_salt(format!("voice_{}", ch.id))
                                    .selected_text(current_label)
                                    .show_ui(ui, |ui| {
                                        if ui
                                            .selectable_label(ch.voice_id.is_none(), "(none)")
                                            .clicked()
                                        {
                                            ch.voice_id = None;
                                            snapshot_needed = true;
                                        }
                                        for voice in available_voices {
                                            let sel =
                                                ch.voice_id.as_ref() == Some(&voice.voice_id);
                                            if ui.selectable_label(sel, &voice.name).clicked() {
                                                ch.voice_id = Some(voice.voice_id.clone());
                                                snapshot_needed = true;
                                            }
                                        }
                                    });
                                ui.end_row();
                            }
                        });

                    // Relationships
                    if !ch.relationships.is_empty()
                        || ui.small_button("+ Relationship").clicked()
                    {
                        if ch.relationships.is_empty() {
                            ch.relationships.push(
                                crate::model::relationship::Relationship::new("Friendship"),
                            );
                            snapshot_needed = true;
                        }
                        ui.separator();
                        let mut remove_rel = None;
                        egui::Grid::new(format!("rel_{}", ch.id))
                            .num_columns(3)
                            .spacing([4.0, 2.0])
                            .show(ui, |ui| {
                                for (ri, rel) in ch.relationships.iter_mut().enumerate() {
                                    if ui
                                        .add(
                                            egui::TextEdit::singleline(&mut rel.name)
                                                .desired_width(80.0),
                                        )
                                        .gained_focus()
                                    {
                                        snapshot_needed = true;
                                    }
                                    if ui
                                        .add(
                                            egui::DragValue::new(&mut rel.value)
                                                .range(rel.min..=rel.max),
                                        )
                                        .drag_started()
                                    {
                                        snapshot_needed = true;
                                    }
                                    if ui.small_button("X").clicked() {
                                        remove_rel = Some(ri);
                                        snapshot_needed = true;
                                    }
                                    ui.end_row();
                                }
                            });
                        if let Some(ri) = remove_rel {
                            ch.relationships.remove(ri);
                        }
                    }
                });
            }
            if let Some(idx) = remove_char {
                let removed_id = graph.characters[idx].id;
                graph.characters.remove(idx);
                graph.barks.remove(&removed_id);
            }
            if ui.button("+ Add Character").clicked() {
                snapshot_needed = true;
                graph.characters.push(Character::new(format!(
                    "Character {}",
                    graph.characters.len() + 1
                )));
            }
        });

    // --- Groups section ---
    if !graph.groups.is_empty() {
        egui::CollapsingHeader::new("Groups")
            .default_open(true)
            .show(ui, |ui| {
                let mut remove_group = None;
                for (i, group) in graph.groups.iter_mut().enumerate() {
                    ui.group(|ui| {
                        ui.horizontal(|ui| {
                            let color = egui::Color32::from_rgba_premultiplied(
                                group.color[0],
                                group.color[1],
                                group.color[2],
                                group.color[3].max(100),
                            );
                            let (rect, _) = ui.allocate_exact_size(
                                egui::Vec2::new(12.0, 12.0),
                                egui::Sense::hover(),
                            );
                            ui.painter().rect_filled(rect, 2.0, color);
                            ui.strong(format!("{} ({})", group.name, group.node_ids.len()));
                            ui.with_layout(
                                egui::Layout::right_to_left(egui::Align::Center),
                                |ui| {
                                    if ui
                                        .small_button("X")
                                        .on_hover_text("Delete group")
                                        .clicked()
                                    {
                                        remove_group = Some(i);
                                        snapshot_needed = true;
                                    }
                                },
                            );
                        });
                        egui::Grid::new(format!("grp_{}", group.id))
                            .num_columns(2)
                            .spacing([8.0, 4.0])
                            .min_col_width(60.0)
                            .show(ui, |ui| {
                                ui.label("Name:");
                                if ui.text_edit_singleline(&mut group.name).gained_focus() {
                                    snapshot_needed = true;
                                }
                                ui.end_row();
                                ui.label("Color:");
                                let mut color_arr =
                                    [group.color[0], group.color[1], group.color[2]];
                                if ui.color_edit_button_srgb(&mut color_arr).changed() {
                                    snapshot_needed = true;
                                }
                                group.color[0] = color_arr[0];
                                group.color[1] = color_arr[1];
                                group.color[2] = color_arr[2];
                                ui.end_row();
                            });
                    });
                }
                if let Some(idx) = remove_group {
                    graph.groups.remove(idx);
                }
            });
    }

    snapshot_needed
}
