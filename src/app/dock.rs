use egui_dock::{DockArea, DockState, NodeIndex, SurfaceIndex, TabViewer};
use serde::{Deserialize, Serialize};

use super::TaleNodeApp;

/// Each dockable panel in the application.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub(super) enum DockTab {
    Canvas,
    LeftPanel,
    Inspector,
    ScriptEditor,
    Validation,
    Playtest,
    Comments,
    Bookmarks,
    Analytics,
    VersionHistory,
    Templates,
    Localization,
    VoiceGeneration,
    Collaboration,
    Barks,
    Quests,
    Timeline,
    WorldDatabase,
    WritingAssistant,
}

/// Which dock region a tab belongs to (for smart placement).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TabRegion {
    Canvas,
    Left,
    Right,
    Bottom,
}

impl DockTab {
    fn title(self) -> &'static str {
        match self {
            Self::Canvas => "Canvas",
            Self::LeftPanel => "Project",
            Self::Inspector => "Inspector",
            Self::ScriptEditor => "Script Editor",
            Self::Validation => "Validation",
            Self::Playtest => "Playtest",
            Self::Comments => "Comments",
            Self::Bookmarks => "Bookmarks",
            Self::Analytics => "Analytics",
            Self::VersionHistory => "Version History",
            Self::Templates => "Templates",
            Self::Localization => "Localization",
            Self::VoiceGeneration => "Voice Generation",
            Self::Collaboration => "Collaboration",
            Self::Barks => "Barks",
            Self::Quests => "Quests",
            Self::Timeline => "Timeline",
            Self::WorldDatabase => "World DB",
            Self::WritingAssistant => "AI Writer",
        }
    }

    fn region(self) -> TabRegion {
        match self {
            Self::Canvas => TabRegion::Canvas,
            Self::LeftPanel => TabRegion::Left,
            Self::Inspector | Self::ScriptEditor => TabRegion::Right,
            Self::Localization => TabRegion::Bottom,
            Self::VoiceGeneration => TabRegion::Bottom,
            Self::Collaboration => TabRegion::Bottom,
            Self::Barks => TabRegion::Bottom,
            Self::Quests => TabRegion::Bottom,
            Self::Timeline => TabRegion::Bottom,
            Self::WorldDatabase => TabRegion::Bottom,
            Self::WritingAssistant => TabRegion::Right,
            _ => TabRegion::Bottom,
        }
    }
}

/// Build the default dock layout matching the Unity-style arrangement.
pub(super) fn default_dock_state() -> DockState<DockTab> {
    let mut state = DockState::new(vec![DockTab::Canvas]);
    let surface = state.main_surface_mut();

    // Split left panel (15%) — returns [old=Canvas, new=LeftPanel]
    let [rest, _left] =
        surface.split_left(NodeIndex::root(), 0.15, vec![DockTab::LeftPanel]);

    // Split right panel from remaining (inspector + script as tabs, ~22% of total)
    let [center, _right] = surface.split_right(
        rest,
        0.78,
        vec![DockTab::Inspector, DockTab::ScriptEditor],
    );

    // Split bottom tab group from center (25% height)
    let [_canvas, _bottom] = surface.split_below(
        center,
        0.75,
        vec![
            DockTab::Validation,
            DockTab::Comments,
            DockTab::Bookmarks,
            DockTab::Analytics,
            DockTab::VersionHistory,
            DockTab::Templates,
            DockTab::Playtest,
        ],
    );

    state
}

/// Adapter that borrows TaleNodeApp to implement egui_dock::TabViewer.
pub(super) struct AppTabViewer<'a> {
    pub app: &'a mut TaleNodeApp,
}

impl TabViewer for AppTabViewer<'_> {
    type Tab = DockTab;

    fn title(&mut self, tab: &mut DockTab) -> egui::WidgetText {
        tab.title().into()
    }

    fn ui(&mut self, ui: &mut egui::Ui, tab: &mut DockTab) {
        match *tab {
            DockTab::Canvas => self.app.show_canvas(ui),
            DockTab::LeftPanel => self.app.render_left_panel_tab(ui),
            DockTab::Inspector => self.app.render_inspector_tab(ui),
            DockTab::ScriptEditor => self.app.render_script_tab(ui),
            DockTab::Validation => self.app.render_validation_tab(ui),
            DockTab::Playtest => self.app.render_playtest_tab(ui),
            DockTab::Comments => self.app.render_comments_tab(ui),
            DockTab::Bookmarks => self.app.render_bookmarks_tab(ui),
            DockTab::Analytics => self.app.render_analytics_tab(ui),
            DockTab::VersionHistory => self.app.render_version_tab(ui),
            DockTab::Templates => self.app.render_templates_tab(ui),
            DockTab::Localization => self.app.render_locale_tab(ui),
            DockTab::VoiceGeneration => self.app.render_voice_tab(ui),
            DockTab::Collaboration => self.app.render_collab_tab(ui),
            DockTab::Barks => self.app.render_barks_tab(ui),
            DockTab::Quests => self.app.render_quests_tab(ui),
            DockTab::Timeline => self.app.render_timeline_tab(ui),
            DockTab::WorldDatabase => self.app.render_world_database_tab(ui),
            DockTab::WritingAssistant => self.app.render_writing_assistant_tab(ui),
        }
    }

    fn closeable(&mut self, tab: &mut DockTab) -> bool {
        !matches!(tab, DockTab::Canvas)
    }

    fn on_close(&mut self, tab: &mut DockTab) -> bool {
        if *tab == DockTab::ScriptEditor {
            self.app.script_panel_text.clear();
            self.app.script_panel_dirty = false;
            self.app.script_panel_stale = false;
        }
        true
    }

    fn id(&mut self, tab: &mut DockTab) -> egui::Id {
        egui::Id::new(*tab as u8)
    }

    fn clear_background(&self, tab: &DockTab) -> bool {
        !matches!(tab, DockTab::Canvas)
    }

    fn scroll_bars(&self, tab: &DockTab) -> [bool; 2] {
        match tab {
            DockTab::Canvas => [false, false],
            _ => [true, true],
        }
    }
}

