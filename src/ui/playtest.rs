use egui::Color32;
use uuid::Uuid;

use crate::model::graph::DialogueGraph;
use crate::model::node::NodeType;
use crate::model::port::PortId;
use crate::scripting::interpolate::interpolate_text;
use crate::scripting::{
    evaluate_condition, evaluate_condition_expr, execute_event, ScriptContext,
};

/// State for dialogue playtest/preview mode.
pub struct PlaytestState {
    /// Current node the playtest is on.
    pub current_node: Option<Uuid>,
    /// Log of dialogue lines played so far.
    pub log: Vec<PlaytestLogEntry>,
    /// Whether playtest is currently running.
    pub running: bool,
    /// Runtime variable context.
    pub variables: ScriptContext,
}

pub struct PlaytestLogEntry {
    pub speaker: String,
    pub text: String,
}

impl PlaytestState {
    pub fn new() -> Self {
        Self {
            current_node: None,
            log: Vec::new(),
            running: false,
            variables: ScriptContext::default(),
        }
    }

    /// Start playtest from the Start node.
    pub fn start(&mut self, graph: &DialogueGraph) {
        self.log.clear();
        self.running = true;
        self.variables = ScriptContext::from_variables(&graph.variables);
        self.current_node = graph
            .nodes
            .values()
            .find(|n| matches!(n.node_type, NodeType::Start))
            .map(|n| n.id);

        // Auto-advance past the Start node
        if let Some(id) = self.current_node {
            self.current_node = self.follow_first_output(graph, id);
            self.auto_advance(graph);
        }
    }

    pub fn stop(&mut self) {
        self.running = false;
        self.current_node = None;
    }

    /// Follow the first output port connection from a node.
    fn follow_first_output(&self, graph: &DialogueGraph, node_id: Uuid) -> Option<Uuid> {
        let node = graph.nodes.get(&node_id)?;
        let first_output = node.outputs.first()?;
        self.follow_port(graph, first_output.id)
    }

    /// Follow a specific output port to its connected node.
    fn follow_port(&self, graph: &DialogueGraph, port_id: PortId) -> Option<Uuid> {
        graph
            .connections
            .iter()
            .find(|c| c.from_port == port_id)
            .map(|c| c.to_node)
    }

    /// Auto-advance through nodes that don't need user interaction
    /// (Event, Condition, Random).
    fn auto_advance(&mut self, graph: &DialogueGraph) {
        while let Some(id) = self.current_node {
            let Some(node) = graph.nodes.get(&id) else { break };

            match &node.node_type {
                NodeType::Event(data) => {
                    execute_event(&mut self.variables, data);
                    self.log.push(PlaytestLogEntry {
                        speaker: "[Event]".to_string(),
                        text: format!("{} action(s) executed", data.actions.len()),
                    });
                    self.current_node = self.follow_first_output(graph, id);
                }
                NodeType::Condition(data) => {
                    let result = evaluate_condition(&self.variables, data);
                    self.log.push(PlaytestLogEntry {
                        speaker: "[Condition]".to_string(),
                        text: format!(
                            "{} {} {:?} → {}",
                            data.variable_name,
                            match data.operator {
                                crate::model::node::CompareOp::Eq => "==",
                                crate::model::node::CompareOp::Neq => "!=",
                                crate::model::node::CompareOp::Gt => ">",
                                crate::model::node::CompareOp::Lt => "<",
                                crate::model::node::CompareOp::Gte => ">=",
                                crate::model::node::CompareOp::Lte => "<=",
                                crate::model::node::CompareOp::Contains => "contains",
                            },
                            data.value,
                            result
                        ),
                    });
                    if result {
                        // True branch = first output (index 0)
                        if let Some(port) = node.outputs.first() {
                            self.current_node = self.follow_port(graph, port.id);
                        } else {
                            self.current_node = None;
                        }
                    } else {
                        // False branch = second output (index 1)
                        if let Some(port) = node.outputs.get(1) {
                            self.current_node = self.follow_port(graph, port.id);
                        } else {
                            self.current_node = None;
                        }
                    }
                }
                NodeType::Random(data) => {
                    // Pick a random branch weighted by weights
                    let total: f32 = data.branches.iter().map(|b| b.weight).sum();
                    let mut roll: f32 = rand_simple(total);
                    let mut chosen_idx = 0;
                    for (i, branch) in data.branches.iter().enumerate() {
                        roll -= branch.weight;
                        if roll <= 0.0 {
                            chosen_idx = i;
                            break;
                        }
                    }
                    self.log.push(PlaytestLogEntry {
                        speaker: "[Random]".to_string(),
                        text: format!("Branch {} selected", chosen_idx + 1),
                    });
                    // Follow the chosen output port
                    if let Some(port) = node.outputs.get(chosen_idx) {
                        self.current_node = self.follow_port(graph, port.id);
                    } else {
                        self.current_node = None;
                    }
                }
                NodeType::Start => {
                    self.current_node = self.follow_first_output(graph, id);
                }
                // Dialogue, Choice, End — stop and wait for UI
                _ => break,
            }
        }
    }

    /// Make a choice (for Choice nodes). `choice_index` is the output port index.
    pub fn make_choice(&mut self, graph: &DialogueGraph, choice_index: usize) {
        let Some(id) = self.current_node else { return };
        let Some(node) = graph.nodes.get(&id) else { return };

        if let NodeType::Choice(data) = &node.node_type {
            if let Some(choice) = data.choices.get(choice_index) {
                let text = interpolate_text(&choice.text, &self.variables);
                self.log.push(PlaytestLogEntry {
                    speaker: "[You]".to_string(),
                    text,
                });
            }
        }

        if let Some(port) = node.outputs.get(choice_index) {
            self.current_node = self.follow_port(graph, port.id);
        } else {
            self.current_node = None;
        }
        self.auto_advance(graph);
    }

