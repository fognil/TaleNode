use std::time::Instant;

use crate::model::{graph::DialogueGraph, node::Node};
use super::TaleNodeApp;

impl TaleNodeApp {
    pub(super) fn do_open(&mut self) {
        let path = rfd::FileDialog::new()
            .add_filter("TaleNode Project", &["talenode"])
            .pick_file();
        let Some(path) = path else { return };
        let contents = match std::fs::read_to_string(&path) {
            Ok(c) => c,
            Err(e) => {
                self.status_message =
                    Some((format!("Failed to read file: {e}"), Instant::now(), true));
                return;
            }
        };
        match crate::model::project::Project::load_from_string(&contents) {
            Ok(mut project) => {
                // Try loading versions from sidecar file
                let sidecar = path.with_extension("talenode.versions");
                if project.versions.is_empty() {
                    if let Ok(vdata) = std::fs::read_to_string(&sidecar) {
                        let _ = project.merge_versions(&vdata);
                    }
                }
                self.graph = project.graph;
                self.graph.rebuild_connection_index();
                self.project_name = project.name;
                self.versions = project.versions;
                self.project_path = Some(path);
                self.selected_nodes.clear();
                self.spatial_grid.mark_dirty();
                self.minimap_bounds_dirty = true;
                self.pending_zoom_fit = true;
                self.portrait_cache.clear();
                if let Some(ref layout) = project.dock_layout {
                    self.dock_state_from_json(layout);
                }
                self.sync_script_if_open();
            }
            Err(e) => {
                self.status_message =
                    Some((format!("Failed to parse project: {e}"), Instant::now(), true));
            }
        }
    }

    pub(super) fn do_save(&mut self, save_as: bool) {
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
                versions: self.versions.clone(),
                dock_layout: self.dock_state_to_json(),
            };
            match project.save_split() {
                Ok((main_json, versions_json)) => {
                    // Create .bak backup before overwriting
                    if path.exists() {
                        let bak = path.with_extension("talenode.bak");
                        let _ = std::fs::copy(&path, &bak);
                    }
                    if let Err(e) = std::fs::write(&path, main_json) {
                        self.status_message =
                            Some((format!("Failed to write file: {e}"), Instant::now(), true));
                    } else {
                        let sidecar = path.with_extension("talenode.versions");
                        if let Some(vj) = versions_json {
                            let _ = std::fs::write(&sidecar, vj);
                        } else {
                            let _ = std::fs::remove_file(&sidecar);
                        }
                        self.project_path = Some(path);
                        self.status_message =
                            Some(("Saved".to_string(), Instant::now(), false));
                        self.last_auto_save = Instant::now();
                    }
                }
                Err(e) => {
                    self.status_message =
                        Some((format!("Failed to serialize project: {e}"), Instant::now(), true));
                }
            }
        }
    }

    pub(super) fn do_new_project(&mut self) {
        self.graph = DialogueGraph::new();
        self.graph.add_node(Node::new_start([100.0, 200.0]));
        self.graph.rebuild_connection_index();
        self.selected_nodes.clear();
        self.spatial_grid.mark_dirty();
        self.minimap_bounds_dirty = true;
        self.pending_zoom_fit = true;
        self.project_name = "Untitled".to_string();
        self.project_path = None;
        self.versions.clear();
        self.history.clear();
        self.portrait_cache.clear();
        self.dock_reset_layout();
    }
}
