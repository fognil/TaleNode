use egui::{Color32, Ui};
use uuid::Uuid;

use crate::model::graph::DialogueGraph;
use crate::model::review::ReviewStatus;

/// Color for each review status badge.
pub fn status_color(status: ReviewStatus) -> Color32 {
    match status {
        ReviewStatus::Draft => Color32::from_rgb(158, 158, 158),      // gray
        ReviewStatus::NeedsReview => Color32::from_rgb(251, 188, 4),  // yellow
        ReviewStatus::Approved => Color32::from_rgb(76, 175, 80),     // green
    }
}

/// Action returned by the comments panel for app.rs to handle.
pub enum CommentsPanelAction {
    None,
    Navigate(Uuid),
    AddComment(Uuid, String),
    DeleteComment(Uuid),
}

/// Draw the comments panel. Returns an action for the caller to process.
pub fn show_comments_panel(
    ui: &mut Ui,
    graph: &DialogueGraph,
    filter: &mut Option<ReviewStatus>,
    comment_target_node: &mut Option<Uuid>,
    new_comment_text: &mut String,
) -> CommentsPanelAction {
    let mut action = CommentsPanelAction::None;

    ui.heading("Comments");
    ui.separator();

    // Filter buttons
    ui.horizontal(|ui| {
        if ui
            .selectable_label(filter.is_none(), "All")
            .clicked()
        {
            *filter = None;
        }
        for status in ReviewStatus::all() {
            let color = status_color(*status);
            let selected = *filter == Some(*status);
            let label = egui::RichText::new(status.label()).color(color);
            if ui.selectable_label(selected, label).clicked() {
                *filter = Some(*status);
            }
        }
    });

    ui.separator();

    // Collect nodes matching the filter, sorted by title
    let mut matching_nodes: Vec<(Uuid, String, ReviewStatus)> = graph
        .nodes
        .values()
        .filter_map(|node| {
            let status = graph.get_review_status(node.id);
            if let Some(f) = filter {
                if status != *f {
                    return None;
                }
            }
            Some((node.id, node.title().to_string(), status))
        })
        .collect();
    matching_nodes.sort_by(|a, b| a.1.cmp(&b.1));

    // Scrollable list of nodes with their comments
    let panel_height = ui.available_height() - 30.0;
    egui::ScrollArea::vertical()
        .max_height(panel_height.max(60.0))
        .show(ui, |ui| {
            if matching_nodes.is_empty() {
                ui.colored_label(
                    Color32::from_rgb(140, 140, 140),
                    "No nodes match the filter.",
                );
                return;
            }

            for (node_id, title, status) in &matching_nodes {
                let color = status_color(*status);
                let header = ui.horizontal(|ui| {
                    // Status dot
                    let (rect, _) =
                        ui.allocate_exact_size(egui::vec2(10.0, 10.0), egui::Sense::hover());
                    ui.painter().circle_filled(rect.center(), 4.0, color);

                    // Clickable node title
                    let resp = ui.selectable_label(
                        *comment_target_node == Some(*node_id),
                        egui::RichText::new(title).strong(),
                    );
                    if resp.clicked() {
                        *comment_target_node = Some(*node_id);
                        action = CommentsPanelAction::Navigate(*node_id);
                    }

                    // Comment count
                    let count = graph.comments.iter().filter(|c| c.node_id == *node_id).count();
                    if count > 0 {
                        ui.colored_label(
                            Color32::from_rgb(140, 140, 140),
                            format!("({count})"),
                        );
                    }
                });
                // Use header response for context
                let _ = header;

                // Show comments for this node
                let node_comments: Vec<_> = graph
                    .comments
                    .iter()
                    .filter(|c| c.node_id == *node_id)
                    .collect();
                if !node_comments.is_empty() {
                    ui.indent(node_id.to_string(), |ui| {
                        for comment in &node_comments {
                            ui.horizontal(|ui| {
                                ui.colored_label(
                                    Color32::from_rgb(200, 200, 200),
                                    &comment.text,
                                );
                                if ui.small_button("X").clicked() {
                                    action = CommentsPanelAction::DeleteComment(comment.id);
                                }
                            });
                        }
                    });
                }

                ui.add_space(2.0);
            }
        });

    // Bottom row: add comment input
    ui.separator();
    ui.horizontal(|ui| {
        let target_label = if let Some(tid) = comment_target_node {
            graph
                .nodes
                .get(tid)
                .map(|n| n.title().to_string())
                .unwrap_or_else(|| "???".to_string())
        } else {
            "(select a node)".to_string()
        };
        ui.label(format!("To: {target_label}"));

        let resp = ui.text_edit_singleline(new_comment_text);
        if (resp.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter))
            || ui.button("Add").clicked())
            && comment_target_node.is_some()
            && !new_comment_text.trim().is_empty()
        {
            let node_id = comment_target_node.unwrap();
            let text = new_comment_text.trim().to_string();
            *new_comment_text = String::new();
            action = CommentsPanelAction::AddComment(node_id, text);
        }
    });

    action
}
