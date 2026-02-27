mod async_handlers;
pub mod async_runtime;
mod bark_handlers;
mod canvas;
mod canvas_tooltip;
mod collab_handlers;
mod confirm;
mod context_menu;
mod dock;
mod extension_handlers;
mod file_import;
mod file_io;
mod file_io_locale;
mod menu;
mod panel_handlers;
mod panels;
mod quest_handlers;
mod search;
pub mod settings;
mod shortcuts;
mod subgraph_nav;
mod templates;
pub(super) mod theme;
mod timeline_handlers;
mod voice_handlers;
mod world_handlers;
mod writing_handlers;

use egui::Pos2;
use std::collections::HashSet;
use std::time::Instant;
use uuid::Uuid;

use crate::actions::history::UndoHistory;
use crate::model::graph::DialogueGraph;
use crate::model::node::Node;
use crate::model::port::{PortDirection, PortId};
use crate::ui::canvas::CanvasState;
use crate::ui::playtest::PlaytestState;
use crate::ui::spatial_grid::SpatialGrid;
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

/// Filter state for focusing the canvas on a subset of nodes.
#[derive(Debug, Clone, Default)]
struct CanvasFilter {
    active: bool,
    tags: Vec<String>,
    node_types: HashSet<std::mem::Discriminant<crate::model::node::NodeType>>,
    visible_cache: HashSet<Uuid>,
}

/// Top-level application state.
pub struct TaleNodeApp {
    pub graph: DialogueGraph,
    pub canvas: CanvasState,
    pub selected_nodes: HashSet<Uuid>,
    interaction: InteractionState,
    context_menu_pos: Option<[f32; 2]>,
    project_name: String,
    project_path: Option<std::path::PathBuf>,
    dock_state: Option<egui_dock::DockState<dock::DockTab>>,
    history: UndoHistory,
    validation_warnings: Vec<ValidationWarning>,
    validation_dirty: bool,
    search_query: String,
    show_search: bool,
    search_results: Vec<Uuid>,
    search_results_set: HashSet<Uuid>,
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
    discovered_plugins: Vec<crate::model::plugin::PluginManifest>,
    plugin_last_result: Option<(String, bool)>,
    writing_in_progress: bool,
    writing_suggestions: Option<(Uuid, Vec<String>)>,
    writing_tone_report: Option<(Uuid, String)>,
    writing_instruction: String,
    writing_choice_count: usize,
    available_ai_models: Vec<String>,
    ai_models_loading: bool,
    spatial_grid: SpatialGrid,
    minimap_bounds_dirty: bool,
    minimap_bounds_cache: Option<egui::Rect>,
    canvas_filter: CanvasFilter,
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
            selected_nodes: HashSet::new(),
            interaction: InteractionState::Idle,
            context_menu_pos: None,
            project_name: "Untitled".to_string(),
            project_path: None,
            dock_state: Some(dock::default_dock_state()),
            history: UndoHistory::new(),
            validation_warnings: Vec::new(),
            validation_dirty: true,
            search_query: String::new(),
            show_search: false,
            search_results: Vec::new(),
            search_results_set: HashSet::new(),
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
            discovered_plugins: Vec::new(),
            plugin_last_result: None,
            writing_in_progress: false,
            writing_suggestions: None,
            writing_tone_report: None,
            writing_instruction: String::new(),
            writing_choice_count: 3,
            available_ai_models: Vec::new(),
            ai_models_loading: false,
            spatial_grid: SpatialGrid::default(),
            minimap_bounds_dirty: true,
            minimap_bounds_cache: None,
            canvas_filter: CanvasFilter::default(),
        }
    }

    /// Save a snapshot for undo before mutating the graph.
    fn snapshot(&mut self) {
        self.history.save_snapshot(&self.graph);
        self.spatial_grid.mark_dirty();
        self.validation_dirty = true;
        self.minimap_bounds_dirty = true;
        if self.has_script_tab() {
            self.script_panel_stale = true;
        }
    }
}

impl eframe::App for TaleNodeApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        ctx.set_visuals(theme::build_visuals(&self.settings.theme));
        theme::apply_font_size(ctx, self.settings.theme.font_size);

        if self.validation_dirty {
            self.validation_warnings = validator::validate(&self.graph);
            self.validation_dirty = false;
        }

        if self.project_path.is_some() && self.last_auto_save.elapsed().as_secs() >= 60 {
            self.last_auto_save = Instant::now();
            self.do_save(false);
            self.status_message = Some(("Auto-saved".to_string(), Instant::now(), false));
        }

        self.poll_async_results();
        self.handle_keyboard_shortcuts(ctx);
        self.show_confirmation_dialog(ctx);

        egui::TopBottomPanel::top("menu_bar").show(ctx, |ui| {
            self.show_menu_bar(ui);
        });
        if self.show_search {
            egui::TopBottomPanel::top("search_bar").show(ctx, |ui| {
                self.show_search_bar(ui);
            });
        }
        if self.canvas_filter.active {
            egui::TopBottomPanel::top("filter_bar").show(ctx, |ui| {
                self.show_filter_bar(ui);
            });
        }
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

        egui::TopBottomPanel::bottom("status_bar").show(ctx, |ui| {
            self.show_status_bar(ui);
        });
        crate::ui::audio_manager::show_audio_manager(
            ctx,
            &mut self.audio_manager,
            &mut self.graph,
        );

        // Settings window (floating)
        if self.settings_open {
            let action = settings::show_settings_window(
                ctx,
                &mut self.settings,
                &mut self.settings_open,
                &self.available_ai_models,
                self.ai_models_loading,
            );
            match action {
                Some(settings::SettingsAction::FetchModels) => {
                    self.start_fetch_ai_models();
                }
                Some(settings::SettingsAction::ProviderChanged) => {
                    self.available_ai_models.clear();
                }
                None => {}
            }
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

