use egui::Ui;

use crate::model::character::Character;
use crate::model::locale::LocaleSettings;
use crate::model::node::{
    CompareOp, EventAction, EventActionType, VariableValue,
};

#[allow(clippy::too_many_arguments)]
pub(super) fn show_dialogue_inspector(
    ui: &mut Ui,
    data: &mut crate::model::node::DialogueData,
    characters: &[Character],
    editing_locale: &Option<String>,
    locale: &LocaleSettings,
    uuid8: &str,
    portrait_cache: &mut crate::ui::portrait_cache::PortraitCache,
    project_dir: Option<&std::path::Path>,
) -> bool {
    let mut snapshot_needed = false;

    ui.label("Speaker:");

    let current_label = if let Some(sid) = data.speaker_id {
        characters
            .iter()
            .find(|c| c.id == sid)
            .map(|c| c.name.clone())
            .unwrap_or_else(|| data.speaker_name.clone())
    } else if data.speaker_name.is_empty() {
        "(None)".to_string()
    } else {
        format!("{} (custom)", data.speaker_name)
    };

    egui::ComboBox::from_id_salt("speaker_combo")
        .selected_text(&current_label)
        .show_ui(ui, |ui| {
            if ui
                .selectable_label(
                    data.speaker_id.is_none() && data.speaker_name.is_empty(),
                    "(None)",
                )
                .clicked()
            {
                snapshot_needed = true;
                data.speaker_id = None;
                data.speaker_name.clear();
            }
            for ch in characters {
                let selected = data.speaker_id == Some(ch.id);
                if ui.selectable_label(selected, &ch.name).clicked() {
                    snapshot_needed = true;
                    data.speaker_id = Some(ch.id);
                    data.speaker_name = ch.name.clone();
                }
            }
        });

    // Manual name override
    ui.horizontal(|ui| {
        ui.label("Name:");
        let resp = ui.text_edit_singleline(&mut data.speaker_name);
        if resp.gained_focus() {
            snapshot_needed = true;
        }
        if resp.changed() {
            let matches_char = characters.iter().any(|c| c.name == data.speaker_name);
            if !matches_char {
                data.speaker_id = None;
            }
        }
    });

    // Portrait preview + override
    let portrait_path = data.portrait_override.as_deref()
        .filter(|s| !s.is_empty())
        .or_else(|| {
            data.speaker_id.and_then(|sid| {
                characters.iter().find(|c| c.id == sid)
                    .map(|c| c.portrait_path.as_str())
                    .filter(|s| !s.is_empty())
            })
        })
        .unwrap_or("");
    let tex_id = portrait_cache
        .get_or_load(ui.ctx(), portrait_path, project_dir)
        .map(|h| h.id());
    ui.horizontal(|ui| {
        ui.label("Portrait:");
        if let Some(id) = tex_id {
            ui.image(egui::load::SizedTexture::new(id, [24.0, 24.0]));
        }
        let mut override_text = data.portrait_override.clone().unwrap_or_default();
        let field_w = (ui.available_width() - 35.0).max(60.0);
        let resp = ui.add(
            egui::TextEdit::singleline(&mut override_text).desired_width(field_w),
        );
        if resp.gained_focus() {
            snapshot_needed = true;
        }
        if resp.changed() {
            data.portrait_override = if override_text.is_empty() { None } else { Some(override_text) };
        }
        if ui.small_button("[...]").on_hover_text("Browse for portrait image").clicked() {
            snapshot_needed = true;
            if let Some(path) = rfd::FileDialog::new()
                .add_filter("Images", &["png", "jpg", "jpeg", "bmp", "gif"])
                .pick_file()
            {
                let rel = crate::ui::portrait_cache::make_relative_path(&path, project_dir);
                data.portrait_override = Some(rel);
            }
        }
    });

    ui.add_space(4.0);
    ui.label("Text:");
    if editing_locale.is_some() {
        locale_text_field(ui, &data.text, &format!("dlg_{uuid8}"), editing_locale, locale, true);
    }
    if ui
        .add(egui::TextEdit::multiline(&mut data.text).desired_rows(4))
        .gained_focus()
    {
        snapshot_needed = true;
    }
    ui.colored_label(
        egui::Color32::from_rgb(140, 140, 140),
        "Use {variable} for interpolation",
    );

    ui.add_space(4.0);
    ui.label("Emotion:");
    let emotions = ["neutral", "happy", "sad", "angry", "surprised", "scared"];
    egui::ComboBox::from_id_salt("emotion_combo")
        .selected_text(&data.emotion)
        .show_ui(ui, |ui| {
            for e in &emotions {
                if ui.selectable_label(data.emotion == *e, *e).clicked() {
                    snapshot_needed = true;
                    data.emotion = e.to_string();
                }
            }
        });

    ui.add_space(4.0);
    ui.label("Audio clip:");
    ui.horizontal(|ui| {
        let mut audio = data.audio_clip.clone().unwrap_or_default();
        let resp = ui.text_edit_singleline(&mut audio);
        if resp.gained_focus() {
            snapshot_needed = true;
        }
        if resp.changed() {
            data.audio_clip = if audio.is_empty() { None } else { Some(audio) };
        }
        if ui.button("Browse").on_hover_text("Browse for audio file").clicked() {
            snapshot_needed = true;
            if let Some(path) = rfd::FileDialog::new()
                .add_filter("Audio", &["wav", "ogg", "mp3"])
                .pick_file()
            {
                data.audio_clip = Some(path.display().to_string());
            }
        }
    });

    snapshot_needed
}