// --- Dock helper methods on TaleNodeApp ---

impl TaleNodeApp {
    /// Check whether the dock currently contains the given tab.
    pub(super) fn dock_has_tab(&self, tab: DockTab) -> bool {
        if let Some(ref ds) = self.dock_state {
            ds.iter_all_tabs().any(|(_, t)| *t == tab)
        } else {
            false
        }
    }

    /// Toggle a tab: add it if missing, remove it if present.
    pub(super) fn dock_toggle_tab(&mut self, tab: DockTab) {
        if self.dock_has_tab(tab) {
            self.dock_remove_tab(tab);
        } else {
            self.dock_add_tab(tab);
        }
    }

    /// Add a tab to the dock. Places it in the leaf matching the tab's region.
    pub(super) fn dock_add_tab(&mut self, tab: DockTab) {
        if self.dock_has_tab(tab) {
            return;
        }

        // Initialize script panel text when opening Script Editor
        if tab == DockTab::ScriptEditor {
            self.script_panel_text =
                crate::export::yarn_export::export_yarn(&self.graph);
            self.script_panel_dirty = false;
            self.script_panel_stale = false;
        }

        let Some(ref mut ds) = self.dock_state else {
            return;
        };
        let tree = ds.main_surface_mut();
        let target_region = tab.region();

        // Find a leaf that already contains a tab in the same region
        let target_node = tree.iter().enumerate().find_map(|(idx, node)| {
            if let egui_dock::Node::Leaf { tabs, .. } = node {
                if tabs.iter().any(|t| t.region() == target_region) {
                    return Some(NodeIndex(idx));
                }
            }
            None
        });

        if let Some(node_idx) = target_node {
            if let egui_dock::Node::Leaf { tabs, active, .. } =
                &mut tree[node_idx]
            {
                *active = egui_dock::TabIndex(tabs.len());
                tabs.push(tab);
            }
        } else {
            tree.push_to_first_leaf(tab);
        }
    }

    /// Remove all instances of the given tab from the dock.
    pub(super) fn dock_remove_tab(&mut self, tab: DockTab) {
        let Some(ref mut ds) = self.dock_state else {
            return;
        };
        // Collect (surface, node, tab_index) tuples to remove
        let mut to_remove = Vec::new();
        for ((surface_idx, node_idx), t) in ds.iter_all_tabs() {
            if *t == tab {
                let tree = &ds[surface_idx];
                if let egui_dock::Node::Leaf { tabs, .. } = &tree[node_idx] {
                    if let Some(pos) = tabs.iter().position(|t| *t == tab) {
                        to_remove.push((surface_idx, node_idx, egui_dock::TabIndex(pos)));
                    }
                }
            }
        }
        for (surface_idx, node_idx, tab_idx) in to_remove.into_iter().rev() {
            ds[surface_idx].remove_tab((node_idx, tab_idx));
        }
    }

    /// Set a tab as the active tab in its leaf (auto-focus).
    pub(super) fn focus_dock_tab(&mut self, tab: DockTab) {
        let Some(ref mut ds) = self.dock_state else {
            return;
        };
        let tree = &mut ds[SurfaceIndex::main()];
        if let Some((node_idx, tab_idx)) = tree.find_tab(&tab) {
            if let egui_dock::Node::Leaf { active, .. } = &mut tree[node_idx] {
                *active = tab_idx;
            }
        }
    }

    /// Show the dock area inside a CentralPanel. Uses Option::take() to
    /// avoid simultaneous mutable borrows of dock_state and self.
    pub(super) fn show_dock(&mut self, ctx: &egui::Context) {
        let mut dock_state = self
            .dock_state
            .take()
            .unwrap_or_else(default_dock_state);

        egui::CentralPanel::default()
            .frame(egui::Frame::NONE)
            .show(ctx, |ui| {
                let style = egui_dock::Style::from_egui(ui.style().as_ref());
                let mut viewer = AppTabViewer { app: self };
                DockArea::new(&mut dock_state)
                    .style(style)
                    .show_close_buttons(true)
                    .draggable_tabs(true)
                    .show_inside(ui, &mut viewer);
            });

        self.dock_state = Some(dock_state);
    }

    /// Reset dock layout to defaults.
    pub(super) fn dock_reset_layout(&mut self) {
        self.dock_state = Some(default_dock_state());
    }

    /// Check if script editor tab is open (for stale marking).
    pub(super) fn has_script_tab(&self) -> bool {
        self.dock_has_tab(DockTab::ScriptEditor)
    }

    /// If the Script Editor tab is open, populate its text from the current graph.
    pub(super) fn sync_script_if_open(&mut self) {
        if self.has_script_tab() {
            self.script_panel_text =
                crate::export::yarn_export::export_yarn(&self.graph);
            self.script_panel_dirty = false;
            self.script_panel_stale = false;
        }
    }

    /// Serialize dock state to JSON Value for project save.
    pub(super) fn dock_state_to_json(&self) -> Option<serde_json::Value> {
        self.dock_state
            .as_ref()
            .and_then(|ds| serde_json::to_value(ds).ok())
    }

    /// Restore dock state from JSON Value loaded from project.
    pub(super) fn dock_state_from_json(&mut self, value: &serde_json::Value) {
        if let Ok(ds) = serde_json::from_value::<DockState<DockTab>>(value.clone()) {
            self.dock_state = Some(ds);
        }
    }
}
