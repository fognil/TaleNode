use std::time::Instant;

use super::TaleNodeApp;

impl TaleNodeApp {
    pub(super) fn do_import_yarn(&mut self) {
        let Some(path) = rfd::FileDialog::new()
            .add_filter("Yarn Spinner", &["yarn"])
            .pick_file()
        else {
            return;
        };
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

    pub(super) fn do_import_ink(&mut self) {
        let Some(path) = rfd::FileDialog::new()
            .add_filter("Ink", &["ink"])
            .pick_file()
        else {
            return;
        };
        let contents = match std::fs::read_to_string(&path) {
            Ok(c) => c,
            Err(e) => {
                self.status_message =
                    Some((format!("Failed to read file: {e}"), Instant::now(), true));
                return;
            }
        };
        match crate::import::ink_import::import_ink(&contents) {
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
                    Some(("Imported from Ink".to_string(), Instant::now(), false));
            }
            Err(e) => {
                self.status_message =
                    Some((format!("Failed to import Ink: {e}"), Instant::now(), true));
            }
        }
    }
}
