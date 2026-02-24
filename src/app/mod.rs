mod canvas;
mod file_io;
mod menu;
mod panel_handlers;
mod panels;
mod search;

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
    /// Batch audio assignment state.
    audio_manager: crate::ui::audio_manager::AudioManagerState,
    /// Whether the bookmark panel is visible.
    show_bookmark_panel: bool,
    /// Tag filter for bookmark panel.
    bookmark_tag_filter: Option<String>,
    /// Text input for new tag in bookmark panel.
    bookmark_new_tag_text: String,
    /// Text input for new tag in inspector.
    inspector_new_tag_text: String,
    /// Whether the comments panel is visible.
    show_comments_panel: bool,
    /// Filter for the comments panel.
    comments_filter: Option<crate::model::review::ReviewStatus>,
    /// Which node is targeted for new comments.
    comment_target_node: Option<Uuid>,
    /// Text input for new comment.
    new_comment_text: String,
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
            show_bookmark_panel: false,
            bookmark_tag_filter: None,
            bookmark_new_tag_text: String::new(),
            inspector_new_tag_text: String::new(),
            show_comments_panel: false,
            comments_filter: None,
            comment_target_node: None,
            new_comment_text: String::new(),
        }
    }

    /// Save a snapshot for undo before mutating the graph.
    fn snapshot(&mut self) {
        self.history.save_snapshot(&self.graph);
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
        if ctx.input(|i| i.key_pressed(egui::Key::Escape)) && self.show_search {
            self.show_search = false;
            self.show_replace = false;
            self.search_query.clear();
            self.search_results.clear();
            self.replace_query.clear();
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

        // Main canvas
        egui::CentralPanel::default()
            .frame(egui::Frame::NONE)
            .show(ctx, |ui| {
                self.show_canvas(ui);
            });
    }
}
