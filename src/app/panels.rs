use egui::{Color32, Pos2, Rect, Sense, Stroke, StrokeKind};

use crate::ui::node_widget;
use crate::validation::validator::Severity;

use super::TaleNodeApp;

impl TaleNodeApp {
    pub(super) fn show_status_bar(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.label(&self.project_name);
            ui.separator();
            ui.label(format!("Nodes: {}", self.graph.nodes.len()));
            ui.separator();
            ui.label(format!("Connections: {}", self.graph.connections.len()));
            ui.separator();
            ui.label(format!("Zoom: {:.0}%", self.canvas.zoom * 100.0));
            if !self.selected_nodes.is_empty() {
                ui.separator();
                ui.label(format!("Selected: {}", self.selected_nodes.len()));
            }

            // Status message (auto-save, errors, etc.)
            if let Some((ref msg, when, is_error)) = self.status_message {
                if when.elapsed().as_secs() < 5 {
                    ui.separator();
                    let color = if is_error {
                        Color32::from_rgb(255, 100, 100)
                    } else {
                        Color32::from_rgb(100, 200, 100)
                    };
                    ui.colored_label(color, msg);
                }
            }

            // Validation indicator (right-aligned)
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                let errors = self
                    .validation_warnings
                    .iter()
                    .filter(|w| w.severity == Severity::Error)
                    .count();
                let warns = self
                    .validation_warnings
                    .iter()
                    .filter(|w| w.severity == Severity::Warning)
                    .count();

                let label = if errors > 0 {
                    format!("{errors} error(s), {warns} warning(s)")
                } else if warns > 0 {
                    format!("{warns} warning(s)")
                } else {
                    "No issues".to_string()
                };

                let color = if errors > 0 {
                    Color32::from_rgb(255, 100, 100)
                } else if warns > 0 {
                    Color32::from_rgb(255, 200, 100)
                } else {
                    Color32::from_rgb(100, 200, 100)
                };

                if ui
                    .add(
                        egui::Label::new(egui::RichText::new(label).color(color))
                            .sense(Sense::click()),
                    )
                    .clicked()
                {
                    self.dock_toggle_tab(super::dock::DockTab::Validation);
                }
            });
        });
    }

    pub(super) fn show_validation_panel(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.heading("Validation");
        });
        ui.separator();

        if self.validation_warnings.is_empty() {
            ui.label("No issues found.");
        } else {
            egui::ScrollArea::vertical()
                .max_height(200.0)
                .show(ui, |ui| {
                    for warning in &self.validation_warnings {
                        let (icon, color) = match warning.severity {
                            Severity::Error => ("E", Color32::from_rgb(255, 100, 100)),
                            Severity::Warning => ("!", Color32::from_rgb(255, 200, 100)),
                        };

                        ui.horizontal(|ui| {
                            ui.colored_label(color, icon);
                            let resp = ui.label(&warning.message);
                            if let Some(node_id) = warning.node_id {
                                if resp.interact(Sense::click()).clicked() {
                                    self.selected_nodes.clear();
                                    self.selected_nodes.insert(node_id);
                                    if let Some(node) = self.graph.nodes.get(&node_id) {
                                        self.canvas.pan_offset = egui::Vec2::new(
                                            -node.position[0] * self.canvas.zoom,
                                            -node.position[1] * self.canvas.zoom,
                                        );
                                    }
                                }
                            }
                        });
                    }
                });
        }
    }

    /// Compute the bounding rect of a group in canvas coordinates.
    fn group_bounds(&self, group: &crate::model::group::NodeGroup) -> Option<Rect> {
        let mut min_x = f32::MAX;
        let mut min_y = f32::MAX;
        let mut max_x = f32::MIN;
        let mut max_y = f32::MIN;
        for &nid in &group.node_ids {
            if let Some(node) = self.graph.nodes.get(&nid) {
                let r = node_widget::node_rect(node);
                min_x = min_x.min(r.min.x);
                min_y = min_y.min(r.min.y);
                max_x = max_x.max(r.max.x);
                max_y = max_y.max(r.max.y);
            }
        }
        if min_x >= max_x || min_y >= max_y {
            return None;
        }
        let pad = 20.0;
        Some(Rect::from_min_max(
            Pos2::new(min_x - pad, min_y - pad - 20.0),
            Pos2::new(max_x + pad, max_y + pad),
        ))
    }

    /// Build the set of node IDs hidden by collapsed groups.
    pub(super) fn hidden_by_collapsed_groups(&self) -> std::collections::HashSet<uuid::Uuid> {
        let mut hidden = std::collections::HashSet::new();
        for group in &self.graph.groups {
            if group.collapsed {
                for &nid in &group.node_ids {
                    hidden.insert(nid);
                }
            }
        }
        hidden
    }

    pub(super) fn draw_groups(&self, painter: &egui::Painter) {
        let clip = painter.clip_rect();
        for group in &self.graph.groups {
            if group.node_ids.is_empty() {
                continue;
            }
            let Some(group_rect) = self.group_bounds(group) else { continue };
            let screen_rect = self.canvas.canvas_rect_to_screen(group_rect);

            // Viewport culling
            if !clip.intersects(screen_rect) {
                continue;
            }

            let [r, g, b, a] = group.color;
            let border_a = (a as u16 * 3).min(255) as u8;

            if group.collapsed {
                // Collapsed: compact header-only rectangle
                let collapsed_h = 32.0 * self.canvas.zoom;
                let collapsed_rect = Rect::from_min_size(
                    screen_rect.min,
                    egui::Vec2::new(screen_rect.width(), collapsed_h),
                );
                painter.rect_filled(collapsed_rect, 8.0, Color32::from_rgba_premultiplied(r, g, b, 80));
                painter.rect_stroke(
                    collapsed_rect, 8.0,
                    Stroke::new(1.5, Color32::from_rgba_premultiplied(r, g, b, border_a)),
                    StrokeKind::Inside,
                );
                let font_size = 12.0 * self.canvas.zoom;
                let label = format!("{} ({} nodes)", group.name, group.node_ids.len());
                painter.text(
                    Pos2::new(collapsed_rect.min.x + 8.0, collapsed_rect.center().y),
                    egui::Align2::LEFT_CENTER, &label,
                    egui::FontId::proportional(font_size),
                    Color32::from_rgba_premultiplied(r, g, b, 220),
                );
            } else {
                painter.rect_filled(screen_rect, 8.0, Color32::from_rgba_premultiplied(r, g, b, a));
                painter.rect_stroke(
                    screen_rect, 8.0,
                    Stroke::new(1.0, Color32::from_rgba_premultiplied(r, g, b, border_a)),
                    StrokeKind::Inside,
                );
                let font_size = 12.0 * self.canvas.zoom;
                painter.text(
                    Pos2::new(screen_rect.min.x + 8.0, screen_rect.min.y + 4.0),
                    egui::Align2::LEFT_TOP, &group.name,
                    egui::FontId::proportional(font_size),
                    Color32::from_rgba_premultiplied(r, g, b, 200),
                );
            }
        }
    }

    fn compute_minimap_bounds(&mut self) {
        if !self.minimap_bounds_dirty {
            return;
        }
        self.minimap_bounds_dirty = false;
        let mut min_x = f32::MAX;
        let mut min_y = f32::MAX;
        let mut max_x = f32::MIN;
        let mut max_y = f32::MIN;
        for node in self.graph.nodes.values() {
            let rect = node_widget::node_rect(node);
            min_x = min_x.min(rect.min.x);
            min_y = min_y.min(rect.min.y);
            max_x = max_x.max(rect.max.x);
            max_y = max_y.max(rect.max.y);
        }
        if min_x < max_x && min_y < max_y {
            let margin = 200.0;
            self.minimap_bounds_cache = Some(Rect::from_min_max(
                Pos2::new(min_x - margin, min_y - margin),
                Pos2::new(max_x + margin, max_y + margin),
            ));
        } else {
            self.minimap_bounds_cache = None;
        }
    }

    pub(super) fn draw_minimap(
        &mut self,
        ui: &egui::Ui,
        painter: &egui::Painter,
        canvas_rect: Rect,
    ) {
        if self.graph.nodes.is_empty() {
            return;
        }
        self.compute_minimap_bounds();
        let Some(world) = self.minimap_bounds_cache else { return };

        let max_dim = 180.0;
        let aspect = world.width() / world.height();
        let (mw, mh) = if aspect > 1.0 {
            (max_dim, (max_dim / aspect).max(60.0))
        } else {
            ((max_dim * aspect).max(60.0), max_dim)
        };
        let minimap_rect = Rect::from_min_size(
            Pos2::new(canvas_rect.max.x - mw - 10.0, canvas_rect.max.y - mh - 10.0),
            egui::Vec2::new(mw, mh),
        );
        painter.rect_filled(minimap_rect, 4.0, Color32::from_rgba_premultiplied(30, 30, 30, 200));
        painter.rect_stroke(minimap_rect, 4.0, Stroke::new(1.0, Color32::from_rgb(80, 80, 80)), StrokeKind::Inside);

        let inner = minimap_rect.shrink(4.0);
        let scale = (inner.width() / world.width()).min(inner.height() / world.height());
        let map = |p: Pos2| Pos2::new(inner.min.x + (p.x - world.min.x) * scale, inner.min.y + (p.y - world.min.y) * scale);
        let unmap = |p: Pos2| Pos2::new((p.x - inner.min.x) / scale + world.min.x, (p.y - inner.min.y) / scale + world.min.y);

        let use_dots = self.graph.nodes.len() > 500;
        for node in self.graph.nodes.values() {
            let color = node_widget::node_color(&node.node_type);
            if use_dots {
                let center = map(Pos2::new(node.position[0], node.position[1]));
                painter.circle_filled(center, 1.5, color);
            } else {
                let rect = node_widget::node_rect(node);
                painter.rect_filled(Rect::from_min_max(map(rect.min), map(rect.max)), 1.0, color);
            }
        }

        let vp_min = map(self.canvas.screen_to_canvas(canvas_rect.min));
        let vp_max = map(self.canvas.screen_to_canvas(canvas_rect.max));
        let vp_rect = Rect::from_min_max(vp_min, vp_max).intersect(inner);
        painter.rect_stroke(vp_rect, 1.0, Stroke::new(1.0, Color32::from_rgb(200, 200, 200)), StrokeKind::Inside);

        let pointer_pos = ui.input(|i| i.pointer.interact_pos()).unwrap_or(Pos2::ZERO);
        if ui.input(|i| i.pointer.primary_down()) && minimap_rect.contains(pointer_pos) {
            let target = unmap(pointer_pos);
            let half = egui::Vec2::new(canvas_rect.width() * 0.5, canvas_rect.height() * 0.5);
            self.canvas.pan_offset = egui::Vec2::new(
                half.x + canvas_rect.min.x - target.x * self.canvas.zoom,
                half.y + canvas_rect.min.y - target.y * self.canvas.zoom,
            );
        }
    }
}
