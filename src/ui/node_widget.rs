use egui::{Color32, CornerRadius, FontId, Pos2, Rect, Stroke, StrokeKind, Vec2};

use crate::model::character::Character;
use crate::model::node::{Node, NodeType};
use crate::model::port::PortId;
use crate::model::review::ReviewStatus;
use crate::ui::canvas::CanvasState;
use super::node_body::draw_node_body;

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
pub fn node_color(node_type: &NodeType) -> Color32 {
    match node_type {
        NodeType::Start => Color32::from_rgb(76, 175, 80),       // green
        NodeType::Dialogue(_) => Color32::from_rgb(66, 133, 244), // blue
        NodeType::Choice(_) => Color32::from_rgb(251, 188, 4),    // yellow
        NodeType::Condition(_) => Color32::from_rgb(255, 152, 0), // orange
        NodeType::Event(_) => Color32::from_rgb(171, 71, 188),    // purple
        NodeType::Random(_) => Color32::from_rgb(158, 158, 158),  // gray
        NodeType::End(_) => Color32::from_rgb(244, 67, 54),       // red
        NodeType::SubGraph(_) => Color32::from_rgb(0, 188, 212),  // cyan
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

    let content_height = match &node.node_type {
        NodeType::Dialogue(data) if !data.text.is_empty() => {
            let line_count = data.text.lines().take(NODE_TEXT_PREVIEW_LINES).count();
            (line_count.max(1) as f32) * 16.0 + 8.0
        }
        NodeType::Condition(data) => {
            if !data.variable_name.is_empty() { 18.0 } else { 0.0 }
        }
        NodeType::Event(data) => {
            if !data.actions.is_empty() {
                (data.actions.len().min(3) as f32) * 16.0 + 4.0
            } else {
                0.0
            }
        }
        NodeType::SubGraph(_) => 18.0,
        _ => 0.0,
    };

    (ports_height + content_height).max(NODE_MIN_BODY_HEIGHT)
}

/// Get the bounding rect of a node in canvas coordinates.
pub fn node_rect(node: &Node) -> Rect {
    let total_h = if node.collapsed {
        NODE_HEADER_HEIGHT
    } else {
        NODE_HEADER_HEIGHT + node_body_height(node)
    };
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
    let y = if node.collapsed {
        node.position[1] + NODE_HEADER_HEIGHT * 0.5
    } else {
        node.position[1] + NODE_PORT_Y_START + port_index as f32 * NODE_PORT_Y_SPACING
    };
    Pos2::new(x, y)
}

/// Clickable collapse toggle rect in canvas coordinates (triangle area in header).
pub fn collapse_toggle_rect(node: &Node) -> Rect {
    let x = node.position[0] + 2.0;
    let y = node.position[1] + 2.0;
    Rect::from_min_size(Pos2::new(x, y), Vec2::new(22.0, NODE_HEADER_HEIGHT - 4.0))
}

/// Draw a single node on the canvas.
#[allow(clippy::too_many_arguments)]
pub fn draw_node(
    painter: &egui::Painter,
    node: &Node,
    canvas: &CanvasState,
    is_selected: bool,
    is_search_match: bool,
    characters: &[Character],
    review_status: ReviewStatus,
    hovered_port: Option<PortId>,
    playtest_active: bool,
) {
    let rect = node_rect(node);
    let screen_rect = canvas.canvas_rect_to_screen(rect);

    // Skip if off-screen (culling)
    let clip = painter.clip_rect();
    if !clip.intersects(screen_rect) {
        return;
    }

    let color = resolve_node_color(node, characters);
    let body_color = Color32::from_rgb(50, 50, 50);
    let rounding = CornerRadius::same(NODE_ROUNDING);

    // Node header
    let header_rect = Rect::from_min_size(
        screen_rect.min,
        Vec2::new(screen_rect.width(), NODE_HEADER_HEIGHT * canvas.zoom),
    );

    if node.collapsed {
        // Collapsed: header only with full rounding
        painter.rect_filled(screen_rect, rounding, color);
    } else {
        // Expanded: body background + header on top
        painter.rect_filled(screen_rect, rounding, body_color);
        painter.rect_filled(
            header_rect,
            CornerRadius { nw: NODE_ROUNDING, ne: NODE_ROUNDING, sw: 0, se: 0 },
            color,
        );
    }

    // Collapse toggle triangle
    draw_collapse_triangle(painter, node, canvas, &header_rect);

    // Title text (shifted right to make room for triangle)
    let title = node.title();
    let font_size = 14.0 * canvas.zoom;
    let title_center = Pos2::new(
        header_rect.center().x + 8.0 * canvas.zoom,
        header_rect.center().y,
    );
    painter.text(
        title_center,
        egui::Align2::CENTER_CENTER,
        title,
        FontId::proportional(font_size),
        Color32::WHITE,
    );

    if !node.collapsed {
        // Body content (type-specific — delegated to node_body module)
        draw_node_body(painter, node, canvas, &screen_rect, &header_rect);

        // Draw ports
        draw_ports(painter, node, canvas, color, hovered_port);
    }

    // Border
    draw_border(
        painter, &screen_rect, rounding, canvas.zoom,
        is_selected, is_search_match, playtest_active,
    );

    // Review status badge
    if review_status != ReviewStatus::Draft {
        let badge_color = crate::ui::comments_panel::status_color(review_status);
        let badge_radius = 5.0 * canvas.zoom;
        let badge_pos = Pos2::new(
            header_rect.max.x - badge_radius - 4.0 * canvas.zoom,
            header_rect.min.y + badge_radius + 4.0 * canvas.zoom,
        );
        painter.circle_filled(badge_pos, badge_radius, badge_color);
    }
}

/// Draw the collapse/expand triangle indicator in the header.
fn draw_collapse_triangle(
    painter: &egui::Painter,
    node: &Node,
    canvas: &CanvasState,
    header_rect: &Rect,
) {
    let size = 6.0 * canvas.zoom;
    let cx = header_rect.min.x + 12.0 * canvas.zoom;
    let cy = header_rect.center().y;
    let points = if node.collapsed {
        // Right-pointing triangle ▸
        vec![
            Pos2::new(cx - size * 0.4, cy - size),
            Pos2::new(cx + size * 0.6, cy),
            Pos2::new(cx - size * 0.4, cy + size),
        ]
    } else {
        // Down-pointing triangle ▾
        vec![
            Pos2::new(cx - size, cy - size * 0.4),
            Pos2::new(cx + size, cy - size * 0.4),
            Pos2::new(cx, cy + size * 0.6),
        ]
    };
    let color = Color32::from_rgba_premultiplied(255, 255, 255, 200);
    painter.add(egui::Shape::convex_polygon(points, color, Stroke::NONE));
}

#[allow(clippy::too_many_arguments)]
fn draw_border(
    painter: &egui::Painter,
    rect: &Rect,
    rounding: CornerRadius,
    zoom: f32,
    is_selected: bool,
    is_search_match: bool,
    playtest_active: bool,
) {
    // Playtest glow (drawn underneath other borders)
    if playtest_active {
        let t = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as f32;
        let alpha = ((t / 500.0).sin() * 0.3 + 0.7) * 255.0;
        let green = Color32::from_rgba_unmultiplied(76, 175, 80, alpha as u8);
        painter.rect_stroke(
            rect.expand(3.0 * zoom), rounding,
            Stroke::new(3.0 * zoom, green),
            StrokeKind::Outside,
        );
    }
    if is_selected {
        painter.rect_stroke(
            *rect, rounding,
            Stroke::new(2.0 * zoom, Color32::from_rgb(255, 255, 100)),
            StrokeKind::Outside,
        );
    } else if is_search_match {
        // Outer glow ring
        painter.rect_stroke(
            rect.expand(2.0 * zoom), rounding,
            Stroke::new(2.0 * zoom, Color32::from_rgba_unmultiplied(0, 200, 255, 80)),
            StrokeKind::Outside,
        );
        // Inner bright border
        painter.rect_stroke(
            *rect, rounding,
            Stroke::new(2.5 * zoom, Color32::from_rgb(0, 200, 255)),
            StrokeKind::Outside,
        );
    } else {
        painter.rect_stroke(
            *rect, rounding,
            Stroke::new(1.0 * zoom, Color32::from_rgb(70, 70, 70)),
            StrokeKind::Inside,
        );
    }
}

/// Draw input and output ports on a node.
fn draw_ports(
    painter: &egui::Painter,
    node: &Node,
    canvas: &CanvasState,
    accent_color: Color32,
    hovered_port: Option<PortId>,
) {
    let port_radius = NODE_PORT_RADIUS * canvas.zoom;
    let hover_radius = (NODE_PORT_RADIUS + 3.0) * canvas.zoom;
    let label_font = FontId::proportional(10.0 * canvas.zoom);

    for (i, port) in node.inputs.iter().enumerate() {
        let canvas_pos = port_position(node, i, false);
        let screen_pos = canvas.canvas_to_screen(canvas_pos);
        let is_hovered = hovered_port == Some(port.id);
        if is_hovered {
            painter.circle_filled(
                screen_pos, hover_radius,
                Color32::from_rgba_premultiplied(180, 180, 180, 60),
            );
        }
        let r = if is_hovered { hover_radius } else { port_radius };
        painter.circle_filled(screen_pos, r, Color32::from_rgb(180, 180, 180));
        painter.circle_stroke(
            screen_pos, r,
            Stroke::new(1.5 * canvas.zoom, Color32::WHITE),
        );
        if !port.label.is_empty() {
            painter.text(
                Pos2::new(screen_pos.x + port_radius + 4.0 * canvas.zoom, screen_pos.y),
                egui::Align2::LEFT_CENTER, &port.label,
                label_font.clone(), Color32::from_rgb(180, 180, 180),
            );
        }
    }

    for (i, port) in node.outputs.iter().enumerate() {
        let canvas_pos = port_position(node, i, true);
        let screen_pos = canvas.canvas_to_screen(canvas_pos);
        let is_hovered = hovered_port == Some(port.id);
        if is_hovered {
            painter.circle_filled(
                screen_pos, hover_radius,
                Color32::from_rgba_premultiplied(255, 255, 100, 60),
            );
        }
        let r = if is_hovered { hover_radius } else { port_radius };
        painter.circle_filled(screen_pos, r, accent_color);
        painter.circle_stroke(
            screen_pos, r,
            Stroke::new(1.5 * canvas.zoom, Color32::WHITE),
        );
        if !port.label.is_empty() {
            painter.text(
                Pos2::new(screen_pos.x - port_radius - 4.0 * canvas.zoom, screen_pos.y),
                egui::Align2::RIGHT_CENTER, &port.label,
                label_font.clone(), Color32::from_rgb(180, 180, 180),
            );
        }
    }
}

/// Resolve the header color for a node.
fn resolve_node_color(node: &Node, characters: &[Character]) -> Color32 {
    if let NodeType::Dialogue(data) = &node.node_type {
        if let Some(speaker_id) = data.speaker_id {
            if let Some(ch) = characters.iter().find(|c| c.id == speaker_id) {
                return Color32::from_rgb(ch.color[0], ch.color[1], ch.color[2]);
            }
        }
    }
    node_color(&node.node_type)
}
