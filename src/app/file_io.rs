use std::time::Instant;

use crate::model::graph::DialogueGraph;
use crate::model::node::Node;

use super::TaleNodeApp;

impl TaleNodeApp {
    pub(super) fn do_open(&mut self) {
        let file = rfd::FileDialog::new()
            .add_filter("TaleNode Project", &["talenode"])
            .pick_file();
        if let Some(path) = file {
            match std::fs::read_to_string(&path) {
                Ok(contents) => {
                    match crate::model::project::Project::load_from_string(&contents) {
                        Ok(project) => {
                            self.graph = project.graph;
                            self.project_name = project.name;
                            self.versions = project.versions;
                            self.project_path = Some(path);
                            self.selected_nodes.clear();
                        }
                        Err(e) => {
                            eprintln!("Failed to parse project: {e}");
                        }
                    }
                }
                Err(e) => {
                    eprintln!("Failed to read file: {e}");
                }
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
            };
            match project.save_to_string() {
                Ok(json) => {
                    if let Err(e) = std::fs::write(&path, json) {
                        eprintln!("Failed to write file: {e}");
                    } else {
                        self.project_path = Some(path);
                        self.status_message = Some(("Saved".to_string(), Instant::now()));
                        self.last_auto_save = Instant::now();
                    }
                }
                Err(e) => {
                    eprintln!("Failed to serialize project: {e}");
                }
            }
        }
    }

    pub(super) fn do_export_json(&self) {
        let path = rfd::FileDialog::new()
            .add_filter("JSON", &["json"])
            .set_file_name(format!("{}.json", self.project_name))
            .save_file();

        if let Some(path) = path {
            match crate::export::json_export::export_json(&self.graph, &self.project_name) {
                Ok(json) => {
                    if let Err(e) = std::fs::write(&path, json) {
                        eprintln!("Failed to write export: {e}");
                    }
                }
                Err(e) => {
                    eprintln!("Failed to export JSON: {e}");
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
                    self.status_message =
                        Some(("Godot plugin exported".to_string(), Instant::now()));
                }
                Err(e) => {
                    eprintln!("Failed to export Godot plugin: {e}");
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
                    self.status_message =
                        Some(("Unity plugin exported".to_string(), Instant::now()));
                }
                Err(e) => {
                    eprintln!("Failed to export Unity plugin: {e}");
                }
            }
        }
    }

    pub(super) fn do_export_voice_csv(&mut self) {
        let path = rfd::FileDialog::new()
            .add_filter("CSV", &["csv"])
            .set_file_name(format!("{}_voice_script.csv", self.project_name))
            .save_file();

        if let Some(path) = path {
            let csv = crate::export::voice_export::export_voice_csv(
                &self.graph,
                &self.project_name,
            );
            if let Err(e) = std::fs::write(&path, csv) {
                eprintln!("Failed to write voice script: {e}");
            } else {
                self.status_message =
                    Some(("Voice script exported".to_string(), Instant::now()));
            }
        }
    }

    pub(super) fn do_export_xml(&mut self) {
        let path = rfd::FileDialog::new()
            .add_filter("XML", &["xml"])
            .set_file_name(format!("{}.xml", self.project_name))
            .save_file();

        if let Some(path) = path {
            match crate::export::xml_export::export_xml(&self.graph, &self.project_name) {
                Ok(xml) => {
                    if let Err(e) = std::fs::write(&path, xml) {
                        eprintln!("Failed to write XML export: {e}");
                    } else {
                        self.status_message =
                            Some(("XML exported".to_string(), Instant::now()));
                    }
                }
                Err(e) => {
                    eprintln!("Failed to export XML: {e}");
                }
            }
        }
    }

    pub(super) fn do_import_yarn(&mut self) {
        let file = rfd::FileDialog::new()
            .add_filter("Yarn Spinner", &["yarn"])
            .pick_file();
        if let Some(path) = file {
            match std::fs::read_to_string(&path) {
                Ok(contents) => {
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
                                Some(("Imported from Yarn".to_string(), Instant::now()));
                        }
                        Err(e) => {
                            eprintln!("Failed to import Yarn: {e}");
                        }
                    }
                }
                Err(e) => {
                    eprintln!("Failed to read file: {e}");
                }
            }
        }
    }

    /// Handle New Project: reset graph, clear state.
    pub(super) fn do_new_project(&mut self) {
        self.graph = DialogueGraph::new();
        self.graph.add_node(Node::new_start([100.0, 200.0]));
        self.selected_nodes.clear();
        self.project_name = "Untitled".to_string();
        self.project_path = None;
        self.versions.clear();
        self.history.clear();
    }
}
