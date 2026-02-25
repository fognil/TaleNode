use egui::{Color32, Pos2, Rect, Stroke, Vec2};

/// Persistent state for the canvas (pan, zoom).
#[derive(Debug, Clone)]
pub struct CanvasState {
    pub pan_offset: Vec2,
    pub zoom: f32,
}

impl Default for CanvasState {
    fn default() -> Self {
        Self {
            pan_offset: Vec2::ZERO,
            zoom: 1.0,
        }
    }
}

const ZOOM_MIN: f32 = 0.25;
const ZOOM_MAX: f32 = 4.0;
const GRID_SPACING: f32 = 40.0;

impl CanvasState {
    /// Convert screen position to canvas position.
    pub fn screen_to_canvas(&self, screen_pos: Pos2) -> Pos2 {
        Pos2::new(
            (screen_pos.x - self.pan_offset.x) / self.zoom,
            (screen_pos.y - self.pan_offset.y) / self.zoom,
        )
    }

    /// Convert canvas position to screen position.
    pub fn canvas_to_screen(&self, canvas_pos: Pos2) -> Pos2 {
        Pos2::new(
            canvas_pos.x * self.zoom + self.pan_offset.x,
            canvas_pos.y * self.zoom + self.pan_offset.y,
        )
    }

    /// Convert a canvas-space rect to screen-space rect.
    pub fn canvas_rect_to_screen(&self, rect: Rect) -> Rect {
        Rect::from_min_max(
            self.canvas_to_screen(rect.min),
            self.canvas_to_screen(rect.max),
        )
    }

    /// Handle pan and zoom input.
    /// - Middle mouse drag or Space+left drag: pan
    /// - Scroll wheel: zoom
    /// - Trackpad pinch: zoom
    pub fn handle_input(&mut self, response: &egui::Response, ui: &egui::Ui) {
        // Pan with middle mouse button drag
        if response.dragged_by(egui::PointerButton::Middle) {
            self.pan_offset += response.drag_delta();
        }

        // Pan with Space + left mouse drag
        if response.dragged_by(egui::PointerButton::Primary)
            && ui.input(|i| i.key_down(egui::Key::Space))
        {
            self.pan_offset += response.drag_delta();
        }

        if response.hovered() {
            // Trackpad pinch-to-zoom (macOS)
            let pinch = ui.input(|i| i.zoom_delta());
            if pinch != 1.0 {
                let new_zoom = (self.zoom * pinch).clamp(ZOOM_MIN, ZOOM_MAX);
                if let Some(cursor) = response.hover_pos() {
                    let zoom_ratio = new_zoom / self.zoom;
                    self.pan_offset = cursor.to_vec2()
                        - (cursor.to_vec2() - self.pan_offset) * zoom_ratio;
                }
                self.zoom = new_zoom;
            }

            // Scroll wheel zoom (Ctrl+scroll or Cmd+scroll)
            let scroll = ui.input(|i| i.smooth_scroll_delta.y);
            let has_modifier = ui.input(|i| i.modifiers.ctrl || i.modifiers.command);
            if has_modifier && scroll != 0.0 {
                let zoom_factor = 1.0 + scroll * 0.002;
                let new_zoom = (self.zoom * zoom_factor).clamp(ZOOM_MIN, ZOOM_MAX);
                if let Some(cursor) = response.hover_pos() {
                    let zoom_ratio = new_zoom / self.zoom;
                    self.pan_offset = cursor.to_vec2()
                        - (cursor.to_vec2() - self.pan_offset) * zoom_ratio;
                }
                self.zoom = new_zoom;
            }

            // Scroll without modifier: pan
            if !has_modifier && scroll != 0.0 {
                let scroll_x = ui.input(|i| i.smooth_scroll_delta.x);
                self.pan_offset.x += scroll_x;
                self.pan_offset.y += scroll;
            }
        }
    }

    /// Set pan and zoom to fit all nodes in view.
    pub fn zoom_to_fit(
        &mut self,
        nodes: &std::collections::HashMap<uuid::Uuid, crate::model::node::Node>,
        screen_size: Vec2,
    ) {
        if nodes.is_empty() {
            return;
        }
        let mut min_x = f32::MAX;
        let mut min_y = f32::MAX;
        let mut max_x = f32::MIN;
        let mut max_y = f32::MIN;
        for node in nodes.values() {
            let rect = crate::ui::node_widget::node_rect(node);
            min_x = min_x.min(rect.min.x);
            min_y = min_y.min(rect.min.y);
            max_x = max_x.max(rect.max.x);
            max_y = max_y.max(rect.max.y);
        }
        let margin = 80.0;
        let content_w = (max_x - min_x).max(1.0);
        let content_h = (max_y - min_y).max(1.0);
        let zoom_x = (screen_size.x - margin * 2.0) / content_w;
        let zoom_y = (screen_size.y - margin * 2.0) / content_h;
        self.zoom = zoom_x.min(zoom_y).clamp(ZOOM_MIN, ZOOM_MAX);
        let center_x = (min_x + max_x) / 2.0;
        let center_y = (min_y + max_y) / 2.0;
        self.pan_offset = Vec2::new(
            screen_size.x / 2.0 - center_x * self.zoom,
            screen_size.y / 2.0 - center_y * self.zoom,
        );
    }

    /// Draw the background grid.
    pub fn draw_grid(&self, painter: &egui::Painter, canvas_rect: Rect) {
        let bg_color = Color32::from_rgb(30, 30, 30);
        painter.rect_filled(canvas_rect, 0.0, bg_color);

        let grid_color = Color32::from_rgb(45, 45, 45);
        let grid_color_major = Color32::from_rgb(55, 55, 55);
        let spacing = GRID_SPACING * self.zoom;

        if spacing < 4.0 {
            return; // Too zoomed out, skip grid
        }

        // Calculate grid start positions
        let offset_x = self.pan_offset.x % spacing;
        let offset_y = self.pan_offset.y % spacing;

        // Count lines for major grid detection
        let start_ix =
            ((canvas_rect.min.x - self.pan_offset.x) / spacing).floor() as i32;
        let start_iy =
            ((canvas_rect.min.y - self.pan_offset.y) / spacing).floor() as i32;

        // Vertical lines
        let mut x = canvas_rect.min.x + offset_x;
        let mut ix = start_ix;
        while x <= canvas_rect.max.x {
            let color = if ix % 5 == 0 {
                grid_color_major
            } else {
                grid_color
            };
            painter.line_segment(
                [Pos2::new(x, canvas_rect.min.y), Pos2::new(x, canvas_rect.max.y)],
                Stroke::new(1.0, color),
            );
            x += spacing;
            ix += 1;
        }

        // Horizontal lines
        let mut y = canvas_rect.min.y + offset_y;
        let mut iy = start_iy;
        while y <= canvas_rect.max.y {
            let color = if iy % 5 == 0 {
                grid_color_major
            } else {
                grid_color
            };
            painter.line_segment(
                [Pos2::new(canvas_rect.min.x, y), Pos2::new(canvas_rect.max.x, y)],
                Stroke::new(1.0, color),
            );
            y += spacing;
            iy += 1;
        }
    }
}
