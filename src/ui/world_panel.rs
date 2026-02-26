use egui::Ui;
use uuid::Uuid;

use crate::model::graph::DialogueGraph;
use crate::model::world::EntityCategory;

/// Actions returned by the world panel.
pub enum WorldPanelAction {
    None,
    AddEntity,
    RemoveEntity(Uuid),
    EditEntity,
    AddProperty(Uuid),
    RemoveProperty(Uuid, usize),
}

/// Draw the world entity database panel.
pub fn show_world_panel(
    ui: &mut Ui,
    graph: &mut DialogueGraph,
    category_filter: &mut Option<EntityCategory>,
) -> WorldPanelAction {
    let mut action = WorldPanelAction::None;

    // Category filter
    ui.horizontal(|ui| {
        ui.label("Filter:");
        let filter_label = category_filter
            .as_ref()
            .map(|c| c.label())
            .unwrap_or("All");
        egui::ComboBox::from_id_salt("world_category_filter")
            .selected_text(filter_label)
            .show_ui(ui, |ui| {
                ui.selectable_value(category_filter, None, "All");
                for cat in EntityCategory::ALL {
                    let label = cat.label().to_string();
                    ui.selectable_value(category_filter, Some(cat), label);
                }
            });
    });

    ui.separator();

    if graph.world_entities.is_empty() {
        ui.label("No world entities. Add one to get started.");
    }

    let mut remove_entity = None;
    let entity_ids: Vec<Uuid> = graph.world_entities.iter().map(|e| e.id).collect();
    for (ei, entity_id) in entity_ids.iter().enumerate() {
        let Some(entity) = graph.world_entities.get_mut(ei) else {
            continue;
        };

        // Apply category filter
        if let Some(ref filter) = *category_filter {
            if &entity.category != filter {
                continue;
            }
        }

        let header = if entity.name.is_empty() {
            format!("{} {}", entity.category.label(), ei + 1)
        } else {
            format!("[{}] {}", entity.category.label(), entity.name)
        };

        egui::CollapsingHeader::new(&header)
            .id_salt(entity_id)
            .default_open(false)
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label("Name:");
                    if ui.text_edit_singleline(&mut entity.name).gained_focus() {
                        action = WorldPanelAction::EditEntity;
                    }
                });
                ui.horizontal(|ui| {
                    ui.label("Category:");
                    egui::ComboBox::from_id_salt(format!("entity_cat_{entity_id}"))
                        .selected_text(entity.category.label())
                        .show_ui(ui, |ui| {
                            for cat in EntityCategory::ALL {
                                let label = cat.label().to_string();
                                if ui
                                    .selectable_value(&mut entity.category, cat, label)
                                    .changed()
                                {
                                    action = WorldPanelAction::EditEntity;
                                }
                            }
                        });
                });
                ui.horizontal(|ui| {
                    ui.label("Description:");
                    if ui
                        .text_edit_multiline(&mut entity.description)
                        .gained_focus()
                    {
                        action = WorldPanelAction::EditEntity;
                    }
                });

                // Tags
                ui.horizontal(|ui| {
                    ui.label("Tags:");
                    let tags_str = entity.tags.join(", ");
                    ui.label(&tags_str);
                });

                // Properties
                ui.label("Properties:");
                let mut remove_prop = None;
                for (pi, prop) in entity.properties.iter_mut().enumerate() {
                    ui.horizontal(|ui| {
                        if ui
                            .add(
                                egui::TextEdit::singleline(&mut prop.key).desired_width(80.0),
                            )
                            .gained_focus()
                        {
                            action = WorldPanelAction::EditEntity;
                        }
                        ui.label("=");
                        if ui
                            .add(
                                egui::TextEdit::singleline(&mut prop.value).desired_width(120.0),
                            )
                            .gained_focus()
                        {
                            action = WorldPanelAction::EditEntity;
                        }
                        if ui.small_button("X").clicked() {
                            remove_prop = Some(pi);
                        }
                    });
                }
                if let Some(pi) = remove_prop {
                    action = WorldPanelAction::RemoveProperty(*entity_id, pi);
                }
                if ui.small_button("+ Property").clicked() {
                    action = WorldPanelAction::AddProperty(*entity_id);
                }

                ui.separator();
                if ui.small_button("Delete Entity").clicked() {
                    remove_entity = Some(ei);
                }
            });
    }

    if let Some(ei) = remove_entity {
        action = WorldPanelAction::RemoveEntity(graph.world_entities[ei].id);
    }

    ui.add_space(4.0);
    if ui.button("+ Add Entity").clicked() {
        action = WorldPanelAction::AddEntity;
    }

    action
}
