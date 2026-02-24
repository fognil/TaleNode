use std::path::PathBuf;
use uuid::Uuid;

use crate::export::json_export_helpers::build_id_map;
use crate::model::graph::DialogueGraph;
use crate::model::node::NodeType;

/// State for the batch audio assignment window.
#[derive(Default)]
pub struct AudioManagerState {
    pub open: bool,
    folder_path: Option<PathBuf>,
    matches: Vec<AudioMatch>,
    scan_done: bool,
}

struct AudioMatch {
    node_id: Uuid,
    readable_id: String,
    speaker: String,
    text_preview: String,
    audio_path: Option<String>,
}

impl AudioManagerState {
    fn scan_folder(&mut self, graph: &DialogueGraph) {
        let Some(folder) = &self.folder_path else { return };
        let id_map = build_id_map(graph);

        // Collect audio files from the folder
        let audio_files: Vec<(String, PathBuf)> = std::fs::read_dir(folder)
            .into_iter()
            .flatten()
            .filter_map(|e| e.ok())
            .filter(|e| {
                let path = e.path();
                matches!(
                    path.extension().and_then(|x| x.to_str()),
                    Some("wav" | "ogg" | "mp3")
                )
            })
            .map(|e| {
                let stem = e.path().file_stem()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .to_string();
                (stem, e.path())
            })
            .collect();

        // Build match entries for all dialogue nodes
        let mut entries: Vec<AudioMatch> = Vec::new();
        let mut dialogue_nodes: Vec<_> = graph
            .nodes
            .values()
            .filter(|n| matches!(n.node_type, NodeType::Dialogue(_)))
            .filter_map(|n| id_map.get(&n.id).map(|rid| (rid.clone(), n)))
            .collect();

        dialogue_nodes.sort_by(|a, b| {
            let na = a.0.strip_prefix("dlg_").and_then(|s| s.parse::<usize>().ok()).unwrap_or(0);
            let nb = b.0.strip_prefix("dlg_").and_then(|s| s.parse::<usize>().ok()).unwrap_or(0);
            na.cmp(&nb)
        });

        // Track speaker counts for speaker+index matching
        let mut speaker_counts: std::collections::HashMap<String, usize> = std::collections::HashMap::new();

        for (readable_id, node) in &dialogue_nodes {
            if let NodeType::Dialogue(data) = &node.node_type {
                let speaker = if let Some(sid) = data.speaker_id {
                    graph.characters.iter()
                        .find(|c| c.id == sid)
                        .map(|c| c.name.clone())
                        .unwrap_or_else(|| data.speaker_name.clone())
                } else {
                    data.speaker_name.clone()
                };

                let count = speaker_counts.entry(speaker.to_lowercase()).or_insert(0);
                *count += 1;
                let speaker_index = *count;

                let text_preview = if data.text.len() > 40 {
                    format!("{}...", &data.text[..37])
                } else {
                    data.text.clone()
                };

                // Try matching by readable ID first (e.g. dlg_1.wav)
                let mut matched_path = audio_files.iter()
                    .find(|(stem, _)| stem == readable_id)
                    .map(|(_, p)| p.display().to_string());

                // Fallback: match by speaker+index (e.g. elder_1.wav)
                if matched_path.is_none() {
                    let speaker_key = format!(
                        "{}_{}",
                        speaker.to_lowercase().replace(' ', "_"),
                        speaker_index
                    );
                    matched_path = audio_files.iter()
                        .find(|(stem, _)| stem.to_lowercase() == speaker_key)
                        .map(|(_, p)| p.display().to_string());
                }

                entries.push(AudioMatch {
                    node_id: node.id,
                    readable_id: readable_id.clone(),
                    speaker,
                    text_preview,
                    audio_path: matched_path,
                });
            }
        }

        self.matches = entries;
        self.scan_done = true;
    }
}

/// Show the batch audio assignment window.
pub fn show_audio_manager(
    ctx: &egui::Context,
    state: &mut AudioManagerState,
    graph: &mut DialogueGraph,
) {
    if !state.open {
        return;
    }

    let mut open = state.open;
    egui::Window::new("Batch Assign Audio")
        .open(&mut open)
        .default_width(500.0)
        .default_height(400.0)
        .show(ctx, |ui| {
            ui.horizontal(|ui| {
                if let Some(folder) = &state.folder_path {
                    ui.label(format!("Folder: {}", folder.display()));
                } else {
                    ui.label("No folder selected");
                }
                if ui.button("Select Folder").clicked() {
                    if let Some(folder) = rfd::FileDialog::new().pick_folder() {
                        state.folder_path = Some(folder);
                        state.scan_done = false;
                    }
                }
            });

            if state.folder_path.is_some() && !state.scan_done {
                state.scan_folder(graph);
            }

            if state.scan_done {
                ui.separator();
                let matched = state.matches.iter().filter(|m| m.audio_path.is_some()).count();
                let total = state.matches.len();
                ui.label(format!("Matched: {matched}/{total} dialogue lines"));
                ui.add_space(4.0);

                egui::ScrollArea::vertical().max_height(300.0).show(ui, |ui| {
                    egui::Grid::new("audio_match_grid")
                        .striped(true)
                        .min_col_width(60.0)
                        .show(ui, |ui| {
                            ui.strong("ID");
                            ui.strong("Speaker");
                            ui.strong("Text");
                            ui.strong("Audio File");
                            ui.end_row();

                            for m in &state.matches {
                                ui.label(&m.readable_id);
                                ui.label(&m.speaker);
                                ui.label(&m.text_preview);
                                if let Some(path) = &m.audio_path {
                                    let filename = std::path::Path::new(path)
                                        .file_name()
                                        .unwrap_or_default()
                                        .to_string_lossy();
                                    ui.colored_label(
                                        egui::Color32::from_rgb(100, 200, 100),
                                        filename.as_ref(),
                                    );
                                } else {
                                    ui.colored_label(
                                        egui::Color32::from_rgb(180, 180, 180),
                                        "(none)",
                                    );
                                }
                                ui.end_row();
                            }
                        });
                });

                ui.add_space(8.0);
                ui.horizontal(|ui| {
                    if ui.button("Apply Matches").clicked() {
                        for m in &state.matches {
                            if let Some(audio_path) = &m.audio_path {
                                if let Some(node) = graph.nodes.get_mut(&m.node_id) {
                                    if let NodeType::Dialogue(ref mut data) = node.node_type {
                                        data.audio_clip = Some(audio_path.clone());
                                    }
                                }
                            }
                        }
                        state.open = false;
                    }
                    if ui.button("Re-scan").clicked() {
                        state.scan_done = false;
                    }
                });
            }
        });
    state.open = open;
}
