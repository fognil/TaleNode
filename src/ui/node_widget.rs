use egui::{Color32, CornerRadius, FontId, Pos2, Rect, Stroke, StrokeKind, Vec2};

use crate::model::character::Character;
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
const BODY_TEXT_COLOR: Color32 = Color32::from_rgb(200, 200, 200);
const DIM_TEXT_COLOR: Color32 = Color32::from_rgb(140, 140, 140);

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

    // Extra content height depending on node type
    let content_height = match &node.node_type {
        NodeType::Dialogue(data) if !data.text.is_empty() => {
            let line_count = data.text.lines().take(NODE_TEXT_PREVIEW_LINES).count();
            (line_count.max(1) as f32) * 16.0 + 8.0
        }
        NodeType::Condition(data) => {
            // Show "variable op value" summary
            if !data.variable_name.is_empty() { 18.0 } else { 0.0 }
        }
        NodeType::Event(data) => {
            // Show action count
            if !data.actions.is_empty() {
                (data.actions.len().min(3) as f32) * 16.0 + 4.0
            } else {
                0.0
            }
        }
        _ => 0.0,
    };

    (ports_height + content_height).max(NODE_MIN_BODY_HEIGHT)
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
    is_search_match: bool,
    characters: &[Character],
) {
    let rect = node_rect(node);
    let screen_rect = canvas.canvas_rect_to_screen(rect);

    // Skip if off-screen (culling)
    let clip = painter.clip_rect();
    if !clip.intersects(screen_rect) {
        return;
    }

    // Use character color for Dialogue nodes with a linked speaker
    let color = resolve_node_color(node, characters);
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

    // Body content (type-specific)
    draw_node_body(painter, node, canvas, &screen_rect, &header_rect);

    // Draw ports
    draw_ports(painter, node, canvas, color);

    // Border
    if is_selected {
        painter.rect_stroke(
            screen_rect,
            rounding,
            Stroke::new(2.0 * canvas.zoom, Color32::from_rgb(255, 255, 100)),
            StrokeKind::Outside,
        );
    } else if is_search_match {
        painter.rect_stroke(
            screen_rect,
            rounding,
            Stroke::new(2.0 * canvas.zoom, Color32::from_rgb(100, 200, 255)),
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

/// Draw type-specific body content inside the node.
fn draw_node_body(
    painter: &egui::Painter,
    node: &Node,
    canvas: &CanvasState,
    screen_rect: &Rect,
    header_rect: &Rect,
) {
    let small_font = FontId::proportional(11.0 * canvas.zoom);
    let body_x = screen_rect.min.x + 10.0 * canvas.zoom;
    let body_y_start = header_rect.max.y + 6.0 * canvas.zoom;
    let max_text_w = screen_rect.width() - 20.0 * canvas.zoom;

    match &node.node_type {
        NodeType::Dialogue(data) => {
            if !data.text.is_empty() {
                let preview: String = data
                    .text
                    .lines()
                    .take(NODE_TEXT_PREVIEW_LINES)
                    .collect::<Vec<_>>()
                    .join("\n");
                let truncated = truncate_to_width(painter, &preview, &small_font, max_text_w);
                painter.text(
                    Pos2::new(body_x, body_y_start),
                    egui::Align2::LEFT_TOP,
                    &truncated,
                    small_font,
                    BODY_TEXT_COLOR,
                );
            }
        }
        NodeType::Condition(data) => {
            if !data.variable_name.is_empty() {
                let op_str = match data.operator {
                    crate::model::node::CompareOp::Eq => "==",
                    crate::model::node::CompareOp::Neq => "!=",
                    crate::model::node::CompareOp::Gt => ">",
                    crate::model::node::CompareOp::Lt => "<",
                    crate::model::node::CompareOp::Gte => ">=",
                    crate::model::node::CompareOp::Lte => "<=",
                    crate::model::node::CompareOp::Contains => "contains",
                };
                let val_str = format_variable_value(&data.value);
                let summary = format!("{} {} {}", data.variable_name, op_str, val_str);
                let truncated = truncate_to_width(painter, &summary, &small_font, max_text_w);
                painter.text(
                    Pos2::new(body_x, body_y_start),
                    egui::Align2::LEFT_TOP,
                    &truncated,
                    small_font,
                    BODY_TEXT_COLOR,
                );
            } else {
                painter.text(
                    Pos2::new(body_x, body_y_start),
                    egui::Align2::LEFT_TOP,
                    "(no condition set)",
                    small_font,
                    DIM_TEXT_COLOR,
                );
            }
        }
        NodeType::Event(data) => {
            if data.actions.is_empty() {
                painter.text(
                    Pos2::new(body_x, body_y_start),
                    egui::Align2::LEFT_TOP,
                    "(no actions)",
                    small_font,
                    DIM_TEXT_COLOR,
                );
            } else {
                for (i, action) in data.actions.iter().take(3).enumerate() {
                    let action_label = format!("{}: {}", action.key, format_variable_value(&action.value));
                    let truncated = truncate_to_width(painter, &action_label, &small_font, max_text_w);
                    painter.text(
                        Pos2::new(body_x, body_y_start + i as f32 * 16.0 * canvas.zoom),
                        egui::Align2::LEFT_TOP,
                        &truncated,
                        small_font.clone(),
                        BODY_TEXT_COLOR,
                    );
                }
                if data.actions.len() > 3 {
                    painter.text(
                        Pos2::new(body_x, body_y_start + 3.0 * 16.0 * canvas.zoom),
                        egui::Align2::LEFT_TOP,
                        format!("+{} more", data.actions.len() - 3),
                        small_font,
                        DIM_TEXT_COLOR,
                    );
                }
            }
        }
        NodeType::End(data) => {
            if !data.tag.is_empty() {
                let label = format!("tag: {}", data.tag);
                let truncated = truncate_to_width(painter, &label, &small_font, max_text_w);
                painter.text(
                    Pos2::new(body_x, body_y_start),
                    egui::Align2::LEFT_TOP,
                    &truncated,
                    small_font,
                    DIM_TEXT_COLOR,
                );
            }
        }
        _ => {}
    }
}

/// Truncate text with "..." if it exceeds the available width.
fn truncate_to_width(painter: &egui::Painter, text: &str, font: &FontId, max_width: f32) -> String {
    let galley = painter.layout_no_wrap(text.to_string(), font.clone(), Color32::WHITE);
    if galley.rect.width() <= max_width {
        return text.to_string();
    }
    // Remove chars from end until it fits with ellipsis
    let chars: Vec<char> = text.chars().collect();
    let mut end = chars.len();
    while end > 0 {
        end -= 1;
        let candidate: String = chars[..end].iter().collect::<String>() + "...";
        let galley = painter.layout_no_wrap(candidate.clone(), font.clone(), Color32::WHITE);
        if galley.rect.width() <= max_width {
            return candidate;
        }
    }
    "...".to_string()
}

/// Draw input and output ports on a node.
fn draw_ports(
    painter: &egui::Painter,
    node: &Node,
    canvas: &CanvasState,
    accent_color: Color32,
) {
    let port_radius = NODE_PORT_RADIUS * canvas.zoom;
    let label_font = FontId::proportional(10.0 * canvas.zoom);

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
            let label_pos = Pos2::new(
                screen_pos.x + port_radius + 4.0 * canvas.zoom,
                screen_pos.y,
            );
            painter.text(
                label_pos,
                egui::Align2::LEFT_CENTER,
                &port.label,
                label_font.clone(),
                Color32::from_rgb(180, 180, 180),
            );
        }
    }

    for (i, port) in node.outputs.iter().enumerate() {
        let canvas_pos = port_position(node, i, true);
        let screen_pos = canvas.canvas_to_screen(canvas_pos);
        painter.circle_filled(screen_pos, port_radius, accent_color);
        painter.circle_stroke(
            screen_pos,
            port_radius,
            Stroke::new(1.5 * canvas.zoom, Color32::WHITE),
        );

        if !port.label.is_empty() {
            let label_pos = Pos2::new(
                screen_pos.x - port_radius - 4.0 * canvas.zoom,
                screen_pos.y,
            );
            painter.text(
                label_pos,
                egui::Align2::RIGHT_CENTER,
                &port.label,
                label_font.clone(),
                Color32::from_rgb(180, 180, 180),
            );
        }
    }
}

/// Resolve the header color for a node.
/// Dialogue nodes with a linked character use that character's color.
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

fn format_variable_value(val: &crate::model::node::VariableValue) -> String {
    match val {
        crate::model::node::VariableValue::Bool(b) => b.to_string(),
        crate::model::node::VariableValue::Int(i) => i.to_string(),
        crate::model::node::VariableValue::Float(f) => format!("{f:.2}"),
        crate::model::node::VariableValue::Text(s) => format!("\"{s}\""),
    }
}
