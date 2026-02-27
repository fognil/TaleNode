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
                                    self.selected_nodes.push(node_id);
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

    pub(super) fn draw_minimap(
        &mut self,
        ui: &egui::Ui,
        painter: &egui::Painter,
        canvas_rect: Rect,
    ) {
        if self.graph.nodes.is_empty() {
            return;
        }

        let minimap_size = 160.0;
        let padding = 10.0;
        let minimap_rect = Rect::from_min_size(
            Pos2::new(
                canvas_rect.max.x - minimap_size - padding,
                canvas_rect.max.y - minimap_size - padding,
            ),
            egui::Vec2::splat(minimap_size),
        );

        // Background
        painter.rect_filled(
            minimap_rect,
            4.0,
            Color32::from_rgba_premultiplied(30, 30, 30, 200),
        );
        painter.rect_stroke(
            minimap_rect,
            4.0,
            Stroke::new(1.0, Color32::from_rgb(80, 80, 80)),
            StrokeKind::Inside,
        );

        // Compute bounding box of all nodes in canvas coords
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

        let margin = 200.0;
        min_x -= margin;
        min_y -= margin;
        max_x += margin;
        max_y += margin;

        let world_w = max_x - min_x;
        let world_h = max_y - min_y;
        if world_w <= 0.0 || world_h <= 0.0 {
            return;
        }

        let inner_margin = 4.0;
        let inner_rect = minimap_rect.shrink(inner_margin);
        let scale = (inner_rect.width() / world_w).min(inner_rect.height() / world_h);

        let map = |canvas_pos: Pos2| -> Pos2 {
            Pos2::new(
                inner_rect.min.x + (canvas_pos.x - min_x) * scale,
                inner_rect.min.y + (canvas_pos.y - min_y) * scale,
            )
        };

        let unmap = |screen_pos: Pos2| -> Pos2 {
            Pos2::new(
                (screen_pos.x - inner_rect.min.x) / scale + min_x,
                (screen_pos.y - inner_rect.min.y) / scale + min_y,
            )
        };

        // Draw nodes as small colored rectangles
        for node in self.graph.nodes.values() {
            let rect = node_widget::node_rect(node);
            let mapped_min = map(rect.min);
            let mapped_max = map(rect.max);
            let mapped_rect = Rect::from_min_max(mapped_min, mapped_max);
            let color = node_widget::node_color(&node.node_type);
            painter.rect_filled(mapped_rect, 1.0, color);
        }

        // Draw viewport rectangle
        let vp_min = self.canvas.screen_to_canvas(canvas_rect.min);
        let vp_max = self.canvas.screen_to_canvas(canvas_rect.max);
        let vp_mapped_min = map(vp_min);
        let vp_mapped_max = map(vp_max);
        let vp_rect =
            Rect::from_min_max(vp_mapped_min, vp_mapped_max).intersect(inner_rect);
        painter.rect_stroke(
            vp_rect,
            1.0,
            Stroke::new(1.0, Color32::from_rgb(200, 200, 200)),
            StrokeKind::Inside,
        );

        // Handle click/drag on minimap to navigate
        let pointer_pos = ui.input(|i| i.pointer.interact_pos()).unwrap_or(Pos2::ZERO);
        let pointer_down = ui.input(|i| i.pointer.primary_down());

        if pointer_down && minimap_rect.contains(pointer_pos) {
            let canvas_target = unmap(pointer_pos);
            let canvas_center = egui::Vec2::new(
                canvas_rect.width() * 0.5,
                canvas_rect.height() * 0.5,
            );
            self.canvas.pan_offset = egui::Vec2::new(
                canvas_center.x + canvas_rect.min.x - canvas_target.x * self.canvas.zoom,
                canvas_center.y + canvas_rect.min.y - canvas_target.y * self.canvas.zoom,
            );
        }
    }
}
