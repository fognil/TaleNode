use egui::{Color32, Pos2, Rect, Sense, Stroke, StrokeKind};
use uuid::Uuid;

use crate::model::port::{PortDirection, PortId};
use crate::ui::connection_renderer::{draw_bezier_wire, draw_connections};
use crate::ui::node_widget::{self, draw_node, PORT_RADIUS};

use super::{DragWire, InteractionState, TaleNodeApp};

impl TaleNodeApp {
    /// Hit-test: find node under screen position (topmost first).
    pub(super) fn node_at_screen_pos(&self, screen_pos: Pos2) -> Option<Uuid> {
        for node in self.graph.nodes.values() {
            let rect = node_widget::node_rect(node);
            let screen_rect = self.canvas.canvas_rect_to_screen(rect);
            if screen_rect.contains(screen_pos) {
                return Some(node.id);
            }
        }
        None
    }

    /// Hit-test: find port under screen position. Returns (node_id, port_id, direction).
    pub(super) fn port_at_screen_pos(
        &self,
        screen_pos: Pos2,
    ) -> Option<(Uuid, PortId, PortDirection)> {
        let hit_radius = (PORT_RADIUS + 4.0) * self.canvas.zoom;

        for node in self.graph.nodes.values() {
            for (i, port) in node.outputs.iter().enumerate() {
                let port_canvas = node_widget::port_position(node, i, true);
                let port_screen = self.canvas.canvas_to_screen(port_canvas);
                if screen_pos.distance(port_screen) <= hit_radius {
                    return Some((node.id, port.id, PortDirection::Output));
                }
            }
            for (i, port) in node.inputs.iter().enumerate() {
                let port_canvas = node_widget::port_position(node, i, false);
                let port_screen = self.canvas.canvas_to_screen(port_canvas);
                if screen_pos.distance(port_screen) <= hit_radius {
                    return Some((node.id, port.id, PortDirection::Input));
                }
            }
        }
        None
    }

    pub(super) fn show_canvas(&mut self, ui: &mut egui::Ui) {
        let canvas_rect = ui.available_rect_before_wrap();
        let response = ui.allocate_rect(canvas_rect, Sense::click_and_drag());
        let painter = ui.painter_at(canvas_rect);

        // Handle pan/zoom
        self.canvas.handle_input(&response, ui);

        // Draw grid
        self.canvas.draw_grid(&painter, canvas_rect);

        // Draw groups (below connections and nodes)
        self.draw_groups(&painter);

        // Draw connections (below nodes)
        draw_connections(&painter, &self.graph, &self.canvas, None);

        // Draw nodes
        let node_ids: Vec<Uuid> = self.graph.nodes.keys().copied().collect();
        for id in &node_ids {
            if let Some(node) = self.graph.nodes.get(id) {
                let is_selected = self.selected_nodes.contains(id);
                let is_search_match = self.search_results.contains(id);
                let review_status = self.graph.get_review_status(*id);
                draw_node(
                    &painter,
                    node,
                    &self.canvas,
                    is_selected,
                    is_search_match,
                    &self.graph.characters,
                    review_status,
                );
            }
        }

        // Draw dragging wire
        if let InteractionState::DraggingWire(ref drag) = self.interaction {
            let from_node = self.graph.nodes.get(&drag.from_node);
            if let Some(node) = from_node {
                let is_output = drag.from_direction == PortDirection::Output;
                let port_index = if is_output {
                    node.outputs.iter().position(|p| p.id == drag.from_port)
                } else {
                    node.inputs.iter().position(|p| p.id == drag.from_port)
                };
                if let Some(idx) = port_index {
                    let port_canvas = node_widget::port_position(node, idx, is_output);
                    let port_screen = self.canvas.canvas_to_screen(port_canvas);
                    let (from, to) = if is_output {
                        (port_screen, drag.cursor_pos)
                    } else {
                        (drag.cursor_pos, port_screen)
                    };
                    draw_bezier_wire(
                        &painter,
                        from,
                        to,
                        Color32::from_rgb(255, 255, 100),
                        self.canvas.zoom,
                    );
                }
            }
        }

        // Draw box selection
        if let InteractionState::BoxSelecting { start } = self.interaction {
            if let Some(cursor) = response.hover_pos() {
                let sel_rect = Rect::from_two_pos(start, cursor);
                painter.rect_filled(
                    sel_rect,
                    0.0,
                    Color32::from_rgba_premultiplied(100, 150, 255, 30),
                );
                painter.rect_stroke(
                    sel_rect,
                    0.0,
                    Stroke::new(1.0, Color32::from_rgb(100, 150, 255)),
                    StrokeKind::Outside,
                );
            }
        }

        // Draw minimap overlay
        self.draw_minimap(ui, &painter, canvas_rect);

        // === INTERACTION HANDLING ===
        self.handle_interactions(&response, canvas_rect);

        // === CONTEXT MENU ===
        self.handle_context_menu(&response);
    }

