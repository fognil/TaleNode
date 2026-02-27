use egui::Ui;
use uuid::Uuid;

use crate::model::graph::DialogueGraph;
use crate::model::node::NodeType;

pub enum WritingAction {
    None,
    SuggestDialogue { node_id: Uuid, instruction: String },
    GenerateChoices { node_id: Uuid, count: usize },
    CheckTone { node_id: Uuid },
    ApplyDialogue { node_id: Uuid, text: String },
    ApplyChoices { node_id: Uuid, choices: Vec<String> },
}

#[allow(clippy::too_many_arguments)]
pub fn show_writing_panel(
    ui: &mut Ui,
    graph: &DialogueGraph,
    selected_nodes: &std::collections::HashSet<Uuid>,
    in_progress: bool,
    suggestions: &Option<(Uuid, Vec<String>)>,
    tone_report: &Option<(Uuid, String)>,
    instruction: &mut String,
    choice_count: &mut usize,
) -> WritingAction {
    let mut action = WritingAction::None;

    if selected_nodes.len() != 1 {
        ui.centered_and_justified(|ui| {
            ui.label("Select a Dialogue or Choice node");
        });
        return action;
    }

    let Some(&node_id) = selected_nodes.iter().next() else { return action };
    let Some(node) = graph.nodes.get(&node_id) else {
        ui.label("Node not found");
        return action;
    };

    match &node.node_type {
        NodeType::Dialogue(data) => {
            show_dialogue_section(
                ui,
                node_id,
                data,
                graph,
                in_progress,
                suggestions,
                tone_report,
                instruction,
                &mut action,
            );
        }
        NodeType::Choice(data) => {
            show_choice_section(
                ui,
                node_id,
                data,
                in_progress,
                suggestions,
                instruction,
                choice_count,
                &mut action,
            );
        }
        _ => {
            ui.centered_and_justified(|ui| {
                ui.label("Select a Dialogue or Choice node");
            });
        }
    }

    action
}

#[allow(clippy::too_many_arguments)]
fn show_dialogue_section(
    ui: &mut Ui,
    node_id: Uuid,
    data: &crate::model::node::DialogueData,
    graph: &DialogueGraph,
    in_progress: bool,
    suggestions: &Option<(Uuid, Vec<String>)>,
    tone_report: &Option<(Uuid, String)>,
    instruction: &mut String,
    action: &mut WritingAction,
) {
    let speaker = if let Some(sid) = &data.speaker_id {
        graph
            .characters
            .iter()
            .find(|c| &c.id == sid)
            .map(|c| c.name.as_str())
            .unwrap_or(&data.speaker_name)
    } else if !data.speaker_name.is_empty() {
        &data.speaker_name
    } else {
        "Unknown"
    };

    ui.heading("Dialogue Node");
    ui.label(format!("Speaker: {speaker}  |  Emotion: {}", data.emotion));
    ui.separator();
    ui.label("Current text:");
    ui.group(|ui| {
        ui.label(&data.text);
    });
    ui.separator();

    ui.label("Custom instruction (optional):");
    ui.text_edit_singleline(instruction);
    ui.add_space(4.0);

    ui.horizontal(|ui| {
        let suggest_label = if in_progress {
            "Generating..."
        } else {
            "Suggest Alternatives"
        };
        if ui
            .add_enabled(!in_progress, egui::Button::new(suggest_label))
            .clicked()
        {
            *action = WritingAction::SuggestDialogue {
                node_id,
                instruction: instruction.clone(),
            };
        }

        let tone_label = if in_progress {
            "Checking..."
        } else {
            "Check Tone"
        };
        if ui
            .add_enabled(!in_progress, egui::Button::new(tone_label))
            .clicked()
        {
            *action = WritingAction::CheckTone { node_id };
        }
    });

    // Show suggestions
    if let Some((sid, items)) = suggestions {
        if *sid == node_id && !items.is_empty() {
            ui.add_space(8.0);
            ui.separator();
            ui.strong("Suggestions:");
            for (i, suggestion) in items.iter().enumerate() {
                ui.horizontal(|ui| {
                    ui.label(format!("{}.", i + 1));
                    ui.label(suggestion);
                    if ui.small_button("Apply").clicked() {
                        *action = WritingAction::ApplyDialogue {
                            node_id,
                            text: suggestion.clone(),
                        };
                    }
                });
            }
        }
    }

    // Show tone report
    if let Some((tid, report)) = tone_report {
        if *tid == node_id {
            ui.add_space(8.0);
            ui.separator();
            ui.strong("Tone Analysis:");
            ui.label(report);
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn show_choice_section(
    ui: &mut Ui,
    node_id: Uuid,
    data: &crate::model::node::ChoiceData,
    in_progress: bool,
    suggestions: &Option<(Uuid, Vec<String>)>,
    instruction: &mut String,
    choice_count: &mut usize,
    action: &mut WritingAction,
) {
    ui.heading("Choice Node");
    ui.label(format!("Prompt: {}", data.prompt));
    ui.separator();

    ui.label("Current choices:");
    for (i, opt) in data.choices.iter().enumerate() {
        ui.label(format!("  {}. {}", i + 1, opt.text));
    }
    ui.separator();

    ui.horizontal(|ui| {
        ui.label("Generate count:");
        ui.add(egui::DragValue::new(choice_count).range(2..=6));
    });

    ui.label("Custom instruction (optional):");
    ui.text_edit_singleline(instruction);
    ui.add_space(4.0);

    let gen_label = if in_progress {
        "Generating..."
    } else {
        "Generate Options"
    };
    if ui
        .add_enabled(!in_progress, egui::Button::new(gen_label))
        .clicked()
    {
        *action = WritingAction::GenerateChoices {
            node_id,
            count: *choice_count,
        };
    }

    // Show suggestions
    if let Some((sid, items)) = suggestions {
        if *sid == node_id && !items.is_empty() {
            ui.add_space(8.0);
            ui.separator();
            ui.strong("Generated Options:");
            for (i, suggestion) in items.iter().enumerate() {
                ui.label(format!("  {}. {}", i + 1, suggestion));
            }
            ui.add_space(4.0);
            if ui.button("Apply All").clicked() {
                *action = WritingAction::ApplyChoices {
                    node_id,
                    choices: items.clone(),
                };
            }
        }
    }
}
