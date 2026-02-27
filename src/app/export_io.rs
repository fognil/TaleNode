use std::time::Instant;

use super::TaleNodeApp;

impl TaleNodeApp {
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

    pub(super) fn do_export_screenplay(&mut self) {
        let path = rfd::FileDialog::new()
            .add_filter("Text", &["txt"])
            .set_file_name(format!("{}_screenplay.txt", self.project_name))
            .save_file();

        if let Some(path) = path {
            let graph = self.root_graph_for_export();
            let text =
                crate::export::screenplay_export::export_screenplay(&graph, &self.project_name);
            if let Err(e) = std::fs::write(&path, text) {
                self.status_message =
                    Some((format!("Failed to write screenplay: {e}"), Instant::now(), true));
            } else {
                self.status_message =
                    Some(("Screenplay exported".to_string(), Instant::now(), false));
            }
        }
    }

    pub(super) fn do_export_html(&mut self) {
        let path = rfd::FileDialog::new()
            .add_filter("HTML", &["html"])
            .set_file_name(format!("{}.html", self.project_name))
            .save_file();

        if let Some(path) = path {
            let graph = self.root_graph_for_export();
            let html =
                crate::export::html_export::export_html(&graph, &self.project_name);
            if let Err(e) = std::fs::write(&path, html) {
                self.status_message =
                    Some((format!("Failed to write HTML: {e}"), Instant::now(), true));
            } else {
                self.status_message =
                    Some(("HTML playable exported".to_string(), Instant::now(), false));
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

    pub(super) fn do_export_bark_csv(&mut self) {
        let path = rfd::FileDialog::new()
            .add_filter("CSV", &["csv"])
            .set_file_name(format!("{}_barks.csv", self.project_name))
            .save_file();

        if let Some(path) = path {
            let graph = self.root_graph_for_export();
            let csv = crate::export::bark_export::export_bark_csv(&graph);
            if let Err(e) = std::fs::write(&path, csv) {
                self.status_message =
                    Some((format!("Failed to write bark CSV: {e}"), Instant::now(), true));
            } else {
                self.status_message =
                    Some(("Bark CSV exported".to_string(), Instant::now(), false));
            }
        }
    }

    pub(super) fn do_export_markdown(&mut self) {
        let path = rfd::FileDialog::new()
            .add_filter("Markdown", &["md"])
            .set_file_name(format!("{}.md", self.project_name))
            .save_file();

        if let Some(path) = path {
            let graph = self.root_graph_for_export();
            let md = crate::export::document_export::export_markdown(&graph, &self.project_name);
            if let Err(e) = std::fs::write(&path, md) {
                self.status_message =
                    Some((format!("Failed to write Markdown: {e}"), Instant::now(), true));
            } else {
                self.status_message =
                    Some(("Markdown exported".to_string(), Instant::now(), false));
            }
        }
    }

    pub(super) fn do_export_rtf(&mut self) {
        let path = rfd::FileDialog::new()
            .add_filter("RTF", &["rtf"])
            .set_file_name(format!("{}.rtf", self.project_name))
            .save_file();

        if let Some(path) = path {
            let graph = self.root_graph_for_export();
            let rtf = crate::export::document_export::export_rtf(&graph, &self.project_name);
            if let Err(e) = std::fs::write(&path, rtf) {
                self.status_message =
                    Some((format!("Failed to write RTF: {e}"), Instant::now(), true));
            } else {
                self.status_message =
                    Some(("RTF exported".to_string(), Instant::now(), false));
            }
        }
    }
}
