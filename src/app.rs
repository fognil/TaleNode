use egui::{Color32, Pos2, Rect, Sense, Stroke, StrokeKind};
use uuid::Uuid;

use crate::actions::history::UndoHistory;
use crate::model::graph::DialogueGraph;
use crate::model::node::Node;
use crate::model::port::{PortDirection, PortId};
use crate::ui::canvas::CanvasState;
use crate::ui::connection_renderer::{draw_bezier_wire, draw_connections};
use crate::ui::node_widget::{self, draw_node, PORT_RADIUS};

/// Which port the user started dragging from.
#[derive(Debug, Clone)]
struct DragWire {
    from_node: Uuid,
    from_port: PortId,
    from_direction: PortDirection,
    /// Screen position of the free end (follows cursor).
    cursor_pos: Pos2,
}

/// What the user is currently doing on the canvas.
#[derive(Debug, Clone, Default)]
enum InteractionState {
    #[default]
    Idle,
    /// Dragging one or more selected nodes.
    DraggingNodes {
        start_positions: Vec<(Uuid, [f32; 2])>,
    },
    /// Dragging a wire from a port.
    DraggingWire(DragWire),
    /// Box-selecting nodes.
    BoxSelecting {
        start: Pos2,
    },
}

/// Top-level application state.
pub struct TaleNodeApp {
    pub graph: DialogueGraph,
    pub canvas: CanvasState,
    pub selected_nodes: Vec<Uuid>,
    interaction: InteractionState,
    /// Where to open the context menu (canvas coords).
    context_menu_pos: Option<[f32; 2]>,
    /// Current project name.
    project_name: String,
    /// Path to the current .talenode file (None if unsaved).
    project_path: Option<std::path::PathBuf>,
    /// Whether the left panel is visible.
    show_left_panel: bool,
    /// Undo/redo history.
    history: UndoHistory,
}

impl TaleNodeApp {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        let mut graph = DialogueGraph::new();

        // Create a default Start node
        let start = Node::new_start([100.0, 200.0]);
        graph.add_node(start);

