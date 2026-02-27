use std::path::PathBuf;

use super::TaleNodeApp;
use crate::model::plugin;

impl TaleNodeApp {
    pub(super) fn render_extensions_tab(&mut self, ui: &mut egui::Ui) {
        egui::ScrollArea::vertical().show(ui, |ui| {
            let action = crate::ui::extension_panel::show_extension_panel(
                ui,
                &self.discovered_plugins,
                &self.plugin_last_result,
            );
            match action {
                crate::ui::extension_panel::ExtensionPanelAction::RefreshPlugins => {
                    self.refresh_plugins();
                }
                crate::ui::extension_panel::ExtensionPanelAction::RunExport(idx) => {
                    self.run_plugin_export(idx);
                }
                crate::ui::extension_panel::ExtensionPanelAction::RunImport(idx) => {
                    self.run_plugin_import(idx);
                }
                crate::ui::extension_panel::ExtensionPanelAction::OpenPluginsFolder => {
                    let global_dir = plugin_dirs()[0].clone();
                    let _ = std::fs::create_dir_all(&global_dir);
                    self.plugin_last_result = Some((
                        format!("Plugins folder: {}", global_dir.display()),
                        false,
                    ));
                }
                crate::ui::extension_panel::ExtensionPanelAction::None => {}
            }
        });
    }

    pub(super) fn refresh_plugins(&mut self) {
        self.discovered_plugins = plugin::discover_plugins(&plugin_dirs());
        self.plugin_last_result = Some((
            format!("Found {} plugin(s)", self.discovered_plugins.len()),
            false,
        ));
    }

    fn run_plugin_export(&mut self, idx: usize) {
        let Some(p) = self.discovered_plugins.get(idx) else { return };
        let p = p.clone();
        match crate::export::plugin_api::run_export_plugin(
            &p,
            &self.graph,
            &self.project_name,
        ) {
            Ok(content) => {
                if let Some(path) = rfd::FileDialog::new()
                    .set_file_name(format!("{}_export.txt", p.id))
                    .save_file()
                {
                    match std::fs::write(&path, &content) {
                        Ok(()) => {
                            self.plugin_last_result = Some((
                                format!("Exported to {}", path.display()),
                                false,
                            ));
                        }
                        Err(e) => {
                            self.plugin_last_result =
                                Some((format!("Write error: {e}"), true));
                        }
                    }
                }
            }
            Err(e) => {
                self.plugin_last_result = Some((format!("Export error: {e}"), true));
            }
        }
    }

    fn run_plugin_import(&mut self, idx: usize) {
        let Some(p) = self.discovered_plugins.get(idx) else { return };
        let p = p.clone();
        let Some(path) = rfd::FileDialog::new().pick_file() else { return };
        let Ok(content) = std::fs::read_to_string(&path) else {
            self.plugin_last_result = Some(("Cannot read file".to_string(), true));
            return;
        };
        match crate::export::plugin_api::run_import_plugin(&p, &content) {
            Ok(new_graph) => {
                self.snapshot();
                self.graph = new_graph;
                self.graph.rebuild_connection_index();
                self.plugin_last_result = Some((
                    format!("Imported from {}", path.display()),
                    false,
                ));
            }
            Err(e) => {
                self.plugin_last_result = Some((format!("Import error: {e}"), true));
            }
        }
    }
}

fn plugin_dirs() -> Vec<PathBuf> {
    let mut dirs = Vec::new();
    if let Some(config) = dirs::config_dir() {
        dirs.push(config.join("talenode").join("plugins"));
    }
    dirs.push(PathBuf::from("plugins"));
    dirs
}
