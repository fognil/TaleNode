use egui::Ui;
use uuid::Uuid;

use crate::app::async_runtime::VoiceInfo;
use crate::model::graph::DialogueGraph;
use crate::model::node::NodeType;

pub enum VoicePanelAction {
    None,
    FetchVoices,
    GenerateForNode(Uuid),
    GenerateAll,
}

/// Info about a dialogue node for the voice table.
struct VoiceRow {
    node_id: Uuid,
    speaker: String,
    text_preview: String,
    has_audio: bool,
    voice_assigned: bool,
}

pub fn show_voice_panel(
    ui: &mut Ui,
    graph: &DialogueGraph,
    available_voices: &[VoiceInfo],
    in_progress: bool,
) -> VoicePanelAction {
    let mut action = VoicePanelAction::None;

    // Toolbar
    ui.horizontal(|ui| {
        if ui.button("Fetch Voices").clicked() {
            action = VoicePanelAction::FetchVoices;
        }
        if !available_voices.is_empty() {
            ui.label(format!("{} voices available", available_voices.len()));
        }
        ui.separator();
        let label = if in_progress {
            "Generating..."
        } else {
            "Generate All"
        };
        if ui
            .add_enabled(!in_progress, egui::Button::new(label))
            .clicked()
        {
            action = VoicePanelAction::GenerateAll;
        }
    });

    ui.separator();

    // Collect dialogue node info
    let rows = collect_voice_rows(graph);

    if rows.is_empty() {
        ui.label("No dialogue nodes in graph.");
        return action;
    }

    // Stats
    let total = rows.len();
    let with_audio = rows.iter().filter(|r| r.has_audio).count();
    let with_voice = rows.iter().filter(|r| r.voice_assigned).count();
    ui.label(format!(
        "{total} dialogue nodes | {with_voice} with voice assigned | {with_audio} with audio"
    ));
    ui.separator();

    // Table
    show_voice_table(ui, &rows, in_progress, &mut action);

    action
}

fn collect_voice_rows(graph: &DialogueGraph) -> Vec<VoiceRow> {
    let mut rows = Vec::new();
    let mut nodes: Vec<_> = graph.nodes.values().collect();
    nodes.sort_by(|a, b| {
        a.position[1]
            .partial_cmp(&b.position[1])
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    for node in nodes {
        if let NodeType::Dialogue(ref data) = node.node_type {
            let speaker = if let Some(sid) = &data.speaker_id {
                graph
                    .characters
                    .iter()
                    .find(|c| &c.id == sid)
                    .map(|c| c.name.clone())
                    .unwrap_or_else(|| data.speaker_name.clone())
            } else {
                data.speaker_name.clone()
            };

            let voice_assigned = data
                .speaker_id
                .and_then(|sid| graph.characters.iter().find(|c| c.id == sid))
                .is_some_and(|c| c.voice_id.is_some());

            let text_preview = if data.text.len() > 50 {
                format!("{}...", &data.text[..50])
            } else {
                data.text.clone()
            };

            rows.push(VoiceRow {
                node_id: node.id,
                speaker,
                text_preview,
                has_audio: data.audio_clip.is_some(),
                voice_assigned,
            });
        }
    }
    rows
}

fn show_voice_table(
    ui: &mut Ui,
    rows: &[VoiceRow],
    in_progress: bool,
    action: &mut VoicePanelAction,
) {
    egui::ScrollArea::both().show(ui, |ui| {
        egui::Grid::new("voice_table")
            .striped(true)
            .min_col_width(60.0)
            .show(ui, |ui| {
                ui.strong("Speaker");
                ui.strong("Text");
                ui.strong("Voice");
                ui.strong("Audio");
                ui.strong("");
                ui.end_row();

                for row in rows {
                    ui.label(&row.speaker);
                    ui.label(&row.text_preview)
                        .on_hover_text(&row.text_preview);
                    ui.label(if row.voice_assigned {
                        "Assigned"
                    } else {
                        "—"
                    });
                    ui.label(if row.has_audio { "Yes" } else { "—" });
                    if ui
                        .add_enabled(!in_progress, egui::Button::new("Generate"))
                        .clicked()
                    {
                        *action = VoicePanelAction::GenerateForNode(row.node_id);
                    }
                    ui.end_row();
                }
            });
    });
}
