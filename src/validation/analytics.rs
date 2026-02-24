use std::collections::{HashMap, HashSet, VecDeque};
use uuid::Uuid;

use crate::model::graph::DialogueGraph;
use crate::model::node::NodeType;

/// Summary statistics for a dialogue graph.
#[derive(Debug, Clone, Default)]
pub struct GraphAnalytics {
    pub total_nodes: usize,
    pub nodes_by_type: HashMap<&'static str, usize>,
    pub total_connections: usize,
    pub total_paths: usize,
    pub longest_path: usize,
    pub shortest_path: usize,
    pub max_fan_out: usize,
    pub avg_choices: f32,
    pub unreachable_count: usize,
    pub dead_end_count: usize,
}

/// Analyze a dialogue graph and return summary statistics.
pub fn analyze_graph(graph: &DialogueGraph) -> GraphAnalytics {
    let mut stats = GraphAnalytics {
        total_nodes: graph.nodes.len(),
        total_connections: graph.connections.len(),
        ..Default::default()
    };

    count_nodes_by_type(graph, &mut stats);

    let adjacency = build_adjacency(graph);
    let reachable = find_reachable(graph, &adjacency);

    stats.unreachable_count = graph
        .nodes
        .values()
        .filter(|n| !matches!(n.node_type, NodeType::Start))
        .filter(|n| !reachable.contains(&n.id))
        .count();

    count_dead_ends(graph, &mut stats);
    compute_fan_out(graph, &adjacency, &mut stats);
    compute_avg_choices(graph, &mut stats);
    count_paths(graph, &adjacency, &mut stats);

    stats
}

fn count_nodes_by_type(graph: &DialogueGraph, stats: &mut GraphAnalytics) {
    for node in graph.nodes.values() {
        let type_name = match &node.node_type {
            NodeType::Start => "Start",
            NodeType::Dialogue(_) => "Dialogue",
            NodeType::Choice(_) => "Choice",
            NodeType::Condition(_) => "Condition",
            NodeType::Event(_) => "Event",
            NodeType::Random(_) => "Random",
            NodeType::End(_) => "End",
        };
        *stats.nodes_by_type.entry(type_name).or_insert(0) += 1;
    }
}

fn build_adjacency(graph: &DialogueGraph) -> HashMap<Uuid, Vec<Uuid>> {
    let mut adj: HashMap<Uuid, Vec<Uuid>> = HashMap::new();
    for conn in &graph.connections {
        adj.entry(conn.from_node).or_default().push(conn.to_node);
    }
    adj
}

fn find_reachable(graph: &DialogueGraph, adjacency: &HashMap<Uuid, Vec<Uuid>>) -> HashSet<Uuid> {
    let mut reachable = HashSet::new();
    let mut queue = VecDeque::new();
    for node in graph.nodes.values() {
        if matches!(node.node_type, NodeType::Start) {
            queue.push_back(node.id);
            reachable.insert(node.id);
        }
    }
    while let Some(id) = queue.pop_front() {
        if let Some(neighbors) = adjacency.get(&id) {
            for &next in neighbors {
                if reachable.insert(next) {
                    queue.push_back(next);
                }
            }
        }
    }
    reachable
}

fn count_dead_ends(graph: &DialogueGraph, stats: &mut GraphAnalytics) {
    let nodes_with_output: HashSet<Uuid> = graph
        .connections
        .iter()
        .map(|c| c.from_node)
        .collect();
    stats.dead_end_count = graph
        .nodes
        .values()
        .filter(|n| !matches!(n.node_type, NodeType::End(_)))
        .filter(|n| !n.outputs.is_empty())
        .filter(|n| !nodes_with_output.contains(&n.id))
        .count();
}

fn compute_fan_out(
    graph: &DialogueGraph,
    adjacency: &HashMap<Uuid, Vec<Uuid>>,
    stats: &mut GraphAnalytics,
) {
    stats.max_fan_out = graph
        .nodes
        .keys()
        .map(|id| adjacency.get(id).map_or(0, |v| v.len()))
        .max()
        .unwrap_or(0);
}

fn compute_avg_choices(graph: &DialogueGraph, stats: &mut GraphAnalytics) {
    let mut total = 0usize;
    let mut count = 0usize;
    for node in graph.nodes.values() {
        if let NodeType::Choice(data) = &node.node_type {
            total += data.choices.len();
            count += 1;
        }
    }
    stats.avg_choices = if count > 0 {
        total as f32 / count as f32
    } else {
        0.0
    };
}

struct PathCounter<'a> {
    adjacency: &'a HashMap<Uuid, Vec<Uuid>>,
    end_ids: HashSet<Uuid>,
    visited: HashSet<Uuid>,
    total: usize,
    longest: usize,
    shortest: usize,
}

