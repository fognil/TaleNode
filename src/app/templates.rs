use std::collections::{HashMap, HashSet};
use std::time::Instant;

use uuid::Uuid;

use crate::model::connection::Connection;
use crate::model::graph::DialogueGraph;
use crate::model::node::{Node, NodeType};
use crate::model::port::PortId;
use crate::model::template::{NodeTemplate, TemplateLibrary};

use super::TaleNodeApp;

impl TaleNodeApp {
    /// Insert a template at the given canvas position.
    /// Nodes get fresh UUIDs; internal connections are remapped.
    pub(super) fn insert_template(&mut self, template: &NodeTemplate, canvas_pos: [f32; 2]) {
        self.snapshot();
        let mut id_map: HashMap<Uuid, Uuid> = HashMap::new();
        let mut port_map: HashMap<PortId, PortId> = HashMap::new();
        let mut new_ids = Vec::new();

        for (&old_id, node) in &template.nodes {
            let mut dup = node.clone();
            let new_id = Uuid::new_v4();
            id_map.insert(old_id, new_id);
            dup.id = new_id;
            dup.position[0] += canvas_pos[0];
            dup.position[1] += canvas_pos[1];
            for port in dup.inputs.iter_mut().chain(dup.outputs.iter_mut()) {
                let new_port = PortId(Uuid::new_v4());
                port_map.insert(port.id, new_port);
                port.id = new_port;
            }
            if let NodeType::SubGraph(ref mut data) = dup.node_type {
                regenerate_child_ids(&mut data.child_graph);
            }
            new_ids.push(new_id);
            self.graph.add_node(dup);
        }

        for conn in &template.connections {
            if let (Some(&fn_id), Some(&tn_id), Some(&fp), Some(&tp)) = (
                id_map.get(&conn.from_node),
                id_map.get(&conn.to_node),
                port_map.get(&conn.from_port),
                port_map.get(&conn.to_port),
            ) {
                self.graph.add_connection(fn_id, fp, tn_id, tp);
            }
        }

        self.selected_nodes = new_ids;
    }

    /// Save the current selection as a named template.
    pub(super) fn save_selection_as_template(&mut self, name: String) {
        let selected: HashSet<Uuid> = self.selected_nodes.iter().copied().collect();
        if selected.is_empty() {
            return;
        }

        // Find bounding box min to normalize positions.
        let mut min_x = f32::MAX;
        let mut min_y = f32::MAX;
        for id in &selected {
            if let Some(node) = self.graph.nodes.get(id) {
                min_x = min_x.min(node.position[0]);
                min_y = min_y.min(node.position[1]);
            }
        }

        let mut nodes = HashMap::new();
        for id in &selected {
            if let Some(node) = self.graph.nodes.get(id) {
                let mut n = node.clone();
                n.position[0] -= min_x;
                n.position[1] -= min_y;
                nodes.insert(*id, n);
            }
        }

        let connections: Vec<Connection> = self
            .graph
            .connections
            .iter()
            .filter(|c| selected.contains(&c.from_node) && selected.contains(&c.to_node))
            .cloned()
            .collect();

        let template = NodeTemplate {
            id: Uuid::new_v4(),
            name,
            description: String::new(),
            is_builtin: false,
            nodes,
            connections,
        };
        self.template_library.templates.push(template);
        self.save_template_library();
        self.status_message = Some(("Template saved".to_string(), Instant::now(), false));
    }

    /// Delete a user-saved template by ID.
    pub(super) fn delete_template(&mut self, template_id: Uuid) {
        self.template_library
            .templates
            .retain(|t| t.id != template_id || t.is_builtin);
        self.save_template_library();
    }

    /// Load the template library from disk, merging built-in templates.
    pub(super) fn load_template_library() -> TemplateLibrary {
        let path = template_library_path();
        let mut lib = if path.exists() {
            std::fs::read_to_string(&path)
                .ok()
                .and_then(|s| serde_json::from_str::<TemplateLibrary>(&s).ok())
                .unwrap_or_default()
        } else {
            TemplateLibrary::default()
        };

        // Ensure all builtins are present.
        for builtin in builtin_templates() {
            if !lib.templates.iter().any(|t| t.name == builtin.name && t.is_builtin) {
                lib.templates.push(builtin);
            }
        }
        lib
    }

    /// Persist the template library to disk.
    pub(super) fn save_template_library(&self) {
        let path = template_library_path();
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        if let Ok(json) = serde_json::to_string_pretty(&self.template_library) {
            if let Err(e) = std::fs::write(&path, json) {
                eprintln!("Failed to save template library: {e}");
            }
        }
    }
}

