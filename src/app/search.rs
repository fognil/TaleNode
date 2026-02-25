use std::time::Instant;

use egui::Color32;
use uuid::Uuid;

use crate::model::node::Node;

use super::TaleNodeApp;

impl TaleNodeApp {
    /// Search nodes for matching text content.
    pub(super) fn update_search_results(&mut self) {
        self.search_results.clear();
        let query = self.search_query.to_lowercase();
        if query.is_empty() {
            return;
        }
        for node in self.graph.nodes.values() {
            if node_matches_query(node, &query, &self.graph) {
                self.search_results.push(node.id);
            }
        }
        if self.search_index >= self.search_results.len() {
            self.search_index = 0;
        }
    }

    pub(super) fn focus_search_result(&mut self) {
        if let Some(&node_id) = self.search_results.get(self.search_index) {
            self.selected_nodes.clear();
            self.selected_nodes.push(node_id);
            if let Some(node) = self.graph.nodes.get(&node_id) {
                self.canvas.pan_offset = egui::Vec2::new(
                    -node.position[0] * self.canvas.zoom,
                    -node.position[1] * self.canvas.zoom,
                );
            }
        }
    }

    pub(super) fn show_search_bar(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.label("Find:");
            let changed = ui
                .text_edit_singleline(&mut self.search_query)
                .changed();
            if changed {
                self.update_search_results();
            }

            let count = self.search_results.len();
            if !self.search_query.is_empty() {
                if count > 0 {
                    ui.label(format!("{}/{count}", self.search_index + 1));
                    if ui.small_button("<").clicked() {
                        self.search_index = if self.search_index == 0 {
                            count - 1
                        } else {
                            self.search_index - 1
                        };
                        self.focus_search_result();
                    }
                    if ui.small_button(">").clicked() {
                        self.search_index = (self.search_index + 1) % count;
                        self.focus_search_result();
                    }
                } else {
                    ui.colored_label(Color32::from_rgb(255, 100, 100), "No matches");
                }
            }

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.small_button("X").clicked() {
                    self.show_search = false;
                    self.show_replace = false;
                    self.search_query.clear();
                    self.search_results.clear();
                    self.replace_query.clear();
                }
                let toggle_label = if self.show_replace {
                    "Hide Replace"
                } else {
                    "Replace"
                };
                if ui.small_button(toggle_label).clicked() {
                    self.show_replace = !self.show_replace;
                }
            });
        });

        if self.show_replace {
            ui.horizontal(|ui| {
                ui.label("Replace:");
                ui.text_edit_singleline(&mut self.replace_query);

                let has_matches =
                    !self.search_query.is_empty() && !self.search_results.is_empty();
                if ui
                    .add_enabled(has_matches, egui::Button::new("Replace"))
                    .clicked()
                {
                    self.replace_in_current();
                }
                if ui
                    .add_enabled(has_matches, egui::Button::new("Replace All"))
                    .clicked()
                {
                    self.replace_all();
                }
            });
        }
    }

    /// Replace the search query in the currently focused search result node.
    fn replace_in_current(&mut self) {
        let Some(&node_id) = self.search_results.get(self.search_index) else {
            return;
        };
        self.snapshot();
        let query = self.search_query.clone();
        let replacement = self.replace_query.clone();
        if let Some(node) = self.graph.nodes.get_mut(&node_id) {
            replace_in_node(node, &query, &replacement);
        }
        self.update_search_results();
        if !self.search_results.is_empty() {
            if self.search_index >= self.search_results.len() {
                self.search_index = 0;
            }
            self.focus_search_result();
        }
        self.status_message = Some(("Replaced in current node".to_string(), Instant::now(), false));
    }

    /// Replace the search query in all matching nodes.
    fn replace_all(&mut self) {
        if self.search_results.is_empty() {
            return;
        }
        self.snapshot();
        let query = self.search_query.clone();
        let replacement = self.replace_query.clone();
        let ids: Vec<Uuid> = self.search_results.clone();
        let mut count = 0;
        for id in &ids {
            if let Some(node) = self.graph.nodes.get_mut(id) {
                count += replace_in_node(node, &query, &replacement);
            }
        }
        self.update_search_results();
        self.search_index = 0;
        self.status_message = Some((
            format!("{count} replacement(s) across {} node(s)", ids.len()),
            Instant::now(),
            false,
        ));
    }
}

fn node_matches_query(
    node: &Node,
    query: &str,
    graph: &crate::model::graph::DialogueGraph,
) -> bool {
    use crate::model::node::NodeType;

    // Check tags first
    for tag in graph.get_tags(node.id) {
        if tag.to_lowercase().contains(query) {
            return true;
        }
    }

    if node.title().to_lowercase().contains(query) {
        return true;
    }
    match &node.node_type {
        NodeType::Dialogue(data) => {
            data.text.to_lowercase().contains(query)
                || data.speaker_name.to_lowercase().contains(query)
                || data.emotion.to_lowercase().contains(query)
        }
        NodeType::Choice(data) => {
            data.prompt.to_lowercase().contains(query)
                || data
                    .choices
                    .iter()
                    .any(|c| c.text.to_lowercase().contains(query))
        }
        NodeType::Condition(data) => data.variable_name.to_lowercase().contains(query),
        NodeType::Event(data) => data
            .actions
            .iter()
            .any(|a| a.key.to_lowercase().contains(query)),
        NodeType::End(data) => data.tag.to_lowercase().contains(query),
        _ => false,
    }
}

/// Replace all case-insensitive occurrences of `query` in a node's text fields.
/// Returns the number of individual string replacements made.
fn replace_in_node(node: &mut Node, query: &str, replacement: &str) -> usize {
    use crate::model::node::NodeType;
    let mut count = 0;

    match &mut node.node_type {
        NodeType::Dialogue(data) => {
            count += replace_in_string(&mut data.text, query, replacement);
            count += replace_in_string(&mut data.speaker_name, query, replacement);
            count += replace_in_string(&mut data.emotion, query, replacement);
        }
        NodeType::Choice(data) => {
            count += replace_in_string(&mut data.prompt, query, replacement);
            for choice in &mut data.choices {
                count += replace_in_string(&mut choice.text, query, replacement);
            }
        }
        NodeType::Condition(data) => {
            count += replace_in_string(&mut data.variable_name, query, replacement);
        }
        NodeType::Event(data) => {
            for action in &mut data.actions {
                count += replace_in_string(&mut action.key, query, replacement);
            }
        }
        NodeType::End(data) => {
            count += replace_in_string(&mut data.tag, query, replacement);
        }
        _ => {}
    }

    // Sync port labels for Choice nodes after replacement
    if let NodeType::Choice(data) = &node.node_type {
        for (i, choice) in data.choices.iter().enumerate() {
            if let Some(port) = node.outputs.get_mut(i) {
                port.label.clone_from(&choice.text);
            }
        }
    }

    count
}

/// Case-insensitive replace of all occurrences in a string.
/// Returns 1 if any replacement was made, 0 otherwise.
fn replace_in_string(s: &mut String, query: &str, replacement: &str) -> usize {
    let lower = s.to_lowercase();
    let query_lower = query.to_lowercase();
    if !lower.contains(&query_lower) {
        return 0;
    }
    let mut result = String::with_capacity(s.len());
    let mut remaining = s.as_str();
    while let Some(pos) = remaining.to_lowercase().find(&query_lower) {
        result.push_str(&remaining[..pos]);
        result.push_str(replacement);
        remaining = &remaining[pos + query.len()..];
    }
    result.push_str(remaining);
    *s = result;
    1
}
