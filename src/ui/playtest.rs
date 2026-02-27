use uuid::Uuid;

use crate::model::graph::DialogueGraph;
use crate::model::node::NodeType;
use crate::model::port::PortId;
use crate::scripting::interpolate::interpolate_text;
use crate::scripting::{evaluate_condition, execute_event, ScriptContext};

use super::playtest_checkpoint::{PlaytestCheckpoint, MAX_CHECKPOINTS};

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
    /// Stack for SubGraph auto-traversal.
    subgraph_stack: Vec<PlaytestSubFrame>,
    /// Saved playtest checkpoints (ephemeral, not persisted).
    pub checkpoints: Vec<PlaytestCheckpoint>,
    /// Counter for checkpoint IDs.
    checkpoint_counter: usize,
}

struct PlaytestSubFrame {
    graph: DialogueGraph,
    parent_next: Option<Uuid>,
}

#[derive(Clone)]
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
            subgraph_stack: Vec::new(),
            checkpoints: Vec::new(),
            checkpoint_counter: 0,
        }
    }

    /// Start playtest from the Start node.
    pub fn start(&mut self, graph: &DialogueGraph) {
        self.log.clear();
        self.running = true;
        self.subgraph_stack.clear();
        self.checkpoints.clear();
        self.checkpoint_counter = 0;
        self.variables = ScriptContext::from_variables(&graph.variables);
        self.current_node = graph
            .nodes
            .values()
            .find(|n| matches!(n.node_type, NodeType::Start))
            .map(|n| n.id);

        // Auto-advance past the Start node
        if let Some(id) = self.current_node {
            self.current_node = follow_first_output(graph, id);
            self.auto_advance(graph);
        }
    }

    pub fn stop(&mut self) {
        self.running = false;
        self.current_node = None;
        self.subgraph_stack.clear();
    }

    /// Whether the playtest is inside a SubGraph (checkpoints disabled).
    pub fn in_subgraph(&self) -> bool {
        !self.subgraph_stack.is_empty()
    }

    /// Save a checkpoint of the current playtest state.
    pub fn save_checkpoint(&mut self, label: String) {
        if self.checkpoints.len() >= MAX_CHECKPOINTS || self.in_subgraph() {
            return;
        }
        self.checkpoint_counter += 1;
        self.checkpoints.push(PlaytestCheckpoint {
            id: self.checkpoint_counter,
            label,
            current_node: self.current_node,
            log: self.log.clone(),
            variables: self.variables.to_pairs(),
        });
    }

    /// Load a previously saved checkpoint by ID.
    pub fn load_checkpoint(&mut self, id: usize) {
        let Some(idx) = self.checkpoints.iter().position(|cp| cp.id == id) else {
            return;
        };
        let cp = &self.checkpoints[idx];
        self.current_node = cp.current_node;
        self.log = cp.log.clone();
        self.variables = ScriptContext::from_pairs(cp.variables.clone());
        self.subgraph_stack.clear();
    }

    /// Delete a checkpoint by ID.
    pub fn delete_checkpoint(&mut self, id: usize) {
        self.checkpoints.retain(|cp| cp.id != id);
    }

    /// Get the active graph (subgraph if inside one, otherwise the root).
    pub fn active_graph<'a>(&'a self, root: &'a DialogueGraph) -> &'a DialogueGraph {
        self.subgraph_stack.last().map_or(root, |f| &f.graph)
    }

    /// Get a label for the current node (for checkpoint naming).
    pub fn current_node_label(&self, graph: &DialogueGraph) -> String {
        if let Some(id) = self.current_node {
            let active = self.subgraph_stack.last().map_or(graph, |f| &f.graph);
            active
                .nodes
                .get(&id)
                .map(|n| n.title().to_string())
                .unwrap_or_else(|| "Unknown".to_string())
        } else {
            "End".to_string()
        }
    }

    /// Auto-advance through nodes that don't need user interaction.
    fn auto_advance(&mut self, graph: &DialogueGraph) {
        while let Some(id) = self.current_node {
            self.variables.record_visit(&id.to_string());
            // Clone node_type + outputs to release borrow on subgraph_stack.
            let (node_type, outputs) = {
                let g = self.subgraph_stack.last().map_or(graph, |f| &f.graph);
                match g.nodes.get(&id) {
                    Some(n) => (n.node_type.clone(), n.outputs.clone()),
                    None => break,
                }
            };
            match node_type {
                NodeType::Event(data) => {
                    execute_event(&mut self.variables, &data);
                    self.log.push(PlaytestLogEntry {
                        speaker: "[Event]".to_string(),
                        text: format!("{} action(s) executed", data.actions.len()),
                    });
                    let g = self.subgraph_stack.last().map_or(graph, |f| &f.graph);
                    self.current_node = follow_first_output(g, id);
                }
                NodeType::Condition(data) => {
                    let result = evaluate_condition(&self.variables, &data);
                    self.log.push(PlaytestLogEntry {
                        speaker: "[Condition]".to_string(),
                        text: format!(
                            "{} {} {:?} → {}",
                            data.variable_name, op_str(data.operator),
                            data.value, result
                        ),
                    });
                    let port = if result { outputs.first() } else { outputs.get(1) };
                    let g = self.subgraph_stack.last().map_or(graph, |f| &f.graph);
                    self.current_node = port.and_then(|p| follow_port(g, p.id));
                }
                NodeType::Random(data) => {
                    let total: f32 = data.branches.iter().map(|b| b.weight).sum();
                    let mut roll: f32 = rand_simple(total);
                    let mut chosen = 0;
                    for (i, branch) in data.branches.iter().enumerate() {
                        roll -= branch.weight;
                        if roll <= 0.0 { chosen = i; break; }
                    }
                    self.log.push(PlaytestLogEntry {
                        speaker: "[Random]".to_string(),
                        text: format!("Branch {} selected", chosen + 1),
                    });
                    let g = self.subgraph_stack.last().map_or(graph, |f| &f.graph);
                    self.current_node = outputs.get(chosen).and_then(|p| follow_port(g, p.id));
                }
                NodeType::Start => {
                    let g = self.subgraph_stack.last().map_or(graph, |f| &f.graph);
                    self.current_node = follow_first_output(g, id);
                }
                NodeType::SubGraph(data) => {
                    let g = self.subgraph_stack.last().map_or(graph, |f| &f.graph);
                    let parent_next = follow_first_output(g, id);
                    self.log.push(PlaytestLogEntry {
                        speaker: "[SubGraph]".to_string(),
                        text: format!("Entering '{}'", data.name),
                    });
                    self.subgraph_stack.push(PlaytestSubFrame {
                        graph: data.child_graph,
                        parent_next,
                    });
                    let Some(frame) = self.subgraph_stack.last() else { break };
                    let start = frame.graph.nodes.values()
                        .find(|n| matches!(n.node_type, NodeType::Start))
                        .map(|n| n.id);
                    self.current_node = start.and_then(|sid| {
                        let f = self.subgraph_stack.last()?;
                        follow_first_output(&f.graph, sid)
                    });
                }
                NodeType::End(_) if !self.subgraph_stack.is_empty() => {
                    self.log.push(PlaytestLogEntry {
                        speaker: "[SubGraph]".to_string(),
                        text: "Exiting sub-graph".to_string(),
                    });
                    let Some(frame) = self.subgraph_stack.pop() else { break };
                    self.current_node = frame.parent_next;
                }
                _ => break,
            }
        }
    }

    /// Make a choice (for Choice nodes). `choice_index` is the output port index.
    pub fn make_choice(&mut self, graph: &DialogueGraph, choice_index: usize) {
        let Some(id) = self.current_node else { return };
        let choice_text = {
            let g = self.subgraph_stack.last().map_or(graph, |f| &f.graph);
            let Some(node) = g.nodes.get(&id) else { return };
            if let NodeType::Choice(data) = &node.node_type {
                data.choices.get(choice_index).map(|c| c.text.clone())
            } else { None }
        };
        let entry = choice_text.map(|raw| {
            self.variables.set_seq_scope(&id.to_string());
            let text = interpolate_text(&raw, &mut self.variables);
            PlaytestLogEntry { speaker: "[You]".to_string(), text }
        });
        if let Some(e) = entry { self.log.push(e); }
        let next = {
            let g = self.subgraph_stack.last().map_or(graph, |f| &f.graph);
            g.nodes.get(&id)
                .and_then(|n| n.outputs.get(choice_index))
                .and_then(|p| follow_port(g, p.id))
        };
        self.current_node = next;
        self.auto_advance(graph);
    }

    /// Advance past a Dialogue node (user clicked "Continue").
    pub fn advance_dialogue(&mut self, graph: &DialogueGraph) {
        let Some(id) = self.current_node else { return };
        let dlg_info = {
            let g = self.subgraph_stack.last().map_or(graph, |f| &f.graph);
            let Some(node) = g.nodes.get(&id) else { return };
            if let NodeType::Dialogue(data) = &node.node_type {
                let speaker = if data.speaker_name.is_empty() {
                    "???".to_string()
                } else {
                    data.speaker_name.clone()
                };
                Some((speaker, data.text.clone()))
            } else { None }
        };
        let entry = dlg_info.map(|(speaker, raw)| {
            self.variables.set_seq_scope(&id.to_string());
            let text = interpolate_text(&raw, &mut self.variables);
            PlaytestLogEntry { speaker, text }
        });
        if let Some(e) = entry { self.log.push(e); }
        let g = self.subgraph_stack.last().map_or(graph, |f| &f.graph);
        self.current_node = follow_first_output(g, id);
        self.auto_advance(graph);
    }
}

fn follow_first_output(graph: &DialogueGraph, node_id: Uuid) -> Option<Uuid> {
    let node = graph.nodes.get(&node_id)?;
    let port = node.outputs.first()?;
    follow_port(graph, port.id)
}

fn follow_port(graph: &DialogueGraph, port_id: PortId) -> Option<Uuid> {
    graph.connections.iter().find(|c| c.from_port == port_id).map(|c| c.to_node)
}

fn op_str(op: crate::model::node::CompareOp) -> &'static str {
    use crate::model::node::CompareOp;
    match op {
        CompareOp::Eq => "==", CompareOp::Neq => "!=", CompareOp::Gt => ">",
        CompareOp::Lt => "<", CompareOp::Gte => ">=", CompareOp::Lte => "<=",
        CompareOp::Contains => "contains",
    }
}


/// Pseudo-random float in [0, max) using thread-local PRNG.
fn rand_simple(max: f32) -> f32 {
    fastrand::f32() * max
}
