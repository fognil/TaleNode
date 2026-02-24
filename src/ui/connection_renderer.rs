use egui::{Color32, Pos2, Stroke};

use crate::model::connection::Connection;
use crate::model::graph::DialogueGraph;
use crate::ui::canvas::CanvasState;
use crate::ui::node_widget::port_position;

const WIRE_THICKNESS: f32 = 2.5;
const WIRE_COLOR: Color32 = Color32::from_rgb(180, 180, 180);
const WIRE_SELECTED_COLOR: Color32 = Color32::from_rgb(255, 255, 100);

/// Draw all connections in the graph.
pub fn draw_connections(
    painter: &egui::Painter,
    graph: &DialogueGraph,
    canvas: &CanvasState,
    selected_connection: Option<uuid::Uuid>,
) {
    for conn in &graph.connections {
        draw_connection(painter, conn, graph, canvas, selected_connection);
    }
}

fn draw_connection(
    painter: &egui::Painter,
    conn: &Connection,
    graph: &DialogueGraph,
    canvas: &CanvasState,
    selected_connection: Option<uuid::Uuid>,
) {
    let from_node = match graph.nodes.get(&conn.from_node) {
        Some(n) => n,
        None => return,
    };
    let to_node = match graph.nodes.get(&conn.to_node) {
        Some(n) => n,
        None => return,
    };

    // Find port indices
    let from_index = from_node
        .outputs
        .iter()
        .position(|p| p.id == conn.from_port)
        .unwrap_or(0);
    let to_index = to_node
        .inputs
        .iter()
        .position(|p| p.id == conn.to_port)
        .unwrap_or(0);

    let from_canvas = port_position(from_node, from_index, true);
    let to_canvas = port_position(to_node, to_index, false);

    let from_screen = canvas.canvas_to_screen(from_canvas);
    let to_screen = canvas.canvas_to_screen(to_canvas);

    let is_selected = selected_connection.is_some_and(|id| id == conn.id);
    let color = if is_selected {
        WIRE_SELECTED_COLOR
    } else {
        WIRE_COLOR
    };

    draw_bezier_wire(painter, from_screen, to_screen, color, canvas.zoom);
}

/// Draw a cubic bezier wire between two screen positions.
pub fn draw_bezier_wire(
    painter: &egui::Painter,
    from: Pos2,
    to: Pos2,
    color: Color32,
    zoom: f32,
) {
    let dx = (to.x - from.x).abs().max(50.0 * zoom) * 0.5;

    let cp1 = Pos2::new(from.x + dx, from.y);
    let cp2 = Pos2::new(to.x - dx, to.y);

    let thickness = WIRE_THICKNESS * zoom;
    let stroke = Stroke::new(thickness, color);

    // Approximate bezier with line segments
    let segments = 32;
    let mut points = Vec::with_capacity(segments + 1);
    for i in 0..=segments {
        let t = i as f32 / segments as f32;
        let p = cubic_bezier(from, cp1, cp2, to, t);
        points.push(p);
    }

    for pair in points.windows(2) {
        painter.line_segment([pair[0], pair[1]], stroke);
    }
}

/// Evaluate a cubic bezier at parameter t.
fn cubic_bezier(p0: Pos2, p1: Pos2, p2: Pos2, p3: Pos2, t: f32) -> Pos2 {
    let u = 1.0 - t;
    let tt = t * t;
    let uu = u * u;
    let uuu = uu * u;
    let ttt = tt * t;

    Pos2::new(
        uuu * p0.x + 3.0 * uu * t * p1.x + 3.0 * u * tt * p2.x + ttt * p3.x,
        uuu * p0.y + 3.0 * uu * t * p1.y + 3.0 * u * tt * p2.y + ttt * p3.y,
    )
}
