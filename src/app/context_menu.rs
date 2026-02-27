use egui::Pos2;

use super::TaleNodeApp;

impl TaleNodeApp {
    pub(super) fn handle_context_menu(&mut self, response: &egui::Response) {
        let Some(ctx_pos) = self.context_menu_pos else {
            return;
        };
        let mut close_menu = false;

        let menu_id = response.id.with("ctx_menu");
        egui::Area::new(menu_id)
            .fixed_pos(self.canvas.canvas_to_screen(Pos2::new(ctx_pos[0], ctx_pos[1])))
            .order(egui::Order::Foreground)
            .show(&response.ctx, |ui| {
                egui::Frame::popup(ui.style()).show(ui, |ui| {
                    ui.set_min_width(150.0);
                    ui.label("Add Node");
                    ui.separator();

                    use crate::model::node::Node;
                    type NodeCtor = fn([f32; 2]) -> Node;
                    let items: &[(&str, NodeCtor)] = &[
                        ("Start", Node::new_start),
                        ("Dialogue", Node::new_dialogue),
                        ("Choice", Node::new_choice),
                        ("Condition", Node::new_condition),
                        ("Event", Node::new_event),
                        ("Random", Node::new_random),
                        ("End", Node::new_end),
                        ("SubGraph", Node::new_subgraph),
                    ];
                    for (label, constructor) in items {
                        if ui.button(*label).clicked() {
                            self.snapshot();
                            self.graph.add_node(constructor(ctx_pos));
                            close_menu = true;
                        }
                    }

                    // Group / template actions
                    if !self.selected_nodes.is_empty() {
                        ui.separator();
                        if ui.button("Group Selected").clicked() {
                            self.snapshot();
                            let mut group = crate::model::group::NodeGroup::new("Group");
                            group.node_ids = self.selected_nodes.iter().copied().collect();
                            self.graph.groups.push(group);
                            close_menu = true;
                        }
                        let matching_group_idx = self.graph.groups.iter().position(|g| {
                            self.selected_nodes.iter().any(|id| g.node_ids.contains(id))
                        });
                        if matching_group_idx.is_some()
                            && ui.button("Ungroup").clicked()
                        {
                            self.snapshot();
                            self.graph.groups.retain(|g| {
                                !self.selected_nodes.iter().any(|id| g.node_ids.contains(id))
                            });
                            close_menu = true;
                        }
                        if let Some(idx) = matching_group_idx {
                            let is_collapsed = self.graph.groups[idx].collapsed;
                            let label = if is_collapsed { "Expand Group" } else { "Collapse Group" };
                            if ui.button(label).clicked() {
                                self.snapshot();
                                self.graph.groups[idx].collapsed = !is_collapsed;
                                close_menu = true;
                            }
                        }
                        if ui.button("Save as Template").clicked() {
                            self.dock_add_tab(super::dock::DockTab::Templates);
                            close_menu = true;
                        }
                    }

                    // Insert template
                    if !self.template_library.templates.is_empty() {
                        ui.separator();
                        ui.label("Insert Template");
                        let templates: Vec<_> = self
                            .template_library
                            .templates
                            .iter()
                            .map(|t| (t.id, t.name.clone()))
                            .collect();
                        for (tid, tname) in templates {
                            if ui.button(&tname).clicked() {
                                if let Some(t) = self
                                    .template_library
                                    .templates
                                    .iter()
                                    .find(|t| t.id == tid)
                                    .cloned()
                                {
                                    self.insert_template(&t, ctx_pos);
                                }
                                close_menu = true;
                            }
                        }
                    }
                });
            });

        if close_menu
            || response.clicked()
            || response.clicked_by(egui::PointerButton::Primary)
        {
            self.context_menu_pos = None;
        }
    }
}