    /// Advance past a Dialogue node (user clicked "Continue").
    pub fn advance_dialogue(&mut self, graph: &DialogueGraph) {
        let Some(id) = self.current_node else { return };
        let Some(node) = graph.nodes.get(&id) else { return };

        if let NodeType::Dialogue(data) = &node.node_type {
            let speaker = if data.speaker_name.is_empty() {
                "???".to_string()
            } else {
                data.speaker_name.clone()
            };
            let text = interpolate_text(&data.text, &self.variables);
            self.log.push(PlaytestLogEntry { speaker, text });
        }

        self.current_node = self.follow_first_output(graph, id);
        self.auto_advance(graph);
    }
}

/// Show the playtest panel UI.
pub fn show_playtest_panel(
    ui: &mut egui::Ui,
    state: &mut PlaytestState,
    graph: &DialogueGraph,
    selected_nodes: &mut Vec<Uuid>,
) {
    ui.horizontal(|ui| {
        ui.heading("Playtest");
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if state.running {
                if ui.button("Stop").clicked() {
                    state.stop();
                }
                if ui.button("Restart").clicked() {
                    state.start(graph);
                }
            } else if ui.button("Start").clicked() {
                state.start(graph);
            }
        });
    });
    ui.separator();

    if !state.running {
        ui.label("Press Start to begin playtest from the Start node.");
        return;
    }

    // Highlight current node on canvas
    if let Some(id) = state.current_node {
        if !selected_nodes.contains(&id) {
            selected_nodes.clear();
            selected_nodes.push(id);
        }
    }

    // Dialogue log
    egui::ScrollArea::vertical()
        .max_height(200.0)
        .stick_to_bottom(true)
        .show(ui, |ui| {
            for entry in &state.log {
                ui.horizontal(|ui| {
                    ui.colored_label(Color32::from_rgb(150, 200, 255), &entry.speaker);
                    ui.label(&entry.text);
                });
            }
        });

    ui.separator();

    // Current node interaction
    let Some(node_id) = state.current_node else {
        ui.colored_label(Color32::from_rgb(255, 200, 100), "End of dialogue reached.");
        show_variables_section(ui, state);
        return;
    };

    let Some(node) = graph.nodes.get(&node_id) else {
        ui.colored_label(Color32::from_rgb(255, 100, 100), "Error: node not found.");
        return;
    };

    match &node.node_type {
        NodeType::Dialogue(data) => {
            let speaker = if data.speaker_name.is_empty() {
                "???".to_string()
            } else {
                data.speaker_name.clone()
            };
            let text = interpolate_text(&data.text, &state.variables);
            ui.colored_label(Color32::from_rgb(150, 200, 255), &speaker);
            ui.label(&text);
            if !data.emotion.is_empty() {
                ui.colored_label(
                    Color32::from_rgb(180, 180, 180),
                    format!("[{}]", data.emotion),
                );
            }
            ui.add_space(4.0);
            // Need to clone state changes to avoid borrow conflict
            let mut should_advance = false;
            if ui.button("Continue >>").clicked() {
                should_advance = true;
            }
            if should_advance {
                state.advance_dialogue(graph);
            }
        }
        NodeType::Choice(data) => {
            if !data.prompt.is_empty() {
                let prompt = interpolate_text(&data.prompt, &state.variables);
                ui.label(&prompt);
            }
            ui.add_space(4.0);
            let mut chosen = None;
            for (i, choice) in data.choices.iter().enumerate() {
                let text = interpolate_text(&choice.text, &state.variables);
                // Check choice condition
                let available = choice
                    .condition
                    .as_ref()
                    .map(|cond| evaluate_condition_expr(&state.variables, cond))
                    .unwrap_or(true);

                if available {
                    if ui.button(&text).clicked() {
                        chosen = Some(i);
                    }
                } else {
                    ui.add_enabled(false, egui::Button::new(&text));
                }
            }
            if let Some(idx) = chosen {
                state.make_choice(graph, idx);
            }
        }
        NodeType::End(data) => {
            let tag = if data.tag.is_empty() {
                "default"
            } else {
                &data.tag
            };
            ui.colored_label(
                Color32::from_rgb(255, 200, 100),
                format!("Dialogue ended: [{tag}]"),
            );
        }
        _ => {
            ui.label("Unexpected node type in playtest.");
        }
    }

    show_variables_section(ui, state);
}

/// Show current variable values in a collapsible section.
fn show_variables_section(ui: &mut egui::Ui, state: &PlaytestState) {
    let vars = state.variables.all_vars();
    if vars.is_empty() {
        return;
    }
    ui.add_space(8.0);
    egui::CollapsingHeader::new("Variables")
        .default_open(false)
        .show(ui, |ui| {
            egui::Grid::new("playtest_vars_grid")
                .num_columns(2)
                .spacing([8.0, 2.0])
                .show(ui, |ui| {
                    for (name, value) in &vars {
                        ui.colored_label(Color32::from_rgb(180, 220, 180), *name);
                        let display = crate::scripting::eval::eval_to_string(value);
                        ui.monospace(&display);
                        ui.end_row();
                    }
                });
        });
}

/// Simple pseudo-random float in [0, max) using system time.
fn rand_simple(max: f32) -> f32 {
    use std::time::SystemTime;
    let nanos = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or_default()
        .subsec_nanos();
    (nanos as f32 / u32::MAX as f32) * max
}
