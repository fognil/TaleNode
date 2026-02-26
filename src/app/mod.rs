mod async_handlers;
pub mod async_runtime;
mod bark_handlers;
mod canvas;
mod collab_handlers;
mod confirm;
mod context_menu;
mod dock;
mod file_import;
mod file_io;
mod file_io_locale;
mod menu;
mod panel_handlers;
mod panels;
mod quest_handlers;
mod search;
pub mod settings;
mod subgraph_nav;
mod templates;
pub(super) mod theme;
mod voice_handlers;
mod world_handlers;
mod writing_handlers;

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
    dock_state: Option<egui_dock::DockState<dock::DockTab>>,
    history: UndoHistory,
    validation_warnings: Vec<ValidationWarning>,
    search_query: String,
    show_search: bool,
    search_results: Vec<Uuid>,
    search_index: usize,
    replace_query: String,
    show_replace: bool,
    playtest: PlaytestState,
    last_auto_save: Instant,
    status_message: Option<(String, Instant, bool)>,
    audio_manager: crate::ui::audio_manager::AudioManagerState,
    version_new_desc: String,
    versions: Vec<crate::model::project::VersionSnapshot>,
    version_compare_selection: [Option<usize>; 2],
    version_diff_result: Option<crate::model::graph_diff::GraphDiff>,
    bookmark_tag_filter: Option<String>,
    bookmark_new_tag_text: String,
    inspector_new_tag_text: String,
    comments_filter: Option<crate::model::review::ReviewStatus>,
    comment_target_node: Option<Uuid>,
    new_comment_text: String,
    subgraph_stack: Vec<subgraph_nav::SubGraphFrame>,
    template_library: crate::model::template::TemplateLibrary,
    template_new_name: String,
    script_panel_text: String,
    script_panel_dirty: bool,
    script_panel_stale: bool,
    pending_confirmation: Option<confirm::PendingAction>,
    last_inspector_focus_count: usize,
    active_locale: Option<String>,
    locale_filter_untranslated: bool,
    locale_new_name: String,
    settings: settings::AppSettings,
    settings_open: bool,
    tokio_runtime: tokio::runtime::Runtime,
    async_rx: std::sync::mpsc::Receiver<async_runtime::AsyncResult>,
    async_tx: std::sync::mpsc::Sender<async_runtime::AsyncResult>,
    translation_in_progress: bool,
    voice_generation_in_progress: bool,
    available_voices: Vec<async_runtime::VoiceInfo>,
    collab_state: Option<crate::collab::CollabState>,
    collab_host_input: String,
    collab_port_input: u16,
    bark_selected_character: Option<Uuid>,
    world_category_filter: Option<crate::model::world::EntityCategory>,
    writing_in_progress: bool,
    writing_suggestions: Option<(Uuid, Vec<String>)>,
    writing_tone_report: Option<(Uuid, String)>,
    writing_instruction: String,
    writing_choice_count: usize,
}

impl TaleNodeApp {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        let (async_tx, async_rx) = std::sync::mpsc::channel();
        Self {
            graph: {
                let mut g = DialogueGraph::new();
                g.add_node(Node::new_start([100.0, 200.0]));
                g
            },
            canvas: CanvasState::default(),
            selected_nodes: Vec::new(),
            interaction: InteractionState::Idle,
            context_menu_pos: None,
            project_name: "Untitled".to_string(),
            project_path: None,
            dock_state: Some(dock::default_dock_state()),
            history: UndoHistory::new(),
            validation_warnings: Vec::new(),
            search_query: String::new(),
            show_search: false,
            search_results: Vec::new(),
            search_index: 0,
            replace_query: String::new(),
            show_replace: false,
            playtest: PlaytestState::new(),
            last_auto_save: Instant::now(),
            status_message: None,
            audio_manager: Default::default(),
            version_new_desc: String::new(),
            versions: Vec::new(),
            version_compare_selection: [None; 2],
            version_diff_result: None,
            bookmark_tag_filter: None,
            bookmark_new_tag_text: String::new(),
            inspector_new_tag_text: String::new(),
            comments_filter: None,
            comment_target_node: None,
            new_comment_text: String::new(),
            subgraph_stack: Vec::new(),
            template_library: Self::load_template_library(),
            template_new_name: String::new(),
            script_panel_text: String::new(),
            script_panel_dirty: false,
            script_panel_stale: false,
            pending_confirmation: None,
            last_inspector_focus_count: 0,
            active_locale: None,
            locale_filter_untranslated: false,
            locale_new_name: String::new(),
            settings: settings::AppSettings::load(),
            settings_open: false,
            tokio_runtime: tokio::runtime::Runtime::new()
                .expect("Failed to create tokio runtime"),
            async_rx,
            async_tx,
            translation_in_progress: false,
            voice_generation_in_progress: false,
            available_voices: Vec::new(),
            collab_state: None,
            collab_host_input: "127.0.0.1".to_string(),
            collab_port_input: 9847,
            bark_selected_character: None,
            world_category_filter: None,
            writing_in_progress: false,
            writing_suggestions: None,
            writing_tone_report: None,
            writing_instruction: String::new(),
            writing_choice_count: 3,
        }
    }

    /// Save a snapshot for undo before mutating the graph.
    fn snapshot(&mut self) {
        self.history.save_snapshot(&self.graph);
        if self.has_script_tab() {
            self.script_panel_stale = true;
        }
    }
}

impl eframe::App for TaleNodeApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Apply theme
        ctx.set_visuals(theme::build_visuals(&self.settings.theme));
        theme::apply_font_size(ctx, self.settings.theme.font_size);

        // Run validation each frame (cheap for typical graph sizes)
        self.validation_warnings = validator::validate(&self.graph);

        // Auto-save every 60 seconds if project has a save path
        if self.project_path.is_some() && self.last_auto_save.elapsed().as_secs() >= 60 {
            self.last_auto_save = Instant::now();
            self.do_save(false);
            self.status_message = Some(("Auto-saved".to_string(), Instant::now(), false));
        }

        // Poll async results (translation, voice, collab)
        self.poll_async_results();

        // Keyboard shortcuts
        self.handle_keyboard_shortcuts(ctx);

        // Confirmation dialog (modal)
        self.show_confirmation_dialog(ctx);

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

        // Audio manager window (floating, stays outside dock)
        crate::ui::audio_manager::show_audio_manager(
            ctx,
            &mut self.audio_manager,
            &mut self.graph,
        );

        // Settings window (floating)
        if self.settings_open {
            settings::show_settings_window(ctx, &mut self.settings, &mut self.settings_open);
        }

        // Dockable panel layout (replaces all SidePanel/CentralPanel calls)
        let prev_sel = self.last_inspector_focus_count;
        self.show_dock(ctx);

        // Auto-focus Inspector tab when selection transitions to exactly 1 node
        let cur_sel = self.selected_nodes.len();
        if cur_sel == 1 && prev_sel != 1 {
            self.focus_dock_tab(dock::DockTab::Inspector);
        }
        self.last_inspector_focus_count = cur_sel;
    }
}

impl TaleNodeApp {
    fn handle_keyboard_shortcuts(&mut self, ctx: &egui::Context) {
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
            self.pending_confirmation = Some(confirm::PendingAction::NewProject);
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
    }
}