impl PathCounter<'_> {
    fn dfs(&mut self, current: Uuid, depth: usize) {
        if self.end_ids.contains(&current) {
            self.total += 1;
            if depth > self.longest { self.longest = depth; }
            if depth < self.shortest { self.shortest = depth; }
            return;
        }
        if !self.visited.insert(current) {
            return;
        }
        if let Some(neighbors) = self.adjacency.get(&current) {
            let neighbors = neighbors.clone();
            for next in neighbors {
                self.dfs(next, depth + 1);
            }
        }
        self.visited.remove(&current);
    }
}

/// DFS path counting from Start nodes to End nodes with cycle protection.
fn count_paths(
    graph: &DialogueGraph,
    adjacency: &HashMap<Uuid, Vec<Uuid>>,
    stats: &mut GraphAnalytics,
) {
    let end_ids: HashSet<Uuid> = graph
        .nodes
        .values()
        .filter(|n| matches!(n.node_type, NodeType::End(_)))
        .map(|n| n.id)
        .collect();

    if end_ids.is_empty() {
        stats.total_paths = 0;
        stats.longest_path = 0;
        stats.shortest_path = 0;
        return;
    }

    let mut counter = PathCounter {
        adjacency,
        end_ids,
        visited: HashSet::new(),
        total: 0,
        longest: 0,
        shortest: usize::MAX,
    };

    for node in graph.nodes.values() {
        if matches!(node.node_type, NodeType::Start) {
            counter.dfs(node.id, 0);
        }
    }

    stats.total_paths = counter.total;
    stats.longest_path = counter.longest;
    stats.shortest_path = if counter.shortest == usize::MAX { 0 } else { counter.shortest };
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::node::Node;

    #[test]
    fn empty_graph() {
        let graph = DialogueGraph::new();
        let stats = analyze_graph(&graph);
        assert_eq!(stats.total_nodes, 0);
        assert_eq!(stats.total_paths, 0);
        assert_eq!(stats.longest_path, 0);
    }

    #[test]
    fn linear_path() {
        let mut graph = DialogueGraph::new();
        let start = Node::new_start([0.0, 0.0]);
        let dlg = Node::new_dialogue([100.0, 0.0]);
        let end = Node::new_end([200.0, 0.0]);
        let s_out = start.outputs[0].id;
        let d_in = dlg.inputs[0].id;
        let d_out = dlg.outputs[0].id;
        let e_in = end.inputs[0].id;
        let (s_id, d_id, e_id) = (start.id, dlg.id, end.id);
        graph.add_node(start);
        graph.add_node(dlg);
        graph.add_node(end);
        graph.add_connection(s_id, s_out, d_id, d_in);
        graph.add_connection(d_id, d_out, e_id, e_in);

        let stats = analyze_graph(&graph);
        assert_eq!(stats.total_paths, 1);
        assert_eq!(stats.longest_path, 2);
        assert_eq!(stats.shortest_path, 2);
        assert_eq!(stats.unreachable_count, 0);
        assert_eq!(stats.dead_end_count, 0);
    }

    #[test]
    fn branching_paths() {
        let mut graph = DialogueGraph::new();
        let start = Node::new_start([0.0, 0.0]);
        let choice = Node::new_choice([100.0, 0.0]);
        let end1 = Node::new_end([200.0, 0.0]);
        let end2 = Node::new_end([200.0, 100.0]);
        let s_out = start.outputs[0].id;
        let c_in = choice.inputs[0].id;
        let c_out0 = choice.outputs[0].id;
        let c_out1 = choice.outputs[1].id;
        let e1_in = end1.inputs[0].id;
        let e2_in = end2.inputs[0].id;
        let (s_id, c_id, e1_id, e2_id) = (start.id, choice.id, end1.id, end2.id);
        graph.add_node(start);
        graph.add_node(choice);
        graph.add_node(end1);
        graph.add_node(end2);
        graph.add_connection(s_id, s_out, c_id, c_in);
        graph.add_connection(c_id, c_out0, e1_id, e1_in);
        graph.add_connection(c_id, c_out1, e2_id, e2_in);

        let stats = analyze_graph(&graph);
        assert_eq!(stats.total_paths, 2);
        assert_eq!(stats.max_fan_out, 2);
    }

    #[test]
    fn node_counts() {
        let mut graph = DialogueGraph::new();
        graph.add_node(Node::new_start([0.0, 0.0]));
        graph.add_node(Node::new_dialogue([100.0, 0.0]));
        graph.add_node(Node::new_choice([200.0, 0.0]));
        graph.add_node(Node::new_end([300.0, 0.0]));

        let stats = analyze_graph(&graph);
        assert_eq!(stats.total_nodes, 4);
        assert_eq!(stats.nodes_by_type.get("Start"), Some(&1));
        assert_eq!(stats.nodes_by_type.get("Dialogue"), Some(&1));
        assert_eq!(stats.nodes_by_type.get("Choice"), Some(&1));
        assert_eq!(stats.nodes_by_type.get("End"), Some(&1));
        assert_eq!(stats.avg_choices, 2.0);
    }
}
