use std::collections::HashSet;
use uuid::Uuid;

use crate::model::connection::Connection;
use crate::model::graph::DialogueGraph;
use crate::model::node::NodeType;
use crate::model::port::PortId;

/// Create a flat copy of the graph with all SubGraph nodes expanded.
/// Child nodes are inlined into the parent graph and wired appropriately.
/// The returned graph is suitable for export (flat node array, no SubGraph type).
pub fn flatten_subgraphs(graph: &DialogueGraph) -> DialogueGraph {
    let mut flat = graph.clone();
    // Iteratively expand until no SubGraph nodes remain (handles nesting).
    while let Some(sg_id) = flat
        .nodes
        .values()
        .find(|n| matches!(n.node_type, NodeType::SubGraph(_)))
        .map(|n| n.id)
    {
        expand_one(&mut flat, sg_id);
    }
    flat
}

fn expand_one(graph: &mut DialogueGraph, sg_id: Uuid) {
    let Some(sg_node) = graph.nodes.remove(&sg_id) else {
        return;
    };
    let sg_input = sg_node.inputs.first().map(|p| p.id);
    let sg_output = sg_node.outputs.first().map(|p| p.id);
    let child = match sg_node.node_type {
        NodeType::SubGraph(data) => data.child_graph,
        _ => return,
    };

    // Find the node after the SubGraph (what sg_output connects to).
    let sg_next: Option<(Uuid, PortId)> = sg_output.and_then(|out_port| {
        graph
            .connections
            .iter()
            .find(|c| c.from_port == out_port)
            .map(|c| (c.to_node, c.to_port))
    });

    // Collect incoming connections (parent nodes that connect TO this SubGraph).
    let incoming: Vec<Connection> = match sg_input {
        Some(in_port) => graph
            .connections
            .iter()
            .filter(|c| c.to_port == in_port)
            .cloned()
            .collect(),
        None => vec![],
    };

    // Remove all connections to/from the SubGraph node.
    graph
        .connections
        .retain(|c| c.from_node != sg_id && c.to_node != sg_id);

    // Identify child's Start node and End nodes.
    let child_start_id = child
        .nodes
        .values()
        .find(|n| matches!(n.node_type, NodeType::Start))
        .map(|n| n.id);

    let child_end_ids: HashSet<Uuid> = child
        .nodes
        .values()
        .filter(|n| matches!(n.node_type, NodeType::End(_)))
        .map(|n| n.id)
        .collect();

    // Find the first real node after child Start.
    let child_first: Option<(Uuid, PortId)> = child_start_id.and_then(|start_id| {
        let start = child.nodes.get(&start_id)?;
        let out_port = start.outputs.first()?;
        child
            .connections
            .iter()
            .find(|c| c.from_port == out_port.id)
            .and_then(|c| {
                let target = child.nodes.get(&c.to_node)?;
                let target_in = target.inputs.first()?;
                Some((c.to_node, target_in.id))
            })
    });

    // Insert child nodes (skip Start and End) into parent graph.
    for (id, node) in &child.nodes {
        if child_start_id == Some(*id) {
            continue;
        }
        if child_end_ids.contains(id) {
            continue;
        }
        graph.nodes.insert(*id, node.clone());
    }

    // Insert child connections (skip those from Start or to End).
    for conn in &child.connections {
        let from_is_start = child_start_id == Some(conn.from_node);
        let to_is_end = child_end_ids.contains(&conn.to_node);
        if !from_is_start && !to_is_end {
            graph.connections.push(conn.clone());
        }
    }

    // Rewire: parent incoming → child's first real node.
    if let Some((first_id, first_port)) = child_first {
        for inc in &incoming {
            graph.connections.push(Connection {
                id: Uuid::new_v4(),
                from_node: inc.from_node,
                from_port: inc.from_port,
                to_node: first_id,
                to_port: first_port,
            });
        }
    }

    // Rewire: child connections-to-End → parent's next node.
    if let Some((next_id, next_port)) = sg_next {
        for conn in &child.connections {
            if child_end_ids.contains(&conn.to_node) {
                graph.connections.push(Connection {
                    id: Uuid::new_v4(),
                    from_node: conn.from_node,
                    from_port: conn.from_port,
                    to_node: next_id,
                    to_port: next_port,
                });
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::node::Node;

    #[test]
    fn flatten_no_subgraph_is_identity() {
        let mut graph = DialogueGraph::new();
        let start = Node::new_start([0.0, 0.0]);
        let end = Node::new_end([200.0, 0.0]);
        let s_out = start.outputs[0].id;
        let e_in = end.inputs[0].id;
        let s_id = start.id;
        let e_id = end.id;
        graph.add_node(start);
        graph.add_node(end);
        graph.add_connection(s_id, s_out, e_id, e_in);

        let flat = flatten_subgraphs(&graph);
        assert_eq!(flat.nodes.len(), 2);
        assert_eq!(flat.connections.len(), 1);
    }

    #[test]
    fn flatten_simple_subgraph() {
        let mut graph = DialogueGraph::new();
        let start = Node::new_start([0.0, 0.0]);
        let mut sub = Node::new_subgraph([200.0, 0.0]);
        let end = Node::new_end([400.0, 0.0]);

        // Put a dialogue inside the subgraph
        if let NodeType::SubGraph(ref mut data) = sub.node_type {
            let child_dlg = Node::new_dialogue([100.0, 100.0]);
            let child_start_id = data
                .child_graph
                .nodes
                .values()
                .find(|n| matches!(n.node_type, NodeType::Start))
                .unwrap()
                .id;
            let child_start_out = data.child_graph.nodes[&child_start_id].outputs[0].id;
            let dlg_in = child_dlg.inputs[0].id;
            let dlg_out = child_dlg.outputs[0].id;
            let dlg_id = child_dlg.id;

            let child_end = Node::new_end([200.0, 100.0]);
            let ce_in = child_end.inputs[0].id;
            let ce_id = child_end.id;

            data.child_graph.add_node(child_dlg);
            data.child_graph.add_node(child_end);
            data.child_graph
                .add_connection(child_start_id, child_start_out, dlg_id, dlg_in);
            data.child_graph
                .add_connection(dlg_id, dlg_out, ce_id, ce_in);
        }

        let s_out = start.outputs[0].id;
        let sub_in = sub.inputs[0].id;
        let sub_out = sub.outputs[0].id;
        let e_in = end.inputs[0].id;
        let s_id = start.id;
        let sub_id = sub.id;
        let e_id = end.id;

        graph.add_node(start);
        graph.add_node(sub);
        graph.add_node(end);
        graph.add_connection(s_id, s_out, sub_id, sub_in);
        graph.add_connection(sub_id, sub_out, e_id, e_in);

        let flat = flatten_subgraphs(&graph);

        // SubGraph node should be gone, child dialogue should be inlined
        assert!(
            !flat
                .nodes
                .values()
                .any(|n| matches!(n.node_type, NodeType::SubGraph(_)))
        );
        // start + child_dlg + end (parent) = 3 (child Start and End removed)
        assert_eq!(flat.nodes.len(), 3);
        // start→dlg, dlg→end = 2 connections
        assert_eq!(flat.connections.len(), 2);
    }

    #[test]
    fn flatten_empty_subgraph() {
        let mut graph = DialogueGraph::new();
        let start = Node::new_start([0.0, 0.0]);
        let sub = Node::new_subgraph([200.0, 0.0]);
        let end = Node::new_end([400.0, 0.0]);

        let s_out = start.outputs[0].id;
        let sub_in = sub.inputs[0].id;
        let sub_out = sub.outputs[0].id;
        let e_in = end.inputs[0].id;
        let s_id = start.id;
        let sub_id = sub.id;
        let e_id = end.id;

        graph.add_node(start);
        graph.add_node(sub);
        graph.add_node(end);
        graph.add_connection(s_id, s_out, sub_id, sub_in);
        graph.add_connection(sub_id, sub_out, e_id, e_in);

        let flat = flatten_subgraphs(&graph);
        assert!(
            !flat
                .nodes
                .values()
                .any(|n| matches!(n.node_type, NodeType::SubGraph(_)))
        );
        // start + end remain
        assert_eq!(flat.nodes.len(), 2);
    }
}
