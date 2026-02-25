use egui::{Color32, FontId, Pos2, Rect};

use crate::model::node::{Node, NodeType};
use crate::ui::canvas::CanvasState;

const NODE_TEXT_PREVIEW_LINES: usize = 2;
const BODY_TEXT_COLOR: Color32 = Color32::from_rgb(200, 200, 200);
const DIM_TEXT_COLOR: Color32 = Color32::from_rgb(140, 140, 140);

/// Draw type-specific body content inside the node.
pub fn draw_node_body(
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
            draw_condition_body(painter, data, &small_font, body_x, body_y_start, max_text_w);
        }
        NodeType::Event(data) => {
            draw_event_body(painter, data, canvas, &small_font, body_x, body_y_start, max_text_w);
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
        NodeType::SubGraph(data) => {
            let label = format!("{} nodes", data.child_graph.nodes.len());
            let truncated = truncate_to_width(painter, &label, &small_font, max_text_w);
            painter.text(
                Pos2::new(body_x, body_y_start),
                egui::Align2::LEFT_TOP,
                &truncated,
                small_font,
                DIM_TEXT_COLOR,
            );
        }
        _ => {}
    }
}

fn draw_condition_body(
    painter: &egui::Painter,
    data: &crate::model::node::ConditionData,
    font: &FontId,
    body_x: f32,
    body_y: f32,
    max_w: f32,
) {
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
        let truncated = truncate_to_width(painter, &summary, font, max_w);
        painter.text(
            Pos2::new(body_x, body_y),
            egui::Align2::LEFT_TOP,
            &truncated,
            font.clone(),
            BODY_TEXT_COLOR,
        );
    } else {
        painter.text(
            Pos2::new(body_x, body_y),
            egui::Align2::LEFT_TOP,
            "(no condition set)",
            font.clone(),
            DIM_TEXT_COLOR,
        );
    }
}

fn draw_event_body(
    painter: &egui::Painter,
    data: &crate::model::node::EventData,
    canvas: &CanvasState,
    font: &FontId,
    body_x: f32,
    body_y: f32,
    max_w: f32,
) {
    if data.actions.is_empty() {
        painter.text(
            Pos2::new(body_x, body_y),
            egui::Align2::LEFT_TOP,
            "(no actions)",
            font.clone(),
            DIM_TEXT_COLOR,
        );
    } else {
        for (i, action) in data.actions.iter().take(3).enumerate() {
            let label = format!("{}: {}", action.key, format_variable_value(&action.value));
            let truncated = truncate_to_width(painter, &label, font, max_w);
            painter.text(
                Pos2::new(body_x, body_y + i as f32 * 16.0 * canvas.zoom),
                egui::Align2::LEFT_TOP,
                &truncated,
                font.clone(),
                BODY_TEXT_COLOR,
            );
        }
        if data.actions.len() > 3 {
            painter.text(
                Pos2::new(body_x, body_y + 3.0 * 16.0 * canvas.zoom),
                egui::Align2::LEFT_TOP,
                format!("+{} more", data.actions.len() - 3),
                font.clone(),
                DIM_TEXT_COLOR,
            );
        }
    }
}

/// Truncate text with "..." if it exceeds the available width.
pub fn truncate_to_width(
    painter: &egui::Painter,
    text: &str,
    font: &FontId,
    max_width: f32,
) -> String {
    let galley = painter.layout_no_wrap(text.to_string(), font.clone(), Color32::WHITE);
    if galley.rect.width() <= max_width {
        return text.to_string();
    }
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

fn format_variable_value(val: &crate::model::node::VariableValue) -> String {
    match val {
        crate::model::node::VariableValue::Bool(b) => b.to_string(),
        crate::model::node::VariableValue::Int(i) => i.to_string(),
        crate::model::node::VariableValue::Float(f) => format!("{f:.2}"),
        crate::model::node::VariableValue::Text(s) => format!("\"{s}\""),
    }
}
