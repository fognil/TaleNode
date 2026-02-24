use egui::{Color32, Pos2, Rect, Sense, Stroke, StrokeKind};
use std::time::Instant;
use uuid::Uuid;

use crate::actions::history::UndoHistory;
use crate::model::graph::DialogueGraph;
use crate::model::group::NodeGroup;
use crate::model::node::Node;
use crate::model::port::{PortDirection, PortId};
use crate::ui::canvas::CanvasState;
use crate::ui::connection_renderer::{draw_bezier_wire, draw_connections};
use crate::ui::node_widget::{self, draw_node, PORT_RADIUS};
use crate::ui::playtest::PlaytestState;
use crate::validation::validator::{self, Severity, ValidationWarning};

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
    DraggingNodes,
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
    /// Cached validation warnings.
    validation_warnings: Vec<ValidationWarning>,
    /// Whether the validation panel is open.
    show_validation_panel: bool,
    /// Search query for finding nodes.
    search_query: String,
    /// Whether the search bar is visible.
    show_search: bool,
    /// Node IDs matching the current search.
    search_results: Vec<Uuid>,
    /// Current index in search results for cycling through matches.
    search_index: usize,
    /// Replace query text.
    replace_query: String,
    /// Whether the replace row is visible.
    show_replace: bool,
    /// Whether to use dark theme (true) or light theme (false).
    dark_theme: bool,
    /// Playtest mode state.
    playtest: PlaytestState,
    /// Whether the playtest panel is visible.
    show_playtest: bool,
    /// Last auto-save time.
    last_auto_save: Instant,
    /// Brief message shown in status bar.
    status_message: Option<(String, Instant)>,
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
            validation_warnings: Vec::new(),
            show_validation_panel: false,
            search_query: String::new(),
            show_search: false,
            search_results: Vec::new(),
            search_index: 0,
            replace_query: String::new(),
            show_replace: false,
            dark_theme: true,
            playtest: PlaytestState::new(),
            show_playtest: false,
            last_auto_save: Instant::now(),
            status_message: None,
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

        // Draw groups (below connections and nodes)
        self.draw_groups(&painter);

        // Draw connections (below nodes)
        draw_connections(&painter, &self.graph, &self.canvas, None);

        // Draw nodes
        let node_ids: Vec<Uuid> = self.graph.nodes.keys().copied().collect();
        for id in &node_ids {
            if let Some(node) = self.graph.nodes.get(id) {
                let is_selected = self.selected_nodes.contains(id);
                let is_search_match = self.search_results.contains(id);
                draw_node(&painter, node, &self.canvas, is_selected, is_search_match, &self.graph.characters);
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

        // Draw minimap overlay
        self.draw_minimap(ui, &painter, canvas_rect);

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
                self.interaction = InteractionState::DraggingNodes;
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
                InteractionState::DraggingNodes => {
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

                    // Group actions
                    if !self.selected_nodes.is_empty() {
                        ui.separator();
                        if ui.button("Group Selected").clicked() {
                            self.snapshot();
                            let mut group = NodeGroup::new("Group");
                            group.node_ids = self.selected_nodes.clone();
                            self.graph.groups.push(group);
                            close_menu = true;
                        }
                        // Ungroup: remove any group containing all selected nodes
                        let has_group = self.graph.groups.iter().any(|g| {
                            self.selected_nodes.iter().any(|id| g.node_ids.contains(id))
                        });
                        if has_group && ui.button("Ungroup").clicked() {
                            self.snapshot();
                            self.graph.groups.retain(|g| {
                                !self.selected_nodes.iter().any(|id| g.node_ids.contains(id))
                            });
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
            ui.menu_button("Edit", |ui| {
                if ui.add_enabled(self.history.can_undo(), egui::Button::new("Undo")).clicked() {
                    if let Some(prev) = self.history.undo(&self.graph) {
                        self.graph = prev;
                        self.selected_nodes.clear();
                    }
                    ui.close_menu();
                }
                if ui.add_enabled(self.history.can_redo(), egui::Button::new("Redo")).clicked() {
                    if let Some(next) = self.history.redo(&self.graph) {
                        self.graph = next;
                        self.selected_nodes.clear();
                    }
                    ui.close_menu();
                }
                ui.separator();
                if ui.button("Select All").clicked() {
                    self.selected_nodes = self.graph.nodes.keys().copied().collect();
                    ui.close_menu();
                }
                if ui.add_enabled(!self.selected_nodes.is_empty(), egui::Button::new("Delete Selected")).clicked() {
                    self.snapshot();
                    let ids: Vec<Uuid> = self.selected_nodes.drain(..).collect();
                    for id in ids {
                        self.graph.remove_node(id);
                    }
                    ui.close_menu();
                }
            });
            ui.menu_button("View", |ui| {
                if ui.checkbox(&mut self.show_left_panel, "Left Panel").changed() {
                    ui.close_menu();
                }
                if ui.checkbox(&mut self.show_validation_panel, "Validation Panel").changed() {
                    ui.close_menu();
                }
                if ui.checkbox(&mut self.show_playtest, "Playtest Panel").changed() {
                    ui.close_menu();
                }
                ui.separator();
                if ui.button(if self.dark_theme { "Light Theme" } else { "Dark Theme" }).clicked() {
                    self.dark_theme = !self.dark_theme;
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
                        self.status_message = Some(("Saved".to_string(), Instant::now()));
                        self.last_auto_save = Instant::now();
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

    /// Search nodes for matching text content.
    fn update_search_results(&mut self) {
        self.search_results.clear();
        let query = self.search_query.to_lowercase();
        if query.is_empty() {
            return;
        }
        for node in self.graph.nodes.values() {
            if self.node_matches_query(node, &query) {
                self.search_results.push(node.id);
            }
        }
        if self.search_index >= self.search_results.len() {
            self.search_index = 0;
        }
    }

    fn node_matches_query(&self, node: &Node, query: &str) -> bool {
        use crate::model::node::NodeType;

        if node.title().to_lowercase().contains(query) {
            return true;
        }
        match &node.node_type {
            NodeType::Dialogue(data) => {
                data.text.to_lowercase().contains(query)
                    || data.speaker_name.to_lowercase().contains(query)
                    || data.emotion.to_lowercase().contains(query)
            }
            NodeType::Choice(data) => {
                data.prompt.to_lowercase().contains(query)
                    || data.choices.iter().any(|c| c.text.to_lowercase().contains(query))
            }
            NodeType::Condition(data) => data.variable_name.to_lowercase().contains(query),
            NodeType::Event(data) => data.actions.iter().any(|a| {
                a.key.to_lowercase().contains(query)
            }),
            NodeType::End(data) => data.tag.to_lowercase().contains(query),
            _ => false,
        }
    }

    fn focus_search_result(&mut self) {
        if let Some(&node_id) = self.search_results.get(self.search_index) {
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

    fn draw_groups(&self, painter: &egui::Painter) {
        for group in &self.graph.groups {
            if group.node_ids.is_empty() {
                continue;
            }
            // Compute bounding box of all nodes in the group
            let mut min_x = f32::MAX;
            let mut min_y = f32::MAX;
            let mut max_x = f32::MIN;
            let mut max_y = f32::MIN;

            for &node_id in &group.node_ids {
                if let Some(node) = self.graph.nodes.get(&node_id) {
                    let rect = node_widget::node_rect(node);
                    min_x = min_x.min(rect.min.x);
                    min_y = min_y.min(rect.min.y);
                    max_x = max_x.max(rect.max.x);
                    max_y = max_y.max(rect.max.y);
                }
            }

            if min_x >= max_x || min_y >= max_y {
                continue;
            }

            // Add padding around the group
            let pad = 20.0;
            let group_rect = Rect::from_min_max(
                Pos2::new(min_x - pad, min_y - pad - 20.0),
                Pos2::new(max_x + pad, max_y + pad),
            );
            let screen_rect = self.canvas.canvas_rect_to_screen(group_rect);

            let [r, g, b, a] = group.color;
            painter.rect_filled(
                screen_rect,
                8.0,
                Color32::from_rgba_premultiplied(r, g, b, a),
            );
            painter.rect_stroke(
                screen_rect,
                8.0,
                Stroke::new(1.0, Color32::from_rgba_premultiplied(r, g, b, (a as u16 * 3).min(255) as u8)),
                StrokeKind::Inside,
            );

            // Group label
            let font_size = 12.0 * self.canvas.zoom;
            painter.text(
                Pos2::new(screen_rect.min.x + 8.0, screen_rect.min.y + 4.0),
                egui::Align2::LEFT_TOP,
                &group.name,
                egui::FontId::proportional(font_size),
                Color32::from_rgba_premultiplied(r, g, b, 200),
            );
        }
    }

    fn draw_minimap(&mut self, ui: &egui::Ui, painter: &egui::Painter, canvas_rect: Rect) {
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

        // Add margin
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

        // Inverse: minimap screen position → canvas position
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
        let vp_rect = Rect::from_min_max(vp_mapped_min, vp_mapped_max).intersect(inner_rect);
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

    fn show_search_bar(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.label("Find:");
            let changed = ui
                .text_edit_singleline(&mut self.search_query)
                .changed();
            if changed {
                self.update_search_results();
            }

            let count = self.search_results.len();
            if !self.search_query.is_empty() {
                if count > 0 {
                    ui.label(format!("{}/{count}", self.search_index + 1));
                    if ui.small_button("<").clicked() {
                        self.search_index = if self.search_index == 0 {
                            count - 1
                        } else {
                            self.search_index - 1
                        };
                        self.focus_search_result();
                    }
                    if ui.small_button(">").clicked() {
                        self.search_index = (self.search_index + 1) % count;
                        self.focus_search_result();
                    }
                } else {
                    ui.colored_label(Color32::from_rgb(255, 100, 100), "No matches");
                }
            }

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.small_button("X").clicked() {
                    self.show_search = false;
                    self.show_replace = false;
                    self.search_query.clear();
                    self.search_results.clear();
                    self.replace_query.clear();
                }
                let toggle_label = if self.show_replace { "Hide Replace" } else { "Replace" };
                if ui.small_button(toggle_label).clicked() {
                    self.show_replace = !self.show_replace;
                }
            });
        });

        if self.show_replace {
            ui.horizontal(|ui| {
                ui.label("Replace:");
                ui.text_edit_singleline(&mut self.replace_query);

                let has_matches = !self.search_query.is_empty() && !self.search_results.is_empty();
                if ui.add_enabled(has_matches, egui::Button::new("Replace")).clicked() {
                    self.replace_in_current();
                }
                if ui.add_enabled(has_matches, egui::Button::new("Replace All")).clicked() {
                    self.replace_all();
                }
            });
        }
    }

    /// Replace the search query in the currently focused search result node.
    fn replace_in_current(&mut self) {
        let Some(&node_id) = self.search_results.get(self.search_index) else {
            return;
        };
        self.snapshot();
        let query = self.search_query.clone();
        let replacement = self.replace_query.clone();
        if let Some(node) = self.graph.nodes.get_mut(&node_id) {
            replace_in_node(node, &query, &replacement);
        }
        self.update_search_results();
        // Keep index in bounds
        if !self.search_results.is_empty() {
            if self.search_index >= self.search_results.len() {
                self.search_index = 0;
            }
            self.focus_search_result();
        }
        self.status_message = Some(("Replaced in current node".to_string(), Instant::now()));
    }

    /// Replace the search query in all matching nodes.
    fn replace_all(&mut self) {
        if self.search_results.is_empty() {
            return;
        }
        self.snapshot();
        let query = self.search_query.clone();
        let replacement = self.replace_query.clone();
        let ids: Vec<Uuid> = self.search_results.clone();
        let mut count = 0;
        for id in &ids {
            if let Some(node) = self.graph.nodes.get_mut(id) {
                count += replace_in_node(node, &query, &replacement);
            }
        }
        self.update_search_results();
        self.search_index = 0;
        self.status_message = Some((
            format!("{count} replacement(s) across {} node(s)", ids.len()),
            Instant::now(),
        ));
    }

    fn show_status_bar(&mut self, ui: &mut egui::Ui) {
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

            // Status message (auto-save, etc.)
            if let Some((ref msg, when)) = self.status_message {
                if when.elapsed().as_secs() < 3 {
                    ui.separator();
                    ui.colored_label(Color32::from_rgb(100, 200, 100), msg);
                }
            }

            // Validation indicator (right-aligned)
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                let errors = self.validation_warnings.iter().filter(|w| w.severity == Severity::Error).count();
                let warns = self.validation_warnings.iter().filter(|w| w.severity == Severity::Warning).count();

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

                if ui.add(egui::Label::new(egui::RichText::new(label).color(color)).sense(Sense::click())).clicked() {
                    self.show_validation_panel = !self.show_validation_panel;
                }
            });
        });
    }

    fn show_validation_panel(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.heading("Validation");
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.small_button("X").clicked() {
                    self.show_validation_panel = false;
                }
            });
        });
        ui.separator();

        if self.validation_warnings.is_empty() {
            ui.label("No issues found.");
        } else {
            egui::ScrollArea::vertical().max_height(200.0).show(ui, |ui| {
                for warning in &self.validation_warnings {
                    let (icon, color) = match warning.severity {
                        Severity::Error => ("E", Color32::from_rgb(255, 100, 100)),
                        Severity::Warning => ("!", Color32::from_rgb(255, 200, 100)),
                    };

                    ui.horizontal(|ui| {
                        ui.colored_label(color, icon);
                        let resp = ui.label(&warning.message);
                        // Click to select the node
                        if let Some(node_id) = warning.node_id {
                            if resp.interact(Sense::click()).clicked() {
                                self.selected_nodes.clear();
                                self.selected_nodes.push(node_id);
                                // Center canvas on the node
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
}

impl eframe::App for TaleNodeApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Apply theme
        ctx.set_visuals(if self.dark_theme {
            egui::Visuals::dark()
        } else {
            egui::Visuals::light()
        });

        // Run validation each frame (cheap for typical graph sizes)
        self.validation_warnings = validator::validate(&self.graph);

        // Auto-save every 60 seconds if project has a save path
        if self.project_path.is_some() && self.last_auto_save.elapsed().as_secs() >= 60 {
            self.last_auto_save = Instant::now();
            self.do_save(false);
            self.status_message = Some(("Auto-saved".to_string(), Instant::now()));
        }

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
        if ctx.input(|i| i.modifiers.command && i.key_pressed(egui::Key::F)) {
            self.show_search = true;
        }
        // Replace: Ctrl+H on Windows/Linux, Cmd+Shift+H on macOS
        // (Cmd+H is reserved by macOS for "Hide Application")
        if ctx.input(|i| {
            if cfg!(target_os = "macos") {
                i.modifiers.command && i.modifiers.shift && i.key_pressed(egui::Key::H)
            } else {
                i.modifiers.command && i.key_pressed(egui::Key::H)
            }
        }) {
            self.show_search = true;
            self.show_replace = true;
        }
        if ctx.input(|i| i.modifiers.command && i.key_pressed(egui::Key::N)) {
            self.graph = DialogueGraph::new();
            self.graph.add_node(Node::new_start([100.0, 200.0]));
            self.selected_nodes.clear();
            self.project_name = "Untitled".to_string();
            self.project_path = None;
            self.history.clear();
        }
        // Ctrl+A: select all nodes
        if ctx.input(|i| i.modifiers.command && i.key_pressed(egui::Key::A)) {
            self.selected_nodes = self.graph.nodes.keys().copied().collect();
        }
        // Ctrl+D: duplicate selected nodes
        if ctx.input(|i| i.modifiers.command && i.key_pressed(egui::Key::D)) && !self.selected_nodes.is_empty() {
            self.snapshot();
            let mut new_ids = Vec::new();
            for &id in &self.selected_nodes.clone() {
                if let Some(node) = self.graph.nodes.get(&id) {
                    let mut dup = node.clone();
                    dup.id = Uuid::new_v4();
                    dup.position[0] += 30.0;
                    dup.position[1] += 30.0;
                    // Give new UUIDs to all ports
                    for port in &mut dup.inputs {
                        port.id = crate::model::port::PortId(Uuid::new_v4());
                    }
                    for port in &mut dup.outputs {
                        port.id = crate::model::port::PortId(Uuid::new_v4());
                    }
                    new_ids.push(dup.id);
                    self.graph.add_node(dup);
                }
            }
            self.selected_nodes = new_ids;
        }
        if ctx.input(|i| i.key_pressed(egui::Key::Escape)) && self.show_search {
            self.show_search = false;
            self.show_replace = false;
            self.search_query.clear();
            self.search_results.clear();
            self.replace_query.clear();
        }
        // Enter cycles through search results
        if self.show_search
            && !self.search_results.is_empty()
            && ctx.input(|i| i.key_pressed(egui::Key::Enter))
        {
            self.search_index = (self.search_index + 1) % self.search_results.len();
            self.focus_search_result();
        }

        // Menu bar at top
        egui::TopBottomPanel::top("menu_bar").show(ctx, |ui| {
            self.show_menu_bar(ui);
        });

        // Search bar (below menu bar)
        if self.show_search {
            egui::TopBottomPanel::top("search_bar").show(ctx, |ui| {
                self.show_search_bar(ui);
            });
        }

        // Status bar at bottom
        egui::TopBottomPanel::bottom("status_bar").show(ctx, |ui| {
            self.show_status_bar(ui);
        });

        // Validation panel (above status bar)
        if self.show_validation_panel {
            egui::TopBottomPanel::bottom("validation_panel")
                .resizable(true)
                .default_height(150.0)
                .show(ctx, |ui| {
                    self.show_validation_panel(ui);
                });
        }

        // Playtest panel
        if self.show_playtest {
            egui::TopBottomPanel::bottom("playtest_panel")
                .resizable(true)
                .default_height(250.0)
                .show(ctx, |ui| {
                    crate::ui::playtest::show_playtest_panel(
                        ui,
                        &mut self.playtest,
                        &self.graph,
                        &mut self.selected_nodes,
                    );
                });
        }

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

/// Replace all case-insensitive occurrences of `query` in a node's text fields.
/// Returns the number of individual string replacements made.
fn replace_in_node(node: &mut Node, query: &str, replacement: &str) -> usize {
    use crate::model::node::NodeType;
    let mut count = 0;

    match &mut node.node_type {
        NodeType::Dialogue(data) => {
            count += replace_in_string(&mut data.text, query, replacement);
            count += replace_in_string(&mut data.speaker_name, query, replacement);
            count += replace_in_string(&mut data.emotion, query, replacement);
        }
        NodeType::Choice(data) => {
            count += replace_in_string(&mut data.prompt, query, replacement);
            for choice in &mut data.choices {
                count += replace_in_string(&mut choice.text, query, replacement);
            }
        }
        NodeType::Condition(data) => {
            count += replace_in_string(&mut data.variable_name, query, replacement);
        }
        NodeType::Event(data) => {
            for action in &mut data.actions {
                count += replace_in_string(&mut action.key, query, replacement);
            }
        }
        NodeType::End(data) => {
            count += replace_in_string(&mut data.tag, query, replacement);
        }
        _ => {}
    }

    // Sync port labels for Choice nodes after replacement
    if let NodeType::Choice(data) = &node.node_type {
        for (i, choice) in data.choices.iter().enumerate() {
            if let Some(port) = node.outputs.get_mut(i) {
                port.label.clone_from(&choice.text);
            }
        }
    }

    count
}

/// Case-insensitive replace of all occurrences in a string.
/// Returns 1 if any replacement was made, 0 otherwise.
fn replace_in_string(s: &mut String, query: &str, replacement: &str) -> usize {
    let lower = s.to_lowercase();
    let query_lower = query.to_lowercase();
    if !lower.contains(&query_lower) {
        return 0;
    }
    // Build result preserving structure but replacing case-insensitively
    let mut result = String::with_capacity(s.len());
    let mut remaining = s.as_str();
    while let Some(pos) = remaining.to_lowercase().find(&query_lower) {
        result.push_str(&remaining[..pos]);
        result.push_str(replacement);
        remaining = &remaining[pos + query.len()..];
    }
    result.push_str(remaining);
    *s = result;
    1
}
