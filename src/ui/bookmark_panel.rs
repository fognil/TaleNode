use egui::{Color32, Ui};
use uuid::Uuid;

use crate::model::graph::DialogueGraph;

/// Action returned by the bookmark panel for app to handle.
pub enum BookmarkAction {
    None,
    Navigate(Uuid),
    AddTag(Uuid, String),
    RemoveTag(Uuid, String),
}

/// Draw the bookmark/tag panel. Returns an action for the caller to process.
pub fn show_bookmark_panel(
    ui: &mut Ui,
    graph: &DialogueGraph,
    tag_filter: &mut Option<String>,
    new_tag_text: &mut String,
    selected_node: Option<Uuid>,
) -> BookmarkAction {
    let mut action = BookmarkAction::None;

    ui.heading("Bookmarks / Tags");
    ui.separator();

    // Collect all unique tags across all nodes
    let mut all_tags: Vec<String> = graph
        .node_tags
        .values()
        .flatten()
        .cloned()
        .collect();
    all_tags.sort();
    all_tags.dedup();

    // Tag cloud filter
    ui.horizontal_wrapped(|ui| {
        if ui
            .selectable_label(tag_filter.is_none(), "All")
            .clicked()
        {
            *tag_filter = None;
        }
        for tag in &all_tags {
            let selected = tag_filter.as_ref() == Some(tag);
            if ui.selectable_label(selected, tag).clicked() {
                if selected {
                    *tag_filter = None;
                } else {
                    *tag_filter = Some(tag.clone());
                }
            }
        }
    });

    ui.separator();

    // Filtered node list
    let mut matching: Vec<(Uuid, String, Vec<String>)> = graph
        .nodes
        .values()
        .filter_map(|node| {
            let tags = graph.get_tags(node.id);
            if let Some(filter) = tag_filter.as_ref() {
                if !tags.contains(filter) {
                    return None;
                }
            } else if tags.is_empty() {
                return None;
            }
            Some((node.id, node.title().to_string(), tags.to_vec()))
        })
        .collect();
    matching.sort_by(|a, b| a.1.cmp(&b.1));

    let panel_height = ui.available_height() - 30.0;
    egui::ScrollArea::vertical()
        .max_height(panel_height.max(60.0))
        .show(ui, |ui| {
            if matching.is_empty() {
                ui.colored_label(
                    Color32::from_rgb(140, 140, 140),
                    if tag_filter.is_some() {
                        "No nodes with this tag."
                    } else {
                        "No tagged nodes yet."
                    },
                );
                return;
            }

            for (node_id, title, tags) in &matching {
                ui.horizontal(|ui| {
                    let is_selected = selected_node == Some(*node_id);
                    if ui
                        .selectable_label(is_selected, egui::RichText::new(title).strong())
                        .clicked()
                    {
                        action = BookmarkAction::Navigate(*node_id);
                    }
                });

                // Show tags as small labels with X buttons
                ui.horizontal_wrapped(|ui| {
                    for tag in tags {
                        ui.colored_label(
                            Color32::from_rgb(100, 180, 255),
                            format!(" {tag} "),
                        );
                        if ui.small_button("x").clicked() {
                            action =
                                BookmarkAction::RemoveTag(*node_id, tag.clone());
                        }
                    }
                });
                ui.add_space(2.0);
            }
        });

    // Bottom: add tag to selected node
    ui.separator();
    ui.horizontal(|ui| {
        let target_label = if let Some(id) = selected_node {
            graph
                .nodes
                .get(&id)
                .map(|n| n.title().to_string())
                .unwrap_or_else(|| "???".to_string())
        } else {
            "(select a node)".to_string()
        };
        ui.label(format!("Tag: {target_label}"));

        let resp = ui.text_edit_singleline(new_tag_text);
        let enter_pressed =
            resp.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter));
        if (enter_pressed || ui.button("Add").clicked())
            && !new_tag_text.trim().is_empty()
        {
            if let Some(node_id) = selected_node {
                let tag = new_tag_text.trim().to_string();
                *new_tag_text = String::new();
                action = BookmarkAction::AddTag(node_id, tag);
            }
        }
    });

    action
}