pub(super) fn show_condition_inspector(
    ui: &mut Ui,
    data: &mut crate::model::node::ConditionData,
) -> bool {
    let mut snapshot_needed = false;

    ui.label("Variable:");
    if ui
        .text_edit_singleline(&mut data.variable_name)
        .gained_focus()
    {
        snapshot_needed = true;
    }

    ui.add_space(4.0);
    ui.label("Operator:");
    let ops = [
        (CompareOp::Eq, "=="),
        (CompareOp::Neq, "!="),
        (CompareOp::Gt, ">"),
        (CompareOp::Lt, "<"),
        (CompareOp::Gte, ">="),
        (CompareOp::Lte, "<="),
        (CompareOp::Contains, "contains"),
    ];
    let current_label = ops
        .iter()
        .find(|(op, _)| *op == data.operator)
        .map(|(_, l)| *l)
        .unwrap_or("==");
    egui::ComboBox::from_id_salt("op_combo")
        .selected_text(current_label)
        .show_ui(ui, |ui| {
            for (op, label) in &ops {
                if ui
                    .selectable_label(data.operator == *op, *label)
                    .clicked()
                {
                    snapshot_needed = true;
                    data.operator = *op;
                }
            }
        });

    ui.add_space(4.0);
    ui.label("Value:");
    if show_variable_value_editor(ui, &mut data.value, "cond_val") {
        snapshot_needed = true;
    }

    snapshot_needed
}

pub(super) fn show_event_inspector(
    ui: &mut Ui,
    data: &mut crate::model::node::EventData,
) -> bool {
    let mut snapshot_needed = false;

    ui.label("Actions:");
    ui.separator();

    let mut remove_idx = None;
    for (i, action) in data.actions.iter_mut().enumerate() {
        ui.group(|ui| {
            ui.horizontal(|ui| {
                ui.label(format!("#{}", i + 1));
                if ui.small_button("X").on_hover_text("Remove action").clicked() {
                    remove_idx = Some(i);
                    snapshot_needed = true;
                }
            });
            ui.label("Key:");
            if ui.text_edit_singleline(&mut action.key).gained_focus() {
                snapshot_needed = true;
            }
            ui.label("Value:");
            if show_variable_value_editor(ui, &mut action.value, &format!("evt_val_{i}"))
            {
                snapshot_needed = true;
            }
        });
    }

    if let Some(idx) = remove_idx {
        data.actions.remove(idx);
    }

    if ui.button("+ Add Action").on_hover_text("Add a new event action").clicked() {
        snapshot_needed = true;
        data.actions.push(EventAction {
            action_type: EventActionType::SetVariable,
            key: String::new(),
            value: VariableValue::Bool(false),
        });
    }

    snapshot_needed
}