/// Regenerate all UUIDs inside a child graph (used for SubGraph duplication).
pub(super) fn regenerate_child_ids(g: &mut DialogueGraph) {
    let (mut ids, mut ports) = (HashMap::new(), HashMap::new());
    for (oid, mut n) in g.nodes.drain().collect::<Vec<_>>() {
        let nid = Uuid::new_v4();
        ids.insert(oid, nid);
        n.id = nid;
        for p in n.inputs.iter_mut().chain(n.outputs.iter_mut()) {
            let np = PortId(Uuid::new_v4());
            ports.insert(p.id, np);
            p.id = np;
        }
        g.nodes.insert(nid, n);
    }
    for c in &mut g.connections {
        c.id = Uuid::new_v4();
        c.from_node = ids.get(&c.from_node).copied().unwrap_or(c.from_node);
        c.to_node = ids.get(&c.to_node).copied().unwrap_or(c.to_node);
        c.from_port = ports.get(&c.from_port).copied().unwrap_or(c.from_port);
        c.to_port = ports.get(&c.to_port).copied().unwrap_or(c.to_port);
    }
}

fn template_library_path() -> std::path::PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    std::path::PathBuf::from(home)
        .join(".config")
        .join("talenode")
        .join("templates.json")
}

/// Built-in template presets.
fn builtin_templates() -> Vec<NodeTemplate> {
    vec![
        builtin_guard_check(),
        builtin_shop_encounter(),
        builtin_quest_giver(),
    ]
}

fn builtin_guard_check() -> NodeTemplate {
    let mut graph = DialogueGraph::new();
    let start = Node::new_start([0.0, 100.0]);
    let mut cond = Node::new_condition([200.0, 100.0]);
    if let NodeType::Condition(ref mut d) = cond.node_type {
        d.variable_name = "has_pass".to_string();
        d.operator = crate::model::node::CompareOp::Eq;
        d.value = crate::model::node::VariableValue::Bool(true);
    }
    let mut dlg_pass = Node::new_dialogue([450.0, 0.0]);
    if let NodeType::Dialogue(ref mut d) = dlg_pass.node_type {
        d.speaker_name = "Guard".to_string();
        d.text = "You may pass.".to_string();
    }
    let mut dlg_halt = Node::new_dialogue([450.0, 200.0]);
    if let NodeType::Dialogue(ref mut d) = dlg_halt.node_type {
        d.speaker_name = "Guard".to_string();
        d.text = "Halt! You shall not pass!".to_string();
    }
    let end_pass = Node::new_end([700.0, 0.0]);
    let end_halt = Node::new_end([700.0, 200.0]);

    let s_out = start.outputs[0].id;
    let c_in = cond.inputs[0].id;
    let c_true = cond.outputs[0].id;
    let c_false = cond.outputs[1].id;
    let dp_in = dlg_pass.inputs[0].id;
    let dp_out = dlg_pass.outputs[0].id;
    let dh_in = dlg_halt.inputs[0].id;
    let dh_out = dlg_halt.outputs[0].id;
    let ep_in = end_pass.inputs[0].id;
    let eh_in = end_halt.inputs[0].id;

    let (sid, cid) = (start.id, cond.id);
    let (dpid, dhid) = (dlg_pass.id, dlg_halt.id);
    let (epid, ehid) = (end_pass.id, end_halt.id);

    graph.add_node(start);
    graph.add_node(cond);
    graph.add_node(dlg_pass);
    graph.add_node(dlg_halt);
    graph.add_node(end_pass);
    graph.add_node(end_halt);
    graph.add_connection(sid, s_out, cid, c_in);
    graph.add_connection(cid, c_true, dpid, dp_in);
    graph.add_connection(cid, c_false, dhid, dh_in);
    graph.add_connection(dpid, dp_out, epid, ep_in);
    graph.add_connection(dhid, dh_out, ehid, eh_in);

    NodeTemplate {
        id: Uuid::new_v4(),
        name: "Guard Check".to_string(),
        description: "Condition check with pass/fail dialogue".to_string(),
        is_builtin: true,
        nodes: graph.nodes,
        connections: graph.connections,
    }
}

