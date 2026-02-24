use super::TaleNodeApp;

impl TaleNodeApp {
    /// Render bottom panels (comments, bookmarks, analytics, version) and handle their actions.
    pub(super) fn show_bottom_panels(&mut self, ctx: &egui::Context) {
        self.show_comments_bottom_panel(ctx);
        self.show_bookmark_bottom_panel(ctx);
        self.show_analytics_bottom_panel(ctx);
        self.show_version_bottom_panel(ctx);
    }

    fn show_comments_bottom_panel(&mut self, ctx: &egui::Context) {
        if !self.show_comments_panel {
            return;
        }
        if let Some(first) = self.selected_nodes.first() {
            self.comment_target_node = Some(*first);
        }
        egui::TopBottomPanel::bottom("comments_panel")
            .resizable(true)
            .default_height(180.0)
            .show(ctx, |ui| {
                let action = crate::ui::comments_panel::show_comments_panel(
                    ui,
                    &self.graph,
                    &mut self.comments_filter,
                    &mut self.comment_target_node,
                    &mut self.new_comment_text,
                );
                match action {
                    crate::ui::comments_panel::CommentsPanelAction::Navigate(node_id) => {
                        self.selected_nodes = vec![node_id];
                        if let Some(node) = self.graph.nodes.get(&node_id) {
                            self.canvas.pan_offset = egui::Vec2::new(
                                -node.position[0] * self.canvas.zoom,
                                -node.position[1] * self.canvas.zoom,
                            );
                        }
                    }
                    crate::ui::comments_panel::CommentsPanelAction::AddComment(
                        node_id,
                        text,
                    ) => {
                        self.snapshot();
                        let comment =
                            crate::model::review::NodeComment::new(node_id, text);
                        self.graph.comments.push(comment);
                    }
                    crate::ui::comments_panel::CommentsPanelAction::DeleteComment(
                        comment_id,
                    ) => {
                        self.snapshot();
                        self.graph.comments.retain(|c| c.id != comment_id);
                    }
                    crate::ui::comments_panel::CommentsPanelAction::None => {}
                }
            });
    }

    fn show_bookmark_bottom_panel(&mut self, ctx: &egui::Context) {
        if !self.show_bookmark_panel {
            return;
        }
        let sel = self.selected_nodes.first().copied();
        egui::TopBottomPanel::bottom("bookmark_panel")
            .resizable(true)
            .default_height(180.0)
            .show(ctx, |ui| {
                let action = crate::ui::bookmark_panel::show_bookmark_panel(
                    ui,
                    &self.graph,
                    &mut self.bookmark_tag_filter,
                    &mut self.bookmark_new_tag_text,
                    sel,
                );
                match action {
                    crate::ui::bookmark_panel::BookmarkAction::Navigate(id) => {
                        self.selected_nodes = vec![id];
                        if let Some(node) = self.graph.nodes.get(&id) {
                            self.canvas.pan_offset = egui::Vec2::new(
                                -node.position[0] * self.canvas.zoom,
                                -node.position[1] * self.canvas.zoom,
                            );
                        }
                    }
                    crate::ui::bookmark_panel::BookmarkAction::AddTag(id, tag) => {
                        self.snapshot();
                        self.graph.add_tag(id, tag);
                    }
                    crate::ui::bookmark_panel::BookmarkAction::RemoveTag(id, tag) => {
                        self.snapshot();
                        self.graph.remove_tag(id, &tag);
                    }
                    crate::ui::bookmark_panel::BookmarkAction::None => {}
                }
            });
    }

    fn show_analytics_bottom_panel(&mut self, ctx: &egui::Context) {
        if !self.show_analytics_panel {
            return;
        }
        let stats = crate::validation::analytics::analyze_graph(&self.graph);
        egui::TopBottomPanel::bottom("analytics_panel")
            .resizable(true)
            .default_height(200.0)
            .show(ctx, |ui| {
                let action = egui::ScrollArea::vertical()
                    .show(ui, |ui| {
                        crate::ui::analytics_panel::show_analytics_panel(ui, &stats)
                    })
                    .inner;
                match action {
                    crate::ui::analytics_panel::AnalyticsPanelAction::ExportCsv => {
                        self.do_export_analytics_csv(&stats);
                    }
                    crate::ui::analytics_panel::AnalyticsPanelAction::ExportText => {
                        self.do_export_analytics_text(&stats);
                    }
                    crate::ui::analytics_panel::AnalyticsPanelAction::None => {}
                }
            });
    }

    fn show_version_bottom_panel(&mut self, ctx: &egui::Context) {
        if !self.show_version_panel {
            return;
        }
        egui::TopBottomPanel::bottom("version_panel")
            .resizable(true)
            .default_height(180.0)
            .show(ctx, |ui| {
                let action = crate::ui::version_panel::show_version_panel(
                    ui,
                    &self.versions,
                    &mut self.version_new_desc,
                    &mut self.version_compare_selection,
                    self.version_diff_result.as_ref(),
                );
                match action {
                    crate::ui::version_panel::VersionPanelAction::CreateVersion(desc) => {
                        let mut project = crate::model::project::Project {
                            version: "1.0".to_string(),
                            name: self.project_name.clone(),
                            graph: self.graph.clone(),
                            versions: self.versions.clone(),
                        };
                        project.create_version(desc);
                        self.versions = project.versions;
                    }
                    crate::ui::version_panel::VersionPanelAction::RestoreVersion(id) => {
                        let mut project = crate::model::project::Project {
                            version: "1.0".to_string(),
                            name: self.project_name.clone(),
                            graph: self.graph.clone(),
                            versions: self.versions.clone(),
                        };
                        if let Some(old_graph) = project.restore_version(id) {
                            self.snapshot();
                            let _ = old_graph;
                            self.graph = project.graph;
                        }
                    }
                    crate::ui::version_panel::VersionPanelAction::CompareVersions(a, b) => {
                        let va = self.versions.iter().find(|v| v.id == a);
                        let vb = self.versions.iter().find(|v| v.id == b);
                        if let (Some(va), Some(vb)) = (va, vb) {
                            self.version_diff_result = Some(
                                crate::model::graph_diff::diff_graphs(&va.graph, &vb.graph),
                            );
                        }
                    }
                    crate::ui::version_panel::VersionPanelAction::None => {}
                }
            });
    }
}
