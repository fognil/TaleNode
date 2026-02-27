use std::collections::HashSet;

use egui::{Pos2, Rect};
use uuid::Uuid;

use crate::model::port::PortDirection;
use crate::ui::node_widget;

use super::{DragWire, InteractionState, TaleNodeApp};

impl TaleNodeApp {
    pub(super) fn handle_interactions(&mut self, response: &egui::Response, _canvas_rect: Rect) {
        let pointer_pos = response.hover_pos().unwrap_or(Pos2::ZERO);

        // Right click -> context menu
        if response.secondary_clicked() {
            let canvas_pos = self.canvas.screen_to_canvas(pointer_pos);
            self.context_menu_pos = Some([canvas_pos.x, canvas_pos.y]);
        }

        // Left click down -> start interaction
        if response.drag_started_by(egui::PointerButton::Primary) {
            // Check port first
            if let Some((node_id, port_id, direction)) = self.port_at_screen_pos(pointer_pos) {
                self.interaction = InteractionState::DraggingWire(DragWire {
                    from_node: node_id,
                    from_port: port_id,
                    from_direction: direction,
                    cursor_pos: pointer_pos,
                });
            }
            // Check node
            else if let Some(node_id) = self.node_at_screen_pos(pointer_pos) {
                if !self.selected_nodes.contains(&node_id) {
                    self.selected_nodes.clear();
                    self.selected_nodes.insert(node_id);
                }
                // Snapshot before dragging for undo
                self.snapshot();
                self.interaction = InteractionState::DraggingNodes;
            }
            // Empty space -> box select
            else {
                self.selected_nodes.clear();
                self.interaction = InteractionState::BoxSelecting {
                    start: pointer_pos,
                };
            }
        }

        // During drag
        if response.dragged_by(egui::PointerButton::Primary) {
            match &mut self.interaction {
                InteractionState::DraggingNodes => {
                    let delta = response.drag_delta() / self.canvas.zoom;
                    let ids: Vec<Uuid> = self.selected_nodes.iter().copied().collect();
                    for id in &ids {
                        if let Some(node) = self.graph.nodes.get_mut(id) {
                            node.position[0] += delta.x;
                            node.position[1] += delta.y;
                        }
                    }
                    self.spatial_grid.mark_dirty();
                }
                InteractionState::DraggingWire(drag) => {
                    drag.cursor_pos = pointer_pos;
                }
                InteractionState::BoxSelecting { .. } => {
                    // Visual update handled in draw
                }
                InteractionState::Idle => {}
            }
        }

        // Release
        if response.drag_stopped_by(egui::PointerButton::Primary) {
            let interaction = self.interaction.clone();
            match &interaction {
                InteractionState::DraggingWire(drag) => {
                    if let Some((target_node, target_port, target_dir)) =
                        self.port_at_screen_pos(pointer_pos)
                    {
                        match (drag.from_direction, target_dir) {
                            (PortDirection::Output, PortDirection::Input) => {
                                self.snapshot();
                                self.graph.add_connection(
                                    drag.from_node,
                                    drag.from_port,
                                    target_node,
                                    target_port,
                                );
                            }
                            (PortDirection::Input, PortDirection::Output) => {
                                self.snapshot();
                                self.graph.add_connection(
                                    target_node,
                                    target_port,
                                    drag.from_node,
                                    drag.from_port,
                                );
                            }
                            _ => {} // Same direction, ignore
                        }
                    }
                }
                InteractionState::BoxSelecting { start } => {
                    let sel_rect = Rect::from_two_pos(*start, pointer_pos);
                    let canvas_min = self.canvas.screen_to_canvas(sel_rect.min);
                    let canvas_max = self.canvas.screen_to_canvas(sel_rect.max);
                    let candidates = self.spatial_grid.query_rect(
                        canvas_min.x, canvas_min.y, canvas_max.x, canvas_max.y,
                    );
                    self.selected_nodes.clear();
                    for id in candidates {
                        if let Some(node) = self.graph.nodes.get(&id) {
                            let screen_rect =
                                self.canvas.canvas_rect_to_screen(node_widget::node_rect(node));
                            if sel_rect.intersects(screen_rect) {
                                self.selected_nodes.insert(node.id);
                            }
                        }
                    }
                }
                _ => {}
            }
            self.interaction = InteractionState::Idle;
        }

        // Double-click to enter sub-graph
        if response.double_clicked() {
            if let Some(node_id) = self.node_at_screen_pos(pointer_pos) {
                if let Some(node) = self.graph.nodes.get(&node_id) {
                    if matches!(node.node_type, crate::model::node::NodeType::SubGraph(_)) {
                        self.enter_subgraph(node_id);
                        return;
                    }
                }
            }
        }

        // Click to toggle collapse, select node, or deselect on empty space
        if response.clicked() && matches!(self.interaction, InteractionState::Idle) {
            let mut toggled = false;
            if let Some(node_id) = self.node_at_screen_pos(pointer_pos) {
                // Check if click hit the collapse toggle triangle
                if let Some(node) = self.graph.nodes.get(&node_id) {
                    let toggle_canvas = node_widget::collapse_toggle_rect(node);
                    let toggle_screen = self.canvas.canvas_rect_to_screen(toggle_canvas);
                    if toggle_screen.contains(pointer_pos) {
                        self.snapshot();
                        if let Some(node) = self.graph.nodes.get_mut(&node_id) {
                            node.collapsed = !node.collapsed;
                        }
                        toggled = true;
                    }
                }
                if !toggled {
                    self.selected_nodes.clear();
                    self.selected_nodes.insert(node_id);
                }
            } else {
                self.selected_nodes.clear();
            }
        }

        // Delete selected nodes
        if !self.selected_nodes.is_empty() {
            let delete_pressed = response.ctx.input(|i| {
                i.key_pressed(egui::Key::Delete) || i.key_pressed(egui::Key::Backspace)
            });
            if delete_pressed {
                self.snapshot();
                let ids: Vec<Uuid> = self.selected_nodes.drain().collect();
                for id in ids {
                    self.graph.remove_node(id);
                }
            }
        }
    }

    pub(super) fn duplicate_selected(&mut self) {
        if self.selected_nodes.is_empty() {
            return;
        }
        self.snapshot();
        let mut new_ids = HashSet::new();
        let old_ids: Vec<Uuid> = self.selected_nodes.iter().copied().collect();
        for id in &old_ids {
            if let Some(node) = self.graph.nodes.get(id) {
                let mut dup = node.clone();
                dup.id = Uuid::new_v4();
                dup.position[0] += 30.0;
                dup.position[1] += 30.0;
                for p in dup.inputs.iter_mut().chain(dup.outputs.iter_mut()) {
                    p.id = crate::model::port::PortId(Uuid::new_v4());
                }
                if let crate::model::node::NodeType::SubGraph(ref mut data) = dup.node_type {
                    super::templates::regenerate_child_ids(&mut data.child_graph);
                }
                new_ids.insert(dup.id);
                self.graph.add_node(dup);
            }
        }
        self.selected_nodes = new_ids;
    }
}
