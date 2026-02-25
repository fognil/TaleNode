use super::TaleNodeApp;

impl TaleNodeApp {
    pub(super) fn render_left_panel_tab(&mut self, ui: &mut egui::Ui) {
        let pre_graph = self.graph.clone();
        egui::ScrollArea::vertical().show(ui, |ui| {
            let changed =
                crate::ui::left_panel::show_left_panel(ui, &mut self.graph);
            if changed {
                self.history.push_undo(pre_graph);
            }
        });
    }

    pub(super) fn render_inspector_tab(&mut self, ui: &mut egui::Ui) {
        if self.selected_nodes.len() != 1 {
            ui.centered_and_justified(|ui| {
                ui.label("Select a single node to inspect");
            });
            return;
        }
        let selected_id = self.selected_nodes[0];
        let pre_graph = self.graph.clone();
        egui::ScrollArea::vertical().show(ui, |ui| {
            let changed = crate::ui::inspector::show_inspector(
                ui,
                &mut self.graph,
                selected_id,
                &mut self.inspector_new_tag_text,
                &mut self.active_locale,
            );
            if changed {
                self.history.push_undo(pre_graph);
            }
        });
    }

    pub(super) fn render_script_tab(&mut self, ui: &mut egui::Ui) {
        let action = crate::ui::script_panel::show_script_panel(
            ui,
            &mut self.script_panel_text,
            self.script_panel_dirty,
            self.script_panel_stale,
        );
        match action {
            crate::ui::script_panel::ScriptPanelAction::Commit(text) => {
                match crate::import::yarn_import::import_yarn(&text) {
                    Ok(new_graph) => {
                        self.snapshot();
                        self.graph = new_graph;
                        self.selected_nodes.clear();
                        self.script_panel_dirty = false;
                        self.script_panel_stale = false;
                        self.status_message = Some((
                            "Script committed to graph".to_string(),
                            std::time::Instant::now(),
                            false,
                        ));
                    }
                    Err(e) => {
                        self.status_message = Some((
                            format!("Script error: {e}"),
                            std::time::Instant::now(),
                            true,
                        ));
                    }
                }
            }
            crate::ui::script_panel::ScriptPanelAction::Refresh => {
                self.script_panel_text =
                    crate::export::yarn_export::export_yarn(&self.graph);
                self.script_panel_dirty = false;
                self.script_panel_stale = false;
            }
            crate::ui::script_panel::ScriptPanelAction::TextChanged => {
                self.script_panel_dirty = true;
            }
            crate::ui::script_panel::ScriptPanelAction::None => {}
        }
    }

    pub(super) fn render_comments_tab(&mut self, ui: &mut egui::Ui) {
        if let Some(first) = self.selected_nodes.first() {
            self.comment_target_node = Some(*first);
        }
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
    }

    pub(super) fn render_bookmarks_tab(&mut self, ui: &mut egui::Ui) {
        let sel = self.selected_nodes.first().copied();
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
    }

    pub(super) fn render_analytics_tab(&mut self, ui: &mut egui::Ui) {
        let stats = crate::validation::analytics::analyze_graph(&self.graph);
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
    }

    pub(super) fn render_version_tab(&mut self, ui: &mut egui::Ui) {
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
                    dock_layout: None,
                };
                project.create_version(desc);
                self.versions = project.versions;
            }
            crate::ui::version_panel::VersionPanelAction::RestoreVersion(id) => {
                self.pending_confirmation =
                    Some(super::confirm::PendingAction::RestoreVersion(id));
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
    }

    pub(super) fn render_templates_tab(&mut self, ui: &mut egui::Ui) {
        let has_selection = !self.selected_nodes.is_empty();
        let action = crate::ui::template_panel::show_template_panel(
            ui,
            &self.template_library,
            &mut self.template_new_name,
            has_selection,
        );
        match action {
            crate::ui::template_panel::TemplatePanelAction::Insert(tid) => {
                if let Some(t) = self
                    .template_library
                    .templates
                    .iter()
                    .find(|t| t.id == tid)
                    .cloned()
                {
                    let center = [
                        -self.canvas.pan_offset.x / self.canvas.zoom,
                        -self.canvas.pan_offset.y / self.canvas.zoom,
                    ];
                    self.insert_template(&t, center);
                }
            }
            crate::ui::template_panel::TemplatePanelAction::Delete(tid) => {
                self.delete_template(tid);
            }
            crate::ui::template_panel::TemplatePanelAction::SaveSelection(name) => {
                self.save_selection_as_template(name);
            }
            crate::ui::template_panel::TemplatePanelAction::None => {}
        }
    }

    pub(super) fn render_locale_tab(&mut self, ui: &mut egui::Ui) {
        let action = crate::ui::locale_panel::show_locale_panel(
            ui,
            &self.graph,
            &mut self.active_locale,
            &mut self.locale_filter_untranslated,
            &mut self.locale_new_name,
        );
        match action {
            crate::ui::locale_panel::LocalePanelAction::AddLocale(name) => {
                self.snapshot();
                self.graph.locale.add_locale(name);
            }
            crate::ui::locale_panel::LocalePanelAction::RemoveLocale(name) => {
                self.snapshot();
                self.graph.locale.remove_locale(&name);
                if self.active_locale.as_deref() == Some(name.as_str()) {
                    self.active_locale = None;
                }
            }
            crate::ui::locale_panel::LocalePanelAction::SetTranslation {
                key,
                locale,
                text,
            } => {
                self.snapshot();
                self.graph.locale.set_translation(key, locale, text);
            }
            crate::ui::locale_panel::LocalePanelAction::Navigate(node_id) => {
                self.selected_nodes = vec![node_id];
                if let Some(node) = self.graph.nodes.get(&node_id) {
                    self.canvas.pan_offset = egui::Vec2::new(
                        -node.position[0] * self.canvas.zoom,
                        -node.position[1] * self.canvas.zoom,
                    );
                }
            }
            crate::ui::locale_panel::LocalePanelAction::ExportCsv => {
                self.do_export_locale_csv();
            }
            crate::ui::locale_panel::LocalePanelAction::ImportCsv => {
                self.do_import_locale_csv();
            }
            crate::ui::locale_panel::LocalePanelAction::None => {}
        }
    }

    pub(super) fn render_playtest_tab(&mut self, ui: &mut egui::Ui) {
        crate::ui::playtest_panel::show_playtest_panel(
            ui,
            &mut self.playtest,
            &self.graph,
            &mut self.selected_nodes,
        );
    }

    pub(super) fn render_validation_tab(&mut self, ui: &mut egui::Ui) {
        self.show_validation_panel(ui);
    }
}