/// Editor widget for a VariableValue (bool/int/float/text selector + value).
/// Returns `true` if an undo-worthy event started.
pub(super) fn show_variable_value_editor(
    ui: &mut Ui,
    value: &mut VariableValue,
    id: &str,
) -> bool {
    let mut snapshot_needed = false;
    let type_labels = ["Bool", "Int", "Float", "Text"];
    let current_type = match value {
        VariableValue::Bool(_) => 0,
        VariableValue::Int(_) => 1,
        VariableValue::Float(_) => 2,
        VariableValue::Text(_) => 3,
    };

    let mut selected = current_type;
    egui::ComboBox::from_id_salt(format!("{id}_type"))
        .selected_text(type_labels[selected])
        .show_ui(ui, |ui| {
            for (i, label) in type_labels.iter().enumerate() {
                if ui.selectable_label(selected == i, *label).clicked() {
                    selected = i;
                    snapshot_needed = true;
                }
            }
        });

    if selected != current_type {
        *value = match selected {
            0 => VariableValue::Bool(false),
            1 => VariableValue::Int(0),
            2 => VariableValue::Float(0.0),
            _ => VariableValue::Text(String::new()),
        };
    }

    match value {
        VariableValue::Bool(b) => {
            if ui.checkbox(b, "").changed() {
                snapshot_needed = true;
            }
        }
        VariableValue::Int(i) => {
            if ui.add(egui::DragValue::new(i)).drag_started() {
                snapshot_needed = true;
            }
        }
        VariableValue::Float(f) => {
            if ui.add(egui::DragValue::new(f).speed(0.1)).drag_started() {
                snapshot_needed = true;
            }
        }
        VariableValue::Text(s) => {
            if ui.text_edit_singleline(s).gained_focus() {
                snapshot_needed = true;
            }
        }
    }

    snapshot_needed
}

/// Sync output port labels with choice text.
pub(super) fn sync_choice_labels(node: &mut crate::model::node::Node) {
    if let crate::model::node::NodeType::Choice(data) = &node.node_type {
        for (i, choice) in data.choices.iter().enumerate() {
            if let Some(port) = node.outputs.get_mut(i) {
                port.label = choice.text.clone();
            }
        }
    }
}

/// Sync output port labels with random branch weights.
pub(super) fn sync_random_labels(node: &mut crate::model::node::Node) {
    if let crate::model::node::NodeType::Random(data) = &node.node_type {
        for (i, branch) in data.branches.iter().enumerate() {
            if let Some(port) = node.outputs.get_mut(i) {
                port.label = format!("{:.0}%", branch.weight * 100.0);
            }
        }
    }
}

/// Render a translation field for a translatable string.
/// Shows the current translation with a hint if untranslated.
/// Edits are buffered in egui memory and applied by `apply_locale_edits`.
pub(super) fn locale_text_field(
    ui: &mut Ui,
    _default_text: &str,
    key: &str,
    editing_locale: &Option<String>,
    locale: &LocaleSettings,
    multiline: bool,
) {
    let Some(ref loc) = editing_locale else {
        return;
    };
    let current = locale
        .get_translation(key, loc)
        .unwrap_or("")
        .to_string();
    let mut text = current.clone();

    ui.horizontal(|ui| {
        ui.label(format!("[{loc}]"));
        let resp = if multiline {
            ui.add(
                egui::TextEdit::multiline(&mut text)
                    .hint_text("(untranslated)")
                    .desired_rows(3),
            )
        } else {
            ui.add(egui::TextEdit::singleline(&mut text).hint_text("(untranslated)"))
        };
        if resp.changed() && text != current {
            ui.ctx().memory_mut(|mem| {
                let edits = mem
                    .data
                    .get_temp_mut_or_default::<Vec<(String, String)>>(
                        egui::Id::new("locale_edits"),
                    );
                edits.push((key.to_string(), text));
            });
        }
    });
}
