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

    /// Handle pan (middle mouse drag) and zoom (ctrl+scroll).
    pub fn handle_input(&mut self, response: &egui::Response, ui: &egui::Ui) {
        // Pan with middle mouse button drag
        if response.dragged_by(egui::PointerButton::Middle) {
            self.pan_offset += response.drag_delta();
        }

        // Zoom with Ctrl + scroll
        if response.hovered() {
            let scroll = ui.input(|i| i.smooth_scroll_delta.y);
            if ui.input(|i| i.modifiers.ctrl) && scroll != 0.0 {
                let zoom_factor = 1.0 + scroll * 0.002;
                let new_zoom = (self.zoom * zoom_factor).clamp(ZOOM_MIN, ZOOM_MAX);

                // Zoom toward cursor position
                if let Some(cursor) = response.hover_pos() {
                    let zoom_ratio = new_zoom / self.zoom;
                    self.pan_offset = cursor.to_vec2()
                        - (cursor.to_vec2() - self.pan_offset) * zoom_ratio;
                }
                self.zoom = new_zoom;
            }
        }
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