    fn handle_interactions(&mut self, response: &egui::Response, _canvas_rect: Rect) {
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
                    self.selected_nodes.push(node_id);
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
                    let ids: Vec<Uuid> = self.selected_nodes.clone();
                    for id in ids {
                        if let Some(node) = self.graph.nodes.get_mut(&id) {
                            node.position[0] += delta.x;
                            node.position[1] += delta.y;
                        }
                    }
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
                    self.selected_nodes.clear();
                    for node in self.graph.nodes.values() {
                        let node_rect = node_widget::node_rect(node);
                        let screen_rect = self.canvas.canvas_rect_to_screen(node_rect);
                        if sel_rect.intersects(screen_rect) {
                            self.selected_nodes.push(node.id);
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

        // Click to select node or deselect on empty space
        if response.clicked() && matches!(self.interaction, InteractionState::Idle) {
            if let Some(node_id) = self.node_at_screen_pos(pointer_pos) {
                self.selected_nodes.clear();
                self.selected_nodes.push(node_id);
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
                let ids: Vec<Uuid> = self.selected_nodes.drain(..).collect();
                for id in ids {
                    self.graph.remove_node(id);
                }
            }
        }
    }

    fn handle_context_menu(&mut self, response: &egui::Response) {
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

                    // Group actions
                    if !self.selected_nodes.is_empty() {
                        ui.separator();
                        if ui.button("Group Selected").clicked() {
                            self.snapshot();
                            let mut group = crate::model::group::NodeGroup::new("Group");
                            group.node_ids = self.selected_nodes.clone();
                            self.graph.groups.push(group);
                            close_menu = true;
                        }
                        let has_group = self.graph.groups.iter().any(|g| {
                            self.selected_nodes.iter().any(|id| g.node_ids.contains(id))
                        });
                        if has_group && ui.button("Ungroup").clicked() {
                            self.snapshot();
                            self.graph.groups.retain(|g| {
                                !self.selected_nodes.iter().any(|id| g.node_ids.contains(id))
                            });
                            close_menu = true;
                        }
                    }
                });
            });

        if close_menu || response.clicked() || response.clicked_by(egui::PointerButton::Primary) {
            self.context_menu_pos = None;
        }
    }

    /// Duplicate selected nodes with offset.
    pub(super) fn duplicate_selected(&mut self) {
        if self.selected_nodes.is_empty() {
            return;
        }
        self.snapshot();
        let mut new_ids = Vec::new();
        for &id in &self.selected_nodes.clone() {
            if let Some(node) = self.graph.nodes.get(&id) {
                let mut dup = node.clone();
                dup.id = Uuid::new_v4();
                dup.position[0] += 30.0;
                dup.position[1] += 30.0;
                for port in &mut dup.inputs {
                    port.id = crate::model::port::PortId(Uuid::new_v4());
                }
                for port in &mut dup.outputs {
                    port.id = crate::model::port::PortId(Uuid::new_v4());
                }
                if let crate::model::node::NodeType::SubGraph(ref mut data) = dup.node_type {
                    regenerate_child_ids(&mut data.child_graph);
                }
                new_ids.push(dup.id);
                self.graph.add_node(dup);
            }
        }
        self.selected_nodes = new_ids;
    }
}
fn regenerate_child_ids(g: &mut crate::model::graph::DialogueGraph) {
    let (mut ids, mut ports) = (std::collections::HashMap::new(), std::collections::HashMap::new());
    for (oid, mut n) in g.nodes.drain().collect::<Vec<_>>() {
        let nid = Uuid::new_v4();
        ids.insert(oid, nid);
        n.id = nid;
        for p in n.inputs.iter_mut().chain(n.outputs.iter_mut()) {
            let np = PortId(Uuid::new_v4()); ports.insert(p.id, np); p.id = np;
        }
        g.nodes.insert(nid, n);
    }
    for c in &mut g.connections {
        c.id = Uuid::new_v4();
        c.from_node = ids.get(&c.from_node).copied().unwrap_or(c.from_node);
        c.to_node = ids.get(&c.to_node).copied().unwrap_or(c.to_node);
        c.from_port = ports.get(&c.from_port).copied().unwrap_or(c.from_port);
        c.to_port = ports.get(&c.to_port).copied().unwrap_or(c.to_port);
    }
}
