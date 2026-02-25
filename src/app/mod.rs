mod canvas;
mod context_menu;
mod file_io;
mod menu;
mod panel_handlers;
mod panels;
mod search;
mod subgraph_nav;
mod templates;

use egui::Pos2;
use std::time::Instant;
use uuid::Uuid;

use crate::actions::history::UndoHistory;
use crate::model::graph::DialogueGraph;
use crate::model::node::Node;
use crate::model::port::{PortDirection, PortId};
use crate::ui::canvas::CanvasState;
use crate::ui::playtest::PlaytestState;
use crate::validation::validator::{self, ValidationWarning};

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
    context_menu_pos: Option<[f32; 2]>,
    project_name: String,
    project_path: Option<std::path::PathBuf>,
    show_left_panel: bool,
    history: UndoHistory,
    validation_warnings: Vec<ValidationWarning>,
    show_validation_panel: bool,
    search_query: String,
    show_search: bool,
    search_results: Vec<Uuid>,
    search_index: usize,
    replace_query: String,
    show_replace: bool,
    dark_theme: bool,
    playtest: PlaytestState,
    show_playtest: bool,
    last_auto_save: Instant,
    status_message: Option<(String, Instant, bool)>,
    audio_manager: crate::ui::audio_manager::AudioManagerState,
    show_version_panel: bool,
    version_new_desc: String,
    versions: Vec<crate::model::project::VersionSnapshot>,
    version_compare_selection: [Option<usize>; 2],
    version_diff_result: Option<crate::model::graph_diff::GraphDiff>,
    show_analytics_panel: bool,
    show_bookmark_panel: bool,
    bookmark_tag_filter: Option<String>,
    bookmark_new_tag_text: String,
    inspector_new_tag_text: String,
    show_comments_panel: bool,
    comments_filter: Option<crate::model::review::ReviewStatus>,
    comment_target_node: Option<Uuid>,
    new_comment_text: String,
    subgraph_stack: Vec<subgraph_nav::SubGraphFrame>,
    template_library: crate::model::template::TemplateLibrary,
    show_template_panel: bool,
    template_new_name: String,
    show_script_panel: bool,
    script_panel_text: String,
    script_panel_dirty: bool,
    script_panel_stale: bool,
}

impl TaleNodeApp {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        let mut graph = DialogueGraph::new();
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
            audio_manager: Default::default(),
            show_version_panel: false,
            version_new_desc: String::new(),
            versions: Vec::new(),
            version_compare_selection: [None; 2],
            version_diff_result: None,
            show_analytics_panel: false,
            show_bookmark_panel: false,
            bookmark_tag_filter: None,
            bookmark_new_tag_text: String::new(),
            inspector_new_tag_text: String::new(),
            show_comments_panel: false,
            comments_filter: None,
            comment_target_node: None,
            new_comment_text: String::new(),
            subgraph_stack: Vec::new(),
            template_library: Self::load_template_library(),
            show_template_panel: false,
            template_new_name: String::new(),
            show_script_panel: false,
            script_panel_text: String::new(),
            script_panel_dirty: false,
            script_panel_stale: false,
        }
    }

    /// Save a snapshot for undo before mutating the graph.
    fn snapshot(&mut self) {
        self.history.save_snapshot(&self.graph);
        if self.show_script_panel { self.script_panel_stale = true; }
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
            self.status_message = Some(("Auto-saved".to_string(), Instant::now(), false));
        }

        // Keyboard shortcuts
        if ctx.input(|i| {
            i.modifiers.command && !i.modifiers.shift && i.key_pressed(egui::Key::S)
        }) {
            self.do_save(false);
        }
        if ctx.input(|i| i.modifiers.command && i.key_pressed(egui::Key::O)) {
            self.do_open();
        }
        if ctx.input(|i| {
            i.modifiers.command && !i.modifiers.shift && i.key_pressed(egui::Key::Z)
        }) {
            if let Some(prev) = self.history.undo(&self.graph) {
                self.graph = prev;
                self.selected_nodes.clear();
            }
        }
        if ctx.input(|i| {
            i.modifiers.command && i.modifiers.shift && i.key_pressed(egui::Key::Z)
        }) {
            if let Some(next) = self.history.redo(&self.graph) {
                self.graph = next;
                self.selected_nodes.clear();
            }
        }
        if ctx.input(|i| i.modifiers.command && i.key_pressed(egui::Key::F)) {
            self.show_search = true;
        }
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
            self.do_new_project();
        }
        if ctx.input(|i| i.modifiers.command && i.key_pressed(egui::Key::A)) {
            self.selected_nodes = self.graph.nodes.keys().copied().collect();
        }
        if ctx.input(|i| i.modifiers.command && i.key_pressed(egui::Key::D))
            && !self.selected_nodes.is_empty()
        {
            self.duplicate_selected();
        }
        if !self.show_search
            && ctx.input(|i| !i.modifiers.command && i.key_pressed(egui::Key::F))
        {
            let size = ctx.screen_rect().size();
            self.canvas.zoom_to_fit(&self.graph.nodes, size);
        }
        if ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
            if self.show_search {
                self.show_search = false;
                self.show_replace = false;
                self.search_query.clear();
                self.search_results.clear();
                self.replace_query.clear();
            } else if self.is_in_subgraph() {
                self.exit_subgraph();
            }
        }
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

        // Breadcrumb bar for sub-graph navigation
        if self.is_in_subgraph() {
            egui::TopBottomPanel::top("breadcrumb_bar").show(ctx, |ui| {
                ui.horizontal(|ui| {
                    if ui.button("< Back").clicked() {
                        self.exit_subgraph();
                    }
                    ui.separator();
                    ui.label("Root");
                    for label in self.breadcrumb_labels() {
                        ui.label(">");
                        ui.label(label);
                    }
                });
            });
        }

        // Status bar at bottom
        egui::TopBottomPanel::bottom("status_bar").show(ctx, |ui| {
            self.show_status_bar(ui);
        });

        // Comments + Bookmark panels
        self.show_bottom_panels(ctx);

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

        // Audio manager window
        crate::ui::audio_manager::show_audio_manager(
            ctx,
            &mut self.audio_manager,
            &mut self.graph,
        );

        // Left panel (variables, characters)
        if self.show_left_panel {
            let pre_graph = self.graph.clone();
            egui::SidePanel::left("left_panel")
                .default_width(200.0)
                .min_width(150.0)
                .show(ctx, |ui| {
                    egui::ScrollArea::vertical().show(ui, |ui| {
                        let changed = crate::ui::left_panel::show_left_panel(
                            ui,
                            &mut self.graph,
                        );
                        if changed {
                            self.history.push_undo(pre_graph);
                        }
                    });
                });
        }

        // Inspector panel (right side) — only when exactly 1 node selected
        if self.selected_nodes.len() == 1 {
            let selected_id = self.selected_nodes[0];
            let pre_graph = self.graph.clone();
            egui::SidePanel::right("inspector")
                .default_width(280.0)
                .min_width(220.0)
                .show(ctx, |ui| {
                    egui::ScrollArea::vertical().show(ui, |ui| {
                        let changed = crate::ui::inspector::show_inspector(
                            ui,
                            &mut self.graph,
                            selected_id,
                            &mut self.inspector_new_tag_text,
                        );
                        if changed {
                            self.history.push_undo(pre_graph);
                        }
                    });
                });
        }

        // Script editor panel (right side)
        self.show_script_side_panel(ctx);

        // Main canvas
        egui::CentralPanel::default()
            .frame(egui::Frame::NONE)
            .show(ctx, |ui| {
                self.show_canvas(ui);
            });
    }
}
