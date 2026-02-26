use egui::Ui;

use crate::model::plugin::{PluginManifest, PluginType};

/// Actions returned by the extension panel.
pub enum ExtensionPanelAction {
    None,
    RefreshPlugins,
    RunExport(usize),
    RunImport(usize),
    OpenPluginsFolder,
}

/// Draw the extensions/plugin panel.
pub fn show_extension_panel(
    ui: &mut Ui,
    plugins: &[PluginManifest],
    last_result: &Option<(String, bool)>,
) -> ExtensionPanelAction {
    let mut action = ExtensionPanelAction::None;

    // Toolbar
    ui.horizontal(|ui| {
        if ui.button("Refresh").clicked() {
            action = ExtensionPanelAction::RefreshPlugins;
        }
        if ui.button("Open Plugins Folder").clicked() {
            action = ExtensionPanelAction::OpenPluginsFolder;
        }
    });

    ui.separator();

    // Status area
    if let Some((msg, is_error)) = last_result {
        let color = if *is_error {
            egui::Color32::from_rgb(220, 80, 80)
        } else {
            egui::Color32::from_rgb(80, 200, 80)
        };
        ui.colored_label(color, msg);
        ui.separator();
    }

    if plugins.is_empty() {
        ui.label("No plugins found.");
        ui.label("Place plugin folders in ~/.talenode/plugins/ or ./plugins/");
        return action;
    }

    // Plugin list
    for (i, plugin) in plugins.iter().enumerate() {
        egui::CollapsingHeader::new(&plugin.name)
            .id_salt(format!("plugin_{i}"))
            .default_open(false)
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label("Version:");
                    ui.label(&plugin.version);
                });
                if !plugin.author.is_empty() {
                    ui.horizontal(|ui| {
                        ui.label("Author:");
                        ui.label(&plugin.author);
                    });
                }
                if !plugin.description.is_empty() {
                    ui.label(&plugin.description);
                }
                ui.horizontal(|ui| {
                    ui.label("Type:");
                    let badge = plugin.plugin_type.label();
                    ui.label(badge);
                });
                ui.horizontal(|ui| {
                    let can_export = matches!(
                        plugin.plugin_type,
                        PluginType::Export | PluginType::ExportImport
                    );
                    let can_import = matches!(
                        plugin.plugin_type,
                        PluginType::Import | PluginType::ExportImport
                    );
                    if can_export && ui.button("Run Export").clicked() {
                        action = ExtensionPanelAction::RunExport(i);
                    }
                    if can_import && ui.button("Run Import").clicked() {
                        action = ExtensionPanelAction::RunImport(i);
                    }
                });
            });
    }

    action
}