        Self {
            graph,
            canvas: CanvasState::default(),
            selected_nodes: Vec::new(),
            interaction: InteractionState::Idle,
            context_menu_pos: None,
            project_name: "Untitled".to_string(),
            project_path: None,
            show_left_panel: true,
            history: UndoHistory::new(),
        }
    }

    /// Save a snapshot for undo before mutating the graph.
    fn snapshot(&mut self) {
        self.history.save_snapshot(&self.graph);
    }

    /// Hit-test: find node under screen position (topmost first).
    fn node_at_screen_pos(&self, screen_pos: Pos2) -> Option<Uuid> {
        // Iterate in reverse insertion order so "top" nodes are checked first
        for node in self.graph.nodes.values() {
            let rect = node_widget::node_rect(node);
            let screen_rect = self.canvas.canvas_rect_to_screen(rect);
            if screen_rect.contains(screen_pos) {
                return Some(node.id);
            }
        }
        None
    }

    /// Hit-test: find port under screen position. Returns (node_id, port_id, direction).
    fn port_at_screen_pos(
        &self,
        screen_pos: Pos2,
    ) -> Option<(Uuid, PortId, PortDirection)> {
        let hit_radius = (PORT_RADIUS + 4.0) * self.canvas.zoom;

        for node in self.graph.nodes.values() {
            // Check output ports
            for (i, port) in node.outputs.iter().enumerate() {
                let port_canvas = node_widget::port_position(node, i, true);
                let port_screen = self.canvas.canvas_to_screen(port_canvas);
                if screen_pos.distance(port_screen) <= hit_radius {
                    return Some((node.id, port.id, PortDirection::Output));
                }
            }
            // Check input ports
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

    fn show_canvas(&mut self, ui: &mut egui::Ui) {
        let canvas_rect = ui.available_rect_before_wrap();
        let response = ui.allocate_rect(canvas_rect, Sense::click_and_drag());
        let painter = ui.painter_at(canvas_rect);

        // Handle pan/zoom
        self.canvas.handle_input(&response, ui);

        // Draw grid
        self.canvas.draw_grid(&painter, canvas_rect);

        // Draw connections (below nodes)
        draw_connections(&painter, &self.graph, &self.canvas, None);

        // Draw nodes
        let node_ids: Vec<Uuid> = self.graph.nodes.keys().copied().collect();
        for id in &node_ids {
            if let Some(node) = self.graph.nodes.get(id) {
                let is_selected = self.selected_nodes.contains(id);
                draw_node(&painter, node, &self.canvas, is_selected);
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
                painter.rect_filled(
                    sel_rect,
                    0.0,
                    Color32::from_rgba_premultiplied(100, 150, 255, 30),
                );
                painter.rect_stroke(
                    sel_rect,
                    0.0,
                    Stroke::new(1.0, Color32::from_rgb(100, 150, 255)),
                    StrokeKind::Outside,
                );
            }
        }

        // === INTERACTION HANDLING ===
        self.handle_interactions(&response, canvas_rect);

        // === CONTEXT MENU ===
        self.handle_context_menu(&response);
    }

    fn handle_interactions(&mut self, response: &egui::Response, _canvas_rect: Rect) {
        let pointer_pos = response.hover_pos().unwrap_or(Pos2::ZERO);

        // Right click → context menu
        if response.secondary_clicked() {
            let canvas_pos = self.canvas.screen_to_canvas(pointer_pos);
            self.context_menu_pos = Some([canvas_pos.x, canvas_pos.y]);
        }

        // Left click down → start interaction
        if response.drag_started_by(egui::PointerButton::Primary) {
            // Check port first
            if let Some((node_id, port_id, direction)) = self.port_at_screen_pos(pointer_pos) {
                self.interaction = InteractionState::DraggingWire(DragWire {
                    from_node: node_id,
                    from_port: port_id,
                    from_direction: direction,
                    cursor_pos: pointer_pos,
                });
            }
            // Check node
            else if let Some(node_id) = self.node_at_screen_pos(pointer_pos) {
                if !self.selected_nodes.contains(&node_id) {
                    self.selected_nodes.clear();
                    self.selected_nodes.push(node_id);
                }
                // Snapshot before dragging for undo
                self.snapshot();
                let start_positions: Vec<(Uuid, [f32; 2])> = self
                    .selected_nodes
                    .iter()
                    .filter_map(|id| {
                        self.graph.nodes.get(id).map(|n| (*id, n.position))
                    })
                    .collect();
                self.interaction = InteractionState::DraggingNodes { start_positions };
            }
            // Empty space → box select
            else {
                self.selected_nodes.clear();
                self.interaction = InteractionState::BoxSelecting {
                    start: pointer_pos,
                };
            }
        }

        // During drag
        if response.dragged_by(egui::PointerButton::Primary) {
            match &mut self.interaction {
                InteractionState::DraggingNodes { .. } => {
                    let delta = response.drag_delta() / self.canvas.zoom;
                    let ids: Vec<Uuid> = self.selected_nodes.clone();
                    for id in ids {
                        if let Some(node) = self.graph.nodes.get_mut(&id) {
                            node.position[0] += delta.x;
                            node.position[1] += delta.y;
                        }
                    }
                }
                InteractionState::DraggingWire(drag) => {
                    drag.cursor_pos = pointer_pos;
                }
                InteractionState::BoxSelecting { .. } => {
                    // Visual update handled in draw
                }
                InteractionState::Idle => {}
            }
        }

        // Release
        if response.drag_stopped_by(egui::PointerButton::Primary) {
            let interaction = self.interaction.clone();
            match &interaction {
                InteractionState::DraggingWire(drag) => {
                    // Try to connect to a port
                    if let Some((target_node, target_port, target_dir)) =
                        self.port_at_screen_pos(pointer_pos)
                    {
                        // Must connect output→input
                        match (drag.from_direction, target_dir) {
                            (PortDirection::Output, PortDirection::Input) => {
                                self.snapshot();
                                self.graph.add_connection(
                                    drag.from_node,
                                    drag.from_port,
                                    target_node,
                                    target_port,
                                );
                            }
                            (PortDirection::Input, PortDirection::Output) => {
                                self.snapshot();
                                self.graph.add_connection(
                                    target_node,
                                    target_port,
                                    drag.from_node,
                                    drag.from_port,
                                );
                            }
                            _ => {} // Same direction, ignore
                        }
                    }
                }
                InteractionState::BoxSelecting { start } => {
                    let sel_rect = Rect::from_two_pos(*start, pointer_pos);
                    self.selected_nodes.clear();
                    for node in self.graph.nodes.values() {
                        let node_rect = node_widget::node_rect(node);
                        let screen_rect = self.canvas.canvas_rect_to_screen(node_rect);
                        if sel_rect.intersects(screen_rect) {
                            self.selected_nodes.push(node.id);
                        }
                    }
                }
                _ => {}
            }
            self.interaction = InteractionState::Idle;
        }

        // Click on empty space to deselect
        if response.clicked()
            && matches!(self.interaction, InteractionState::Idle)
            && self.node_at_screen_pos(pointer_pos).is_none()
        {
            self.selected_nodes.clear();
        }

        // Delete selected nodes
        if !self.selected_nodes.is_empty() {
            let delete_pressed = response.ctx.input(|i| {
                i.key_pressed(egui::Key::Delete) || i.key_pressed(egui::Key::Backspace)
            });
            if delete_pressed {
                self.snapshot();
                let ids: Vec<Uuid> = self.selected_nodes.drain(..).collect();
                for id in ids {
                    self.graph.remove_node(id);
                }
            }
        }
    }

    fn handle_context_menu(&mut self, response: &egui::Response) {
        if self.context_menu_pos.is_none() {
            return;
        }

        let ctx_pos = self.context_menu_pos.unwrap();
        let mut close_menu = false;

        // Use egui's built-in context menu at the response level
        let menu_id = response.id.with("ctx_menu");
        egui::Area::new(menu_id)
            .fixed_pos(self.canvas.canvas_to_screen(Pos2::new(ctx_pos[0], ctx_pos[1])))
            .order(egui::Order::Foreground)
            .show(&response.ctx, |ui| {
                egui::Frame::popup(ui.style()).show(ui, |ui| {
                    ui.set_min_width(150.0);
                    ui.label("Add Node");
                    ui.separator();

                    type NodeCtor = fn([f32; 2]) -> Node;
                    let items: &[(&str, NodeCtor)] = &[
                        ("Start", Node::new_start),
                        ("Dialogue", Node::new_dialogue),
                        ("Choice", Node::new_choice),
                        ("Condition", Node::new_condition),
                        ("Event", Node::new_event),
                        ("Random", Node::new_random),
                        ("End", Node::new_end),
                    ];
                    for (label, constructor) in items {
                        if ui.button(*label).clicked() {
                            self.snapshot();
                            self.graph.add_node(constructor(ctx_pos));
                            close_menu = true;
                        }
                    }
                });
            });

        // Close menu on click outside or after selection
        if close_menu || response.clicked() || response.clicked_by(egui::PointerButton::Primary) {
            self.context_menu_pos = None;
        }
    }

    fn show_menu_bar(&mut self, ui: &mut egui::Ui) {
        egui::menu::bar(ui, |ui| {
            ui.menu_button("File", |ui| {
                if ui.button("New").clicked() {
                    self.graph = DialogueGraph::new();
                    self.graph.add_node(Node::new_start([100.0, 200.0]));
                    self.selected_nodes.clear();
                    self.project_name = "Untitled".to_string();
                    self.project_path = None;
                    ui.close_menu();
                }
                if ui.button("Open...").clicked() {
                    self.do_open();
                    ui.close_menu();
                }
                if ui.button("Save").clicked() {
                    self.do_save(false);
                    ui.close_menu();
                }
                if ui.button("Save As...").clicked() {
                    self.do_save(true);
                    ui.close_menu();
                }
                ui.separator();
                if ui.button("Export JSON...").clicked() {
                    self.do_export_json();
                    ui.close_menu();
                }
            });
            ui.menu_button("View", |ui| {
                if ui.checkbox(&mut self.show_left_panel, "Left Panel").changed() {
                    ui.close_menu();
                }
            });
        });
    }

    fn do_open(&mut self) {
        let file = rfd::FileDialog::new()
            .add_filter("TaleNode Project", &["talenode"])
            .pick_file();
        if let Some(path) = file {
            match std::fs::read_to_string(&path) {
                Ok(contents) => {
                    match crate::model::project::Project::load_from_string(&contents) {
                        Ok(project) => {
                            self.graph = project.graph;
                            self.project_name = project.name;
                            self.project_path = Some(path);
                            self.selected_nodes.clear();
                        }
                        Err(e) => {
                            eprintln!("Failed to parse project: {e}");
                        }
                    }
                }
                Err(e) => {
                    eprintln!("Failed to read file: {e}");
                }
            }
        }
    }

    fn do_save(&mut self, save_as: bool) {
        let path = if save_as || self.project_path.is_none() {
            rfd::FileDialog::new()
                .add_filter("TaleNode Project", &["talenode"])
                .set_file_name(format!("{}.talenode", self.project_name))
                .save_file()
        } else {
            self.project_path.clone()
        };

        if let Some(path) = path {
            let project = crate::model::project::Project {
                version: "1.0".to_string(),
                name: self.project_name.clone(),
                graph: self.graph.clone(),
            };
            match project.save_to_string() {
                Ok(json) => {
                    if let Err(e) = std::fs::write(&path, json) {
                        eprintln!("Failed to write file: {e}");
                    } else {
                        self.project_path = Some(path);
                    }
                }
                Err(e) => {
                    eprintln!("Failed to serialize project: {e}");
                }
            }
        }
    }

    fn do_export_json(&self) {
        let path = rfd::FileDialog::new()
            .add_filter("JSON", &["json"])
            .set_file_name(format!("{}.json", self.project_name))
            .save_file();

        if let Some(path) = path {
            match crate::export::json_export::export_json(&self.graph, &self.project_name) {
                Ok(json) => {
                    if let Err(e) = std::fs::write(&path, json) {
                        eprintln!("Failed to write export: {e}");
                    }
                }
                Err(e) => {
                    eprintln!("Failed to export JSON: {e}");
                }
            }
        }
    }

    fn show_status_bar(&self, ui: &mut egui::Ui) {
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
        });
    }
}

