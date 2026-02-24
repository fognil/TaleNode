use egui::Color32;
use uuid::Uuid;

use crate::model::graph::DialogueGraph;
use crate::model::node::NodeType;
use crate::model::port::PortId;

/// State for dialogue playtest/preview mode.
pub struct PlaytestState {
    /// Current node the playtest is on.
    pub current_node: Option<Uuid>,
    /// Log of dialogue lines played so far.
    pub log: Vec<PlaytestLogEntry>,
    /// Whether playtest is currently running.
    pub running: bool,
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
        }
    }

    /// Start playtest from the Start node.
    pub fn start(&mut self, graph: &DialogueGraph) {
        self.log.clear();
        self.running = true;
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
        loop {
            let Some(id) = self.current_node else { break };
            let Some(node) = graph.nodes.get(&id) else { break };

            match &node.node_type {
                NodeType::Event(_data) => {
                    self.log.push(PlaytestLogEntry {
                        speaker: "[Event]".to_string(),
                        text: format!("{} action(s) triggered", _data.actions.len()),
                    });
                    self.current_node = self.follow_first_output(graph, id);
                }
                NodeType::Condition(data) => {
                    // In playtest, always take the "True" branch (first output)
                    self.log.push(PlaytestLogEntry {
                        speaker: "[Condition]".to_string(),
                        text: format!("{} → True (simulated)", data.variable_name),
                    });
                    self.current_node = self.follow_first_output(graph, id);
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
                self.log.push(PlaytestLogEntry {
                    speaker: "[You]".to_string(),
                    text: choice.text.clone(),
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
            self.log.push(PlaytestLogEntry {
                speaker,
                text: data.text.clone(),
            });
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
            ui.colored_label(Color32::from_rgb(150, 200, 255), &speaker);
            ui.label(&data.text);
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
                ui.label(&data.prompt);
            }
            ui.add_space(4.0);
            let mut chosen = None;
            for (i, choice) in data.choices.iter().enumerate() {
                if ui.button(&choice.text).clicked() {
                    chosen = Some(i);
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
