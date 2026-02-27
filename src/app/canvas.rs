use egui::{Color32, Pos2, Rect, Sense, Stroke, StrokeKind};
use uuid::Uuid;

use crate::model::port::{PortDirection, PortId};
use crate::ui::connection_renderer::{draw_bezier_wire, draw_connections};
use crate::ui::node_widget::{self, draw_node, PORT_RADIUS};

use super::canvas_tooltip::node_tooltip_text;
use super::{InteractionState, TaleNodeApp};

impl TaleNodeApp {
    /// Hit-test: find node under screen position using spatial grid.
    pub(super) fn node_at_screen_pos(&self, screen_pos: Pos2) -> Option<Uuid> {
        let canvas_pos = self.canvas.screen_to_canvas(screen_pos);
        let candidates = self.spatial_grid.query_point(canvas_pos.x, canvas_pos.y);
        for id in candidates {
            if let Some(node) = self.graph.nodes.get(&id) {
                let rect = node_widget::node_rect(node);
                let screen_rect = self.canvas.canvas_rect_to_screen(rect);
                if screen_rect.contains(screen_pos) {
                    return Some(node.id);
                }
            }
        }
        None
    }

    /// Hit-test: find port under screen position using spatial grid.
    pub(super) fn port_at_screen_pos(
        &self,
        screen_pos: Pos2,
    ) -> Option<(Uuid, PortId, PortDirection)> {
        let hit_radius = (PORT_RADIUS + 4.0) * self.canvas.zoom;
        let canvas_pos = self.canvas.screen_to_canvas(screen_pos);
        let candidates = self.spatial_grid.query_point(canvas_pos.x, canvas_pos.y);

        for id in candidates {
            let Some(node) = self.graph.nodes.get(&id) else { continue };
            if node.collapsed {
                continue;
            }
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

        self.spatial_grid.rebuild_if_dirty(&self.graph.nodes);
        if self.pending_zoom_fit {
            self.pending_zoom_fit = false;
            self.canvas.zoom_to_fit(&self.graph.nodes, canvas_rect.size());
        }
        self.canvas.handle_input(&response, ui);
        self.canvas.draw_grid(&painter, canvas_rect);
        self.draw_groups(&painter);

        let canvas_viewport = egui::Rect::from_min_max(
            self.canvas.screen_to_canvas(canvas_rect.min),
            self.canvas.screen_to_canvas(canvas_rect.max),
        );

        // Nodes hidden by collapsed groups
        let hidden_nodes = self.hidden_by_collapsed_groups();

        // Draw connections (below nodes) — culled by viewport
        draw_connections(&painter, &self.graph, &self.canvas, None, canvas_viewport, &hidden_nodes);

        // Detect port hover for visual feedback (skip at low zoom — ports not visible)
        let hovered_port_info = if self.canvas.zoom >= crate::ui::node_widget::LOD_MEDIUM_ZOOM
            && response.hovered()
        {
            response.hover_pos().and_then(|hp| self.port_at_screen_pos(hp))
        } else {
            None
        };
        if hovered_port_info.is_some() {
            ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
        }

        // Draw nodes
        let playtest_node = if self.playtest.running {
            self.playtest.current_node
        } else {
            None
        };
        let filter_active = self.canvas_filter.active && !self.canvas_filter.visible_cache.is_empty();
        let node_ids: Vec<Uuid> = self.graph.nodes.keys().copied().collect();
        for id in &node_ids {
            if hidden_nodes.contains(id) {
                continue;
            }
            if filter_active && !self.canvas_filter.visible_cache.contains(id) {
                continue;
            }
            if let Some(node) = self.graph.nodes.get(id) {
                let is_selected = self.selected_nodes.contains(id);
                let is_search_match = self.search_results_set.contains(id);
                let review_status = self.graph.get_review_status(*id);
                let hover_port = hovered_port_info
                    .and_then(|(nid, pid, _)| if nid == *id { Some(pid) } else { None });
                let project_dir = self.project_path.as_ref().and_then(|p| p.parent());
                draw_node(
                    &painter,
                    node,
                    &self.canvas,
                    is_selected,
                    is_search_match,
                    &self.graph.characters,
                    review_status,
                    hover_port,
                    playtest_node == Some(*id),
                    &mut self.portrait_cache,
                    project_dir,
                );
            }
        }

        // Node tooltip on hover (skip at low zoom — text not readable)
        if self.canvas.zoom >= crate::ui::node_widget::LOD_MEDIUM_ZOOM
            && hovered_port_info.is_none()
            && matches!(self.interaction, InteractionState::Idle)
        {
            if let Some(hp) = response.hover_pos() {
                if let Some(nid) = self.node_at_screen_pos(hp) {
                    if let Some(node) = self.graph.nodes.get(&nid) {
                        let tip = node_tooltip_text(node);
                        if !tip.is_empty() {
                            egui::show_tooltip_at_pointer(
                                ui.ctx(),
                                ui.layer_id(),
                                egui::Id::new("node_tooltip"),
                                |ui| {
                                    ui.set_max_width(350.0);
                                    ui.label(&tip);
                                },
                            );
                        }
                    }
                }
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
                let sel_color = Color32::from_rgb(100, 150, 255);
                painter.rect_filled(sel_rect, 0.0, Color32::from_rgba_premultiplied(100, 150, 255, 30));
                painter.rect_stroke(sel_rect, 0.0, Stroke::new(1.0, sel_color), StrokeKind::Outside);
            }
        }

        // Request continuous repaint for playtest glow animation
        if self.playtest.running {
            ui.ctx().request_repaint();
        }

        // Draw minimap overlay
        self.draw_minimap(ui, &painter, canvas_rect);

        // === INTERACTION HANDLING ===
        self.handle_interactions(&response, canvas_rect);

        // === CONTEXT MENU ===
        self.handle_context_menu(&response);
    }

}
