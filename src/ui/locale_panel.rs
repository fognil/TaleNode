use egui::Ui;
use uuid::Uuid;

use crate::model::graph::DialogueGraph;
use crate::model::locale::{collect_translatable_strings, LocaleSettings, TranslatableString};

pub enum LocalePanelAction {
    None,
    ExportCsv,
    ImportCsv,
    AddLocale(String),
    RemoveLocale(String),
    SetTranslation {
        key: String,
        locale: String,
        text: String,
    },
    Navigate(Uuid),
}

pub fn show_locale_panel(
    ui: &mut Ui,
    graph: &DialogueGraph,
    active_locale: &mut Option<String>,
    filter_untranslated: &mut bool,
    new_locale_name: &mut String,
) -> LocalePanelAction {
    let mut action = LocalePanelAction::None;
    let locale = &graph.locale;
    let strings = collect_translatable_strings(graph);

    // Locale management toolbar
    ui.horizontal(|ui| {
        ui.label("Default:");
        ui.monospace(&locale.default_locale);
        ui.separator();
        ui.label("Locales:");
        for loc in &locale.extra_locales {
            ui.label(loc);
        }
    });

    ui.horizontal(|ui| {
        ui.text_edit_singleline(new_locale_name);
        if ui.button("Add Locale").clicked() && !new_locale_name.trim().is_empty() {
            let name = new_locale_name.trim().to_lowercase();
            action = LocalePanelAction::AddLocale(name);
            *new_locale_name = String::new();
        }
    });

    ui.horizontal(|ui| {
        if ui.button("Export CSV").clicked() {
            action = LocalePanelAction::ExportCsv;
        }
        if ui.button("Import CSV").clicked() {
            action = LocalePanelAction::ImportCsv;
        }
    });

    // Remove locale buttons
    if !locale.extra_locales.is_empty() {
        ui.horizontal(|ui| {
            ui.label("Remove:");
            for loc in &locale.extra_locales {
                if ui.small_button(format!("x {loc}")).clicked() {
                    action = LocalePanelAction::RemoveLocale(loc.clone());
                }
            }
        });
    }

    // Progress bars per locale
    if !locale.extra_locales.is_empty() {
        show_progress_bars(ui, locale, &strings);
    }

    ui.separator();

    // Filter
    ui.horizontal(|ui| {
        ui.label("Filter:");
        ui.selectable_value(filter_untranslated, false, "All");
        ui.selectable_value(filter_untranslated, true, "Untranslated only");
        ui.separator();
        // Active locale for filtering
        ui.label("Locale:");
        egui::ComboBox::from_id_salt("locale_panel_active")
            .selected_text(
                active_locale
                    .as_deref()
                    .unwrap_or("(all)"),
            )
            .show_ui(ui, |ui| {
                if ui.selectable_label(active_locale.is_none(), "(all)").clicked() {
                    *active_locale = None;
                }
                for loc in &locale.extra_locales {
                    let selected = active_locale.as_deref() == Some(loc.as_str());
                    if ui.selectable_label(selected, loc).clicked() {
                        *active_locale = Some(loc.clone());
                    }
                }
            });
    });

    ui.separator();

    // Translation table
    show_translation_table(
        ui,
        locale,
        &strings,
        active_locale,
        *filter_untranslated,
        &mut action,
    );

    action
}

fn show_progress_bars(ui: &mut Ui, locale: &LocaleSettings, strings: &[TranslatableString]) {
    let total = strings.len();
    if total == 0 {
        return;
    }
    ui.add_space(4.0);
    for loc in &locale.extra_locales {
        let translated = strings
            .iter()
            .filter(|s| {
                locale
                    .get_translation(&s.key, loc)
                    .is_some_and(|t| !t.is_empty())
            })
            .count();
        let pct = translated as f32 / total as f32;
        ui.horizontal(|ui| {
            ui.label(format!("{loc}:"));
            ui.add(
                egui::ProgressBar::new(pct)
                    .text(format!("{translated}/{total} ({:.0}%)", pct * 100.0)),
            );
        });
    }
}

#[allow(clippy::too_many_arguments)]
fn show_translation_table(
    ui: &mut Ui,
    locale: &LocaleSettings,
    strings: &[TranslatableString],
    active_locale: &Option<String>,
    filter_untranslated: bool,
    action: &mut LocalePanelAction,
) {
    let visible_locales: Vec<&String> = if let Some(ref loc) = active_locale {
        locale
            .extra_locales
            .iter()
            .filter(|l| l.as_str() == loc.as_str())
            .collect()
    } else {
        locale.extra_locales.iter().collect()
    };

    egui::ScrollArea::both().show(ui, |ui| {
        egui::Grid::new("locale_table")
            .striped(true)
            .min_col_width(60.0)
            .show(ui, |ui| {
                // Header
                ui.strong("Key");
                ui.strong("Type");
                ui.strong(&locale.default_locale);
                for loc in &visible_locales {
                    ui.strong(loc.as_str());
                }
                ui.strong("");
                ui.end_row();

                for ts in strings {
                    if filter_untranslated && !has_missing_translation(locale, ts, &visible_locales)
                    {
                        continue;
                    }
                    show_string_row(ui, locale, ts, &visible_locales, action);
                }
            });
    });
}

fn has_missing_translation(
    locale: &LocaleSettings,
    ts: &TranslatableString,
    visible_locales: &[&String],
) -> bool {
    visible_locales.iter().any(|loc| {
        locale
            .get_translation(&ts.key, loc)
            .is_none_or(|t| t.is_empty())
    })
}

fn show_string_row(
    ui: &mut Ui,
    locale: &LocaleSettings,
    ts: &TranslatableString,
    visible_locales: &[&String],
    action: &mut LocalePanelAction,
) {
    ui.monospace(&ts.key);
    ui.label(ts.string_type.label());
    // Default text (read-only, truncated)
    let preview = if ts.default_text.len() > 40 {
        format!("{}...", &ts.default_text[..40])
    } else {
        ts.default_text.clone()
    };
    ui.label(&preview).on_hover_text(&ts.default_text);

    for loc in visible_locales {
        let current = locale
            .get_translation(&ts.key, loc)
            .unwrap_or("")
            .to_string();
        let mut text = current.clone();
        let id = format!("lt_{}_{}", ts.key, loc);
        let resp = ui.add(
            egui::TextEdit::singleline(&mut text)
                .hint_text("(untranslated)")
                .id(egui::Id::new(&id))
                .desired_width(120.0),
        );
        if resp.changed() && text != current {
            *action = LocalePanelAction::SetTranslation {
                key: ts.key.clone(),
                locale: loc.to_string(),
                text,
            };
        }
    }

    // Navigate button
    if ui.small_button("Go").on_hover_text("Select node").clicked() {
        *action = LocalePanelAction::Navigate(ts.node_id);
    }
    ui.end_row();
}