fn builtin_shop_encounter() -> NodeTemplate {
    let mut graph = DialogueGraph::new();
    let start = Node::new_start([0.0, 100.0]);
    let mut dlg = Node::new_dialogue([200.0, 100.0]);
    if let NodeType::Dialogue(ref mut d) = dlg.node_type {
        d.speaker_name = "Shopkeeper".to_string();
        d.text = "Welcome to my shop! What can I do for you?".to_string();
    }
    let mut choice = Node::new_choice([450.0, 100.0]);
    if let NodeType::Choice(ref mut d) = choice.node_type {
        d.prompt = "What would you like to do?".to_string();
        if let Some(c) = d.choices.get_mut(0) {
            c.text = "Buy".to_string();
        }
        if let Some(c) = d.choices.get_mut(1) {
            c.text = "Sell".to_string();
        }
    }
    let end_buy = Node::new_end([700.0, 0.0]);
    let end_sell = Node::new_end([700.0, 100.0]);
    let end_leave = Node::new_end([700.0, 200.0]);

    // Add a "Leave" option
    let leave_port = choice.add_choice();
    if let NodeType::Choice(ref mut d) = choice.node_type {
        if let Some(c) = d.choices.last_mut() {
            c.text = "Leave".to_string();
        }
    }

    let s_out = start.outputs[0].id;
    let d_in = dlg.inputs[0].id;
    let d_out = dlg.outputs[0].id;
    let ch_in = choice.inputs[0].id;
    let ch_out0 = choice.outputs[0].id;
    let ch_out1 = choice.outputs[1].id;
    let eb_in = end_buy.inputs[0].id;
    let es_in = end_sell.inputs[0].id;
    let el_in = end_leave.inputs[0].id;

    let (sid, did) = (start.id, dlg.id);
    let chid = choice.id;
    let (ebid, esid, elid) = (end_buy.id, end_sell.id, end_leave.id);

    graph.add_node(start);
    graph.add_node(dlg);
    graph.add_node(choice);
    graph.add_node(end_buy);
    graph.add_node(end_sell);
    graph.add_node(end_leave);
    graph.add_connection(sid, s_out, did, d_in);
    graph.add_connection(did, d_out, chid, ch_in);
    graph.add_connection(chid, ch_out0, ebid, eb_in);
    graph.add_connection(chid, ch_out1, esid, es_in);
    if let Some(lp) = leave_port {
        graph.add_connection(chid, PortId(lp), elid, el_in);
    }

    NodeTemplate {
        id: Uuid::new_v4(),
        name: "Shop Encounter".to_string(),
        description: "Shopkeeper with buy/sell/leave choices".to_string(),
        is_builtin: true,
        nodes: graph.nodes,
        connections: graph.connections,
    }
}

fn builtin_quest_giver() -> NodeTemplate {
    let mut graph = DialogueGraph::new();
    let start = Node::new_start([0.0, 100.0]);
    let mut dlg = Node::new_dialogue([200.0, 100.0]);
    if let NodeType::Dialogue(ref mut d) = dlg.node_type {
        d.speaker_name = "Quest Giver".to_string();
        d.text = "I need your help with something...".to_string();
    }
    let mut choice = Node::new_choice([450.0, 100.0]);
    if let NodeType::Choice(ref mut d) = choice.node_type {
        d.prompt = "Will you help?".to_string();
        if let Some(c) = d.choices.get_mut(0) {
            c.text = "Accept".to_string();
        }
        if let Some(c) = d.choices.get_mut(1) {
            c.text = "Decline".to_string();
        }
    }
    let mut evt = Node::new_event([700.0, 0.0]);
    if let NodeType::Event(ref mut d) = evt.node_type {
        d.actions.push(crate::model::node::EventAction {
            action_type: crate::model::node::EventActionType::SetVariable,
            key: "quest_active".to_string(),
            value: crate::model::node::VariableValue::Bool(true),
        });
    }
    let mut dlg_decline = Node::new_dialogue([700.0, 200.0]);
    if let NodeType::Dialogue(ref mut d) = dlg_decline.node_type {
        d.speaker_name = "Quest Giver".to_string();
        d.text = "Maybe another time, then.".to_string();
    }
    let end_accept = Node::new_end([950.0, 0.0]);
    let end_decline = Node::new_end([950.0, 200.0]);

    let s_out = start.outputs[0].id;
    let d_in = dlg.inputs[0].id;
    let d_out = dlg.outputs[0].id;
    let ch_in = choice.inputs[0].id;
    let ch_out0 = choice.outputs[0].id;
    let ch_out1 = choice.outputs[1].id;
    let ev_in = evt.inputs[0].id;
    let ev_out = evt.outputs[0].id;
    let dd_in = dlg_decline.inputs[0].id;
    let dd_out = dlg_decline.outputs[0].id;
    let ea_in = end_accept.inputs[0].id;
    let ed_in = end_decline.inputs[0].id;

    let (sid, did) = (start.id, dlg.id);
    let chid = choice.id;
    let (evid, ddid) = (evt.id, dlg_decline.id);
    let (eaid, edid) = (end_accept.id, end_decline.id);

    graph.add_node(start);
    graph.add_node(dlg);
    graph.add_node(choice);
    graph.add_node(evt);
    graph.add_node(dlg_decline);
    graph.add_node(end_accept);
    graph.add_node(end_decline);
    graph.add_connection(sid, s_out, did, d_in);
    graph.add_connection(did, d_out, chid, ch_in);
    graph.add_connection(chid, ch_out0, evid, ev_in);
    graph.add_connection(chid, ch_out1, ddid, dd_in);
    graph.add_connection(evid, ev_out, eaid, ea_in);
    graph.add_connection(ddid, dd_out, edid, ed_in);

    NodeTemplate {
        id: Uuid::new_v4(),
        name: "Quest Giver".to_string(),
        description: "NPC quest offer with accept/decline".to_string(),
        is_builtin: true,
        nodes: graph.nodes,
        connections: graph.connections,
    }
}
