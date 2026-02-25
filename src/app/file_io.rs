use std::time::Instant;

use crate::model::graph::DialogueGraph;
use crate::model::node::Node;

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
            Ok(project) => {
                self.graph = project.graph;
                self.project_name = project.name;
                self.versions = project.versions;
                self.project_path = Some(path);
                self.selected_nodes.clear();
                if let Some(ref layout) = project.dock_layout {
                    self.dock_state_from_json(layout);
                }
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
            match project.save_to_string() {
                Ok(json) => {
                    if let Err(e) = std::fs::write(&path, json) {
                        self.status_message =
                            Some((format!("Failed to write file: {e}"), Instant::now(), true));
                    } else {
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

    pub(super) fn do_export_json(&mut self) {
        let path = rfd::FileDialog::new()
            .add_filter("JSON", &["json"])
            .set_file_name(format!("{}.json", self.project_name))
            .save_file();

        if let Some(path) = path {
            let graph = self.root_graph_for_export();
            match crate::export::json_export::export_json(&graph, &self.project_name) {
                Ok(json) => {
                    if let Err(e) = std::fs::write(&path, json) {
                        self.status_message =
                            Some((format!("Failed to write export: {e}"), Instant::now(), true));
                    } else {
                        self.status_message =
                            Some(("JSON exported".to_string(), Instant::now(), false));
                    }
                }
                Err(e) => {
                    self.status_message =
                        Some((format!("Failed to export JSON: {e}"), Instant::now(), true));
                }
            }
        }
    }

    pub(super) fn do_export_godot_plugin(&mut self) {
        let path = rfd::FileDialog::new()
            .set_title("Select Godot project folder")
            .pick_folder();

        if let Some(dir) = path {
            match crate::export::plugin_export::export_godot_plugin(&dir) {
                Ok(()) => {
                    self.write_dialogue_json(&dir);
                    self.status_message =
                        Some(("Godot plugin + JSON exported".to_string(), Instant::now(), false));
                }
                Err(e) => {
                    self.status_message =
                        Some((format!("Failed to export Godot plugin: {e}"), Instant::now(), true));
                }
            }
        }
    }

    pub(super) fn do_export_unity_plugin(&mut self) {
        let path = rfd::FileDialog::new()
            .set_title("Select Unity Assets folder")
            .pick_folder();

        if let Some(dir) = path {
            match crate::export::plugin_export::export_unity_plugin(&dir) {
                Ok(()) => {
                    self.write_dialogue_json(&dir);
                    self.status_message =
                        Some(("Unity plugin + JSON exported".to_string(), Instant::now(), false));
                }
                Err(e) => {
                    self.status_message =
                        Some((format!("Failed to export Unity plugin: {e}"), Instant::now(), true));
                }
            }
        }
    }

    pub(super) fn do_export_unreal_plugin(&mut self) {
        let path = rfd::FileDialog::new()
            .set_title("Select Unreal project Source folder")
            .pick_folder();

        if let Some(dir) = path {
            match crate::export::plugin_export::export_unreal_plugin(&dir) {
                Ok(()) => {
                    self.write_dialogue_json(&dir);
                    self.status_message =
                        Some(("Unreal plugin + JSON exported".to_string(), Instant::now(), false));
                }
                Err(e) => {
                    self.status_message = Some((
                        format!("Failed to export Unreal plugin: {e}"),
                        Instant::now(),
                        true,
                    ));
                }
            }
        }
    }

    fn write_dialogue_json(&mut self, dir: &std::path::Path) {
        let graph = self.root_graph_for_export();
        let filename = format!("{}.json", self.project_name);
        match crate::export::json_export::export_json(&graph, &self.project_name) {
            Ok(json) => {
                if let Err(e) = std::fs::write(dir.join(filename), json) {
                    self.status_message =
                        Some((format!("Failed to write dialogue JSON: {e}"), Instant::now(), true));
                }
            }
            Err(e) => {
                self.status_message =
                    Some((format!("Failed to serialize dialogue JSON: {e}"), Instant::now(), true));
            }
        }
    }

    pub(super) fn do_export_voice_csv(&mut self) {
        let path = rfd::FileDialog::new()
            .add_filter("CSV", &["csv"])
            .set_file_name(format!("{}_voice_script.csv", self.project_name))
            .save_file();

        if let Some(path) = path {
            let graph = self.root_graph_for_export();
            let csv = crate::export::voice_export::export_voice_csv(&graph, &self.project_name);
            if let Err(e) = std::fs::write(&path, csv) {
                self.status_message =
                    Some((format!("Failed to write voice script: {e}"), Instant::now(), true));
            } else {
                self.status_message =
                    Some(("Voice script exported".to_string(), Instant::now(), false));
            }
        }
    }

    pub(super) fn do_export_xml(&mut self) {
        let path = rfd::FileDialog::new()
            .add_filter("XML", &["xml"])
            .set_file_name(format!("{}.xml", self.project_name))
            .save_file();

        if let Some(path) = path {
            let graph = self.root_graph_for_export();
            match crate::export::xml_export::export_xml(&graph, &self.project_name) {
                Ok(xml) => {
                    if let Err(e) = std::fs::write(&path, xml) {
                        self.status_message =
                            Some((format!("Failed to write XML: {e}"), Instant::now(), true));
                    } else {
                        self.status_message =
                            Some(("XML exported".to_string(), Instant::now(), false));
                    }
                }
                Err(e) => {
                    self.status_message =
                        Some((format!("Failed to export XML: {e}"), Instant::now(), true));
                }
            }
        }
    }

    pub(super) fn do_import_yarn(&mut self) {
        let Some(path) = rfd::FileDialog::new()
            .add_filter("Yarn Spinner", &["yarn"])
            .pick_file() else { return };
        let contents = match std::fs::read_to_string(&path) {
            Ok(c) => c,
            Err(e) => {
                self.status_message =
                    Some((format!("Failed to read file: {e}"), Instant::now(), true));
                return;
            }
        };
        match crate::import::yarn_import::import_yarn(&contents) {
            Ok(graph) => {
                self.graph = graph;
                self.selected_nodes.clear();
                self.project_name = path
                    .file_stem()
                    .map(|s| s.to_string_lossy().to_string())
                    .unwrap_or_else(|| "Imported".to_string());
                self.project_path = None;
                self.history.clear();
                self.status_message =
                    Some(("Imported from Yarn".to_string(), Instant::now(), false));
            }
            Err(e) => {
                self.status_message =
                    Some((format!("Failed to import Yarn: {e}"), Instant::now(), true));
            }
        }
    }

    pub(super) fn do_import_chatmapper(&mut self) {
        let file = rfd::FileDialog::new()
            .add_filter("Chat Mapper", &["cmp", "xml"])
            .pick_file();
        if let Some(path) = file {
            match std::fs::read_to_string(&path) {
                Ok(contents) => {
                    match crate::import::chatmapper_import::import_chatmapper(&contents) {
                        Ok(graph) => {
                            self.graph = graph;
                            self.selected_nodes.clear();
                            self.project_name = path
                                .file_stem()
                                .map(|s| s.to_string_lossy().to_string())
                                .unwrap_or_else(|| "Imported".to_string());
                            self.project_path = None;
                            self.history.clear();
                            self.status_message = Some((
                                "Imported from Chat Mapper".to_string(),
                                Instant::now(),
                                false,
                            ));
                        }
                        Err(e) => {
                            self.status_message = Some((
                                format!("Failed to import Chat Mapper: {e}"),
                                Instant::now(),
                                true,
                            ));
                        }
                    }
                }
                Err(e) => {
                    self.status_message =
                        Some((format!("Failed to read file: {e}"), Instant::now(), true));
                }
            }
        }
    }

    pub(super) fn do_import_articy(&mut self) {
        let file = rfd::FileDialog::new()
            .add_filter("articy:draft XML", &["xml"])
            .pick_file();
        if let Some(path) = file {
            match std::fs::read_to_string(&path) {
                Ok(contents) => {
                    match crate::import::articy_import::import_articy(&contents) {
                        Ok(graph) => {
                            self.graph = graph;
                            self.selected_nodes.clear();
                            self.project_name = path
                                .file_stem()
                                .map(|s| s.to_string_lossy().to_string())
                                .unwrap_or_else(|| "Imported".to_string());
                            self.project_path = None;
                            self.history.clear();
                            self.status_message =
                                Some(("Imported from articy".to_string(), Instant::now(), false));
                        }
                        Err(e) => {
                            self.status_message = Some((
                                format!("Failed to import articy: {e}"),
                                Instant::now(),
                                true,
                            ));
                        }
                    }
                }
                Err(e) => {
                    self.status_message =
                        Some((format!("Failed to read file: {e}"), Instant::now(), true));
                }
            }
        }
    }

    pub(super) fn do_export_analytics_csv(
        &mut self,
        stats: &crate::validation::analytics::GraphAnalytics,
    ) {
        let path = rfd::FileDialog::new()
            .add_filter("CSV", &["csv"])
            .set_file_name(format!("{}_analytics.csv", self.project_name))
            .save_file();

        if let Some(path) = path {
            let csv = crate::export::analytics_export::export_analytics_csv(
                stats,
                &self.project_name,
            );
            if let Err(e) = std::fs::write(&path, csv) {
                self.status_message =
                    Some((format!("Failed to write analytics CSV: {e}"), Instant::now(), true));
            } else {
                self.status_message =
                    Some(("Analytics CSV exported".to_string(), Instant::now(), false));
            }
        }
    }

    pub(super) fn do_export_analytics_text(
        &mut self,
        stats: &crate::validation::analytics::GraphAnalytics,
    ) {
        let path = rfd::FileDialog::new()
            .add_filter("Text", &["txt"])
            .set_file_name(format!("{}_analytics.txt", self.project_name))
            .save_file();

        if let Some(path) = path {
            let text = crate::export::analytics_export::export_analytics_text(
                stats,
                &self.project_name,
            );
            if let Err(e) = std::fs::write(&path, text) {
                self.status_message = Some((
                    format!("Failed to write analytics report: {e}"),
                    Instant::now(),
                    true,
                ));
            } else {
                self.status_message =
                    Some(("Analytics report exported".to_string(), Instant::now(), false));
            }
        }
    }

    pub(super) fn do_new_project(&mut self) {
        self.graph = DialogueGraph::new();
        self.graph.add_node(Node::new_start([100.0, 200.0]));
        self.selected_nodes.clear();
        self.project_name = "Untitled".to_string();
        self.project_path = None;
        self.versions.clear();
        self.history.clear();
        self.dock_reset_layout();
    }

}