impl eframe::App for TaleNodeApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Keyboard shortcuts
        if ctx.input(|i| i.modifiers.command && !i.modifiers.shift && i.key_pressed(egui::Key::S)) {
            self.do_save(false);
        }
        if ctx.input(|i| i.modifiers.command && i.key_pressed(egui::Key::O)) {
            self.do_open();
        }
        if ctx.input(|i| i.modifiers.command && !i.modifiers.shift && i.key_pressed(egui::Key::Z)) {
            if let Some(prev) = self.history.undo(&self.graph) {
                self.graph = prev;
                self.selected_nodes.clear();
            }
        }
        if ctx.input(|i| i.modifiers.command && i.modifiers.shift && i.key_pressed(egui::Key::Z)) {
            if let Some(next) = self.history.redo(&self.graph) {
                self.graph = next;
                self.selected_nodes.clear();
            }
        }

        // Menu bar at top
        egui::TopBottomPanel::top("menu_bar").show(ctx, |ui| {
            self.show_menu_bar(ui);
        });

        // Status bar at bottom
        egui::TopBottomPanel::bottom("status_bar").show(ctx, |ui| {
            self.show_status_bar(ui);
        });

        // Left panel (variables, characters)
        if self.show_left_panel {
            egui::SidePanel::left("left_panel")
                .default_width(200.0)
                .min_width(150.0)
                .show(ctx, |ui| {
                    egui::ScrollArea::vertical().show(ui, |ui| {
                        crate::ui::left_panel::show_left_panel(ui, &mut self.graph);
                    });
                });
        }

        // Inspector panel (right side) — only when exactly 1 node selected
        if self.selected_nodes.len() == 1 {
            let selected_id = self.selected_nodes[0];
            egui::SidePanel::right("inspector")
                .default_width(280.0)
                .min_width(220.0)
                .show(ctx, |ui| {
                    egui::ScrollArea::vertical().show(ui, |ui| {
                        crate::ui::inspector::show_inspector(
                            ui,
                            &mut self.graph,
                            selected_id,
                        );
                    });
                });
        }

        // Main canvas
        egui::CentralPanel::default()
            .frame(egui::Frame::NONE)
            .show(ctx, |ui| {
                self.show_canvas(ui);
            });
    }
}
