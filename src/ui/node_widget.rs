use egui::{Color32, CornerRadius, FontId, Pos2, Rect, Stroke, StrokeKind, Vec2};

use crate::model::node::{Node, NodeType};
use crate::ui::canvas::CanvasState;

/// Visual constants for node rendering.
const NODE_WIDTH: f32 = 200.0;
const NODE_HEADER_HEIGHT: f32 = 28.0;
const NODE_PORT_RADIUS: f32 = 6.0;
const NODE_PORT_Y_START: f32 = 44.0;
const NODE_PORT_Y_SPACING: f32 = 22.0;
const NODE_MIN_BODY_HEIGHT: f32 = 30.0;
const NODE_ROUNDING: u8 = 6;
const NODE_TEXT_PREVIEW_LINES: usize = 2;

pub const PORT_RADIUS: f32 = NODE_PORT_RADIUS;

/// Get the color associated with each node type.
fn node_color(node_type: &NodeType) -> Color32 {
    match node_type {
        NodeType::Start => Color32::from_rgb(76, 175, 80),       // green
        NodeType::Dialogue(_) => Color32::from_rgb(66, 133, 244), // blue
        NodeType::Choice(_) => Color32::from_rgb(251, 188, 4),    // yellow
        NodeType::Condition(_) => Color32::from_rgb(255, 152, 0), // orange
        NodeType::Event(_) => Color32::from_rgb(171, 71, 188),    // purple
        NodeType::Random(_) => Color32::from_rgb(158, 158, 158),  // gray
        NodeType::End(_) => Color32::from_rgb(244, 67, 54),       // red
    }
}

/// Compute the height of a node's body (below header).
fn node_body_height(node: &Node) -> f32 {
    let port_count = node.inputs.len().max(node.outputs.len());
    let ports_height = if port_count > 0 {
        NODE_PORT_Y_START - NODE_HEADER_HEIGHT
            + (port_count as f32) * NODE_PORT_Y_SPACING
    } else {
        0.0
    };

    // Extra height for dialogue text preview
    let text_height = match &node.node_type {
        NodeType::Dialogue(data) if !data.text.is_empty() => {
            let line_count = data.text.lines().take(NODE_TEXT_PREVIEW_LINES).count();
            (line_count.max(1) as f32) * 16.0 + 8.0
        }
        _ => 0.0,
    };

    (ports_height + text_height).max(NODE_MIN_BODY_HEIGHT)
}

/// Get the bounding rect of a node in canvas coordinates.
pub fn node_rect(node: &Node) -> Rect {
    let body_h = node_body_height(node);
    let total_h = NODE_HEADER_HEIGHT + body_h;
    Rect::from_min_size(
        Pos2::new(node.position[0], node.position[1]),
        Vec2::new(NODE_WIDTH, total_h),
    )
}

/// Compute port position in canvas coordinates.
pub fn port_position(node: &Node, port_index: usize, is_output: bool) -> Pos2 {
    let x = if is_output {
        node.position[0] + NODE_WIDTH
    } else {
        node.position[0]
    };
    let y = node.position[1] + NODE_PORT_Y_START + port_index as f32 * NODE_PORT_Y_SPACING;
    Pos2::new(x, y)
}

/// Draw a single node on the canvas.
pub fn draw_node(
    painter: &egui::Painter,
    node: &Node,
    canvas: &CanvasState,
    is_selected: bool,
) {
    let rect = node_rect(node);
    let screen_rect = canvas.canvas_rect_to_screen(rect);

    // Skip if off-screen (culling)
    let clip = painter.clip_rect();
    if !clip.intersects(screen_rect) {
        return;
    }

    let color = node_color(&node.node_type);
    let body_color = Color32::from_rgb(50, 50, 50);
    let rounding = CornerRadius::same(NODE_ROUNDING);

    // Node body background
    painter.rect_filled(screen_rect, rounding, body_color);

    // Node header
    let header_rect = Rect::from_min_size(
        screen_rect.min,
        Vec2::new(screen_rect.width(), NODE_HEADER_HEIGHT * canvas.zoom),
    );
    painter.rect_filled(
        header_rect,
        CornerRadius {
            nw: NODE_ROUNDING,
            ne: NODE_ROUNDING,
            sw: 0,
            se: 0,
        },
        color,
    );

    // Title text
    let title = node.title();
    let font_size = 14.0 * canvas.zoom;
    painter.text(
        header_rect.center(),
        egui::Align2::CENTER_CENTER,
        title,
        FontId::proportional(font_size),
        Color32::WHITE,
    );

    // Dialogue text preview
    if let NodeType::Dialogue(data) = &node.node_type {
        if !data.text.is_empty() {
            let preview: String = data
                .text
                .lines()
                .take(NODE_TEXT_PREVIEW_LINES)
                .collect::<Vec<_>>()
                .join("\n");
            let text_pos = Pos2::new(
                screen_rect.min.x + 10.0 * canvas.zoom,
                header_rect.max.y + 8.0 * canvas.zoom,
            );
            let text_font_size = 11.0 * canvas.zoom;
            painter.text(
                text_pos,
                egui::Align2::LEFT_TOP,
                &preview,
                FontId::proportional(text_font_size),
                Color32::from_rgb(200, 200, 200),
            );
        }
    }

    // Draw ports
    let port_radius = NODE_PORT_RADIUS * canvas.zoom;

    for (i, port) in node.inputs.iter().enumerate() {
        let canvas_pos = port_position(node, i, false);
        let screen_pos = canvas.canvas_to_screen(canvas_pos);
        painter.circle_filled(screen_pos, port_radius, Color32::from_rgb(180, 180, 180));
        painter.circle_stroke(
            screen_pos,
            port_radius,
            Stroke::new(1.5 * canvas.zoom, Color32::WHITE),
        );

        if !port.label.is_empty() {
            let label_pos = Pos2::new(screen_pos.x + port_radius + 4.0 * canvas.zoom, screen_pos.y);
            painter.text(
                label_pos,
                egui::Align2::LEFT_CENTER,
                &port.label,
                FontId::proportional(10.0 * canvas.zoom),
                Color32::from_rgb(180, 180, 180),
            );
        }
    }

    for (i, port) in node.outputs.iter().enumerate() {
        let canvas_pos = port_position(node, i, true);
        let screen_pos = canvas.canvas_to_screen(canvas_pos);
        painter.circle_filled(screen_pos, port_radius, color);
        painter.circle_stroke(
            screen_pos,
            port_radius,
            Stroke::new(1.5 * canvas.zoom, Color32::WHITE),
        );

        if !port.label.is_empty() {
            let label_pos = Pos2::new(screen_pos.x - port_radius - 4.0 * canvas.zoom, screen_pos.y);
            painter.text(
                label_pos,
                egui::Align2::RIGHT_CENTER,
                &port.label,
                FontId::proportional(10.0 * canvas.zoom),
                Color32::from_rgb(180, 180, 180),
            );
        }
    }

    // Selection border
    if is_selected {
        painter.rect_stroke(
            screen_rect,
            rounding,
            Stroke::new(2.0 * canvas.zoom, Color32::from_rgb(255, 255, 100)),
            StrokeKind::Outside,
        );
    } else {
        painter.rect_stroke(
            screen_rect,
            rounding,
            Stroke::new(1.0 * canvas.zoom, Color32::from_rgb(70, 70, 70)),
            StrokeKind::Inside,
        );
    }
}
