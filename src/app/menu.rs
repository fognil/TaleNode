use crate::model::graph::DialogueGraph;
use crate::model::node::Node;

use super::TaleNodeApp;

impl TaleNodeApp {
    pub(super) fn show_menu_bar(&mut self, ui: &mut egui::Ui) {
        let mod_key = if cfg!(target_os = "macos") {
            "Cmd"
        } else {
            "Ctrl"
        };

        egui::menu::bar(ui, |ui| {
            ui.menu_button("File", |ui| {
                if ui
                    .add(egui::Button::new("New").shortcut_text(format!("{mod_key}+N")))
                    .clicked()
                {
                    self.graph = DialogueGraph::new();
                    self.graph.add_node(Node::new_start([100.0, 200.0]));
                    self.selected_nodes.clear();
                    self.project_name = "Untitled".to_string();
                    self.project_path = None;
                    ui.close_menu();
                }
                if ui
                    .add(egui::Button::new("Open...").shortcut_text(format!("{mod_key}+O")))
                    .clicked()
                {
                    self.do_open();
                    ui.close_menu();
                }
                if ui
                    .add(egui::Button::new("Save").shortcut_text(format!("{mod_key}+S")))
                    .clicked()
                {
                    self.do_save(false);
                    ui.close_menu();
                }
                if ui.button("Save As...").clicked() {
                    self.do_save(true);
                    ui.close_menu();
                }
                ui.separator();
                if ui.button("Export JSON...").clicked() {
                    self.do_export_json();
                    ui.close_menu();
                }
                if ui.button("Export XML...").clicked() {
                    self.do_export_xml();
                    ui.close_menu();
                }
                if ui.button("Export Godot Plugin...").clicked() {
                    self.do_export_godot_plugin();
                    ui.close_menu();
                }
                if ui.button("Export Unity Plugin...").clicked() {
                    self.do_export_unity_plugin();
                    ui.close_menu();
                }
                if ui.button("Export Unreal Plugin...").clicked() {
                    self.do_export_unreal_plugin();
                    ui.close_menu();
                }
                ui.separator();
                if ui.button("Export Voice Script (CSV)...").clicked() {
                    self.do_export_voice_csv();
                    ui.close_menu();
                }
                if ui.button("Batch Assign Audio...").clicked() {
                    self.audio_manager.open = true;
                    ui.close_menu();
                }
                ui.separator();
                if ui.button("Import from Yarn...").clicked() {
                    self.do_import_yarn();
                    ui.close_menu();
                }
                if ui.button("Import from Chat Mapper...").clicked() {
                    self.do_import_chatmapper();
                    ui.close_menu();
                }
                if ui.button("Import from articy...").clicked() {
                    self.do_import_articy();
                    ui.close_menu();
                }
            });
            ui.menu_button("Edit", |ui| {
                if ui
                    .add_enabled(
                        self.history.can_undo(),
                        egui::Button::new("Undo").shortcut_text(format!("{mod_key}+Z")),
                    )
                    .clicked()
                {
                    if let Some(prev) = self.history.undo(&self.graph) {
                        self.graph = prev;
                        self.selected_nodes.clear();
                    }
                    ui.close_menu();
                }
                if ui
                    .add_enabled(
                        self.history.can_redo(),
                        egui::Button::new("Redo")
                            .shortcut_text(format!("{mod_key}+Shift+Z")),
                    )
                    .clicked()
                {
                    if let Some(next) = self.history.redo(&self.graph) {
                        self.graph = next;
                        self.selected_nodes.clear();
                    }
                    ui.close_menu();
                }
                ui.separator();
                if ui
                    .add(
                        egui::Button::new("Select All")
                            .shortcut_text(format!("{mod_key}+A")),
                    )
                    .clicked()
                {
                    self.selected_nodes = self.graph.nodes.keys().copied().collect();
                    ui.close_menu();
                }
                if ui
                    .add_enabled(
                        !self.selected_nodes.is_empty(),
                        egui::Button::new("Duplicate")
                            .shortcut_text(format!("{mod_key}+D")),
                    )
                    .clicked()
                {
                    self.duplicate_selected();
                    ui.close_menu();
                }
                if ui
                    .add_enabled(
                        !self.selected_nodes.is_empty(),
                        egui::Button::new("Delete Selected").shortcut_text("Del"),
                    )
                    .clicked()
                {
                    self.snapshot();
                    let ids: Vec<uuid::Uuid> = self.selected_nodes.drain(..).collect();
                    for id in ids {
                        self.graph.remove_node(id);
                    }
                    ui.close_menu();
                }
                ui.separator();
                let find_shortcut = format!("{mod_key}+F");
                if ui
                    .add(egui::Button::new("Find...").shortcut_text(&find_shortcut))
                    .clicked()
                {
                    self.show_search = true;
                    self.show_replace = false;
                    ui.close_menu();
                }
                let replace_shortcut = if cfg!(target_os = "macos") {
                    format!("{mod_key}+Shift+H")
                } else {
                    format!("{mod_key}+H")
                };
                if ui
                    .add(
                        egui::Button::new("Find & Replace...")
                            .shortcut_text(&replace_shortcut),
                    )
                    .clicked()
                {
                    self.show_search = true;
                    self.show_replace = true;
                    ui.close_menu();
                }
            });
            ui.menu_button("View", |ui| {
                if ui
                    .checkbox(&mut self.show_left_panel, "Left Panel")
                    .changed()
                {
                    ui.close_menu();
                }
                if ui
                    .checkbox(&mut self.show_validation_panel, "Validation Panel")
                    .changed()
                {
                    ui.close_menu();
                }
                if ui
                    .checkbox(&mut self.show_playtest, "Playtest Panel")
                    .changed()
                {
                    ui.close_menu();
                }
                if ui
                    .checkbox(&mut self.show_comments_panel, "Comments Panel")
                    .changed()
                {
                    ui.close_menu();
                }
                if ui
                    .checkbox(&mut self.show_bookmark_panel, "Bookmark Panel")
                    .changed()
                {
                    ui.close_menu();
                }
                if ui
                    .checkbox(&mut self.show_analytics_panel, "Analytics Panel")
                    .changed()
                {
                    ui.close_menu();
                }
                if ui
                    .checkbox(&mut self.show_version_panel, "Version History")
                    .changed()
                {
                    ui.close_menu();
                }
                if ui
                    .checkbox(&mut self.show_template_panel, "Template Library")
                    .changed()
                {
                    ui.close_menu();
                }
                ui.separator();
                if ui
                    .button(if self.dark_theme {
                        "Light Theme"
                    } else {
                        "Dark Theme"
                    })
                    .clicked()
                {
                    self.dark_theme = !self.dark_theme;
                    ui.close_menu();
                }
            });
        });
    }
}
