use super::{confirm, CanvasFilter, TaleNodeApp};

impl TaleNodeApp {
    pub(super) fn handle_keyboard_shortcuts(&mut self, ctx: &egui::Context) {
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
                self.spatial_grid.mark_dirty();
                self.validation_dirty = true;
            }
        }
        if ctx.input(|i| {
            i.modifiers.command && i.modifiers.shift && i.key_pressed(egui::Key::Z)
        }) {
            if let Some(next) = self.history.redo(&self.graph) {
                self.graph = next;
                self.selected_nodes.clear();
                self.spatial_grid.mark_dirty();
                self.validation_dirty = true;
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
        if ctx.input(|i| {
            i.modifiers.command && i.modifiers.shift && i.key_pressed(egui::Key::F)
        }) {
            self.canvas_filter.active = !self.canvas_filter.active;
            if self.canvas_filter.active && self.canvas_filter.visible_cache.is_empty() {
                self.canvas_filter.visible_cache = crate::ui::filter_bar::compute_visible_nodes(
                    &self.graph, &self.canvas_filter.tags, &self.canvas_filter.node_types,
                );
            }
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
                self.search_results_set.clear();
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

    pub(super) fn show_filter_bar(&mut self, ui: &mut egui::Ui) {
        let action = crate::ui::filter_bar::show_filter_bar(
            ui, &self.graph,
            &mut self.canvas_filter.tags,
            &mut self.canvas_filter.node_types,
            self.canvas_filter.active,
        );
        match action {
            crate::ui::filter_bar::FilterAction::Changed => {
                self.canvas_filter.visible_cache = crate::ui::filter_bar::compute_visible_nodes(
                    &self.graph, &self.canvas_filter.tags, &self.canvas_filter.node_types,
                );
            }
            crate::ui::filter_bar::FilterAction::Clear => {
                self.canvas_filter = CanvasFilter::default();
            }
            crate::ui::filter_bar::FilterAction::None => {}
        }
    }
}
