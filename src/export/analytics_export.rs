use crate::validation::analytics::GraphAnalytics;

/// Export analytics as CSV.
pub fn export_analytics_csv(stats: &GraphAnalytics, name: &str) -> String {
    let mut csv = String::from("Metric,Value\n");
    csv.push_str(&format!("Project,{name}\n"));
    csv.push_str(&format!("Total Nodes,{}\n", stats.total_nodes));

    // Node counts by type
    let mut types: Vec<_> = stats.nodes_by_type.iter().collect();
    types.sort_by_key(|(k, _)| *k);
    for (type_name, count) in &types {
        csv.push_str(&format!("{type_name},{count}\n"));
    }

    csv.push_str(&format!("Total Connections,{}\n", stats.total_connections));
    csv.push_str(&format!("Total Paths,{}\n", stats.total_paths));

    if stats.total_paths > 0 {
        csv.push_str(&format!("Longest Path,{}\n", stats.longest_path));
        csv.push_str(&format!("Shortest Path,{}\n", stats.shortest_path));
    }

    csv.push_str(&format!("Max Fan-out,{}\n", stats.max_fan_out));

    if stats.avg_choices > 0.0 {
        csv.push_str(&format!("Avg Choices per Choice Node,{:.1}\n", stats.avg_choices));
    }

    csv.push_str(&format!("Unreachable Nodes,{}\n", stats.unreachable_count));
    csv.push_str(&format!("Dead Ends,{}\n", stats.dead_end_count));

    csv
}

/// Export analytics as a human-readable text report.
pub fn export_analytics_text(stats: &GraphAnalytics, name: &str) -> String {
    let mut out = String::new();

    out.push_str("=== TaleNode Analytics Report ===\n");
    out.push_str(&format!("Project: {name}\n"));
    out.push('\n');

    // Node summary
    out.push_str("--- Node Summary ---\n");
    out.push_str(&format!("Total nodes: {}\n", stats.total_nodes));

    if !stats.nodes_by_type.is_empty() {
        let mut parts = Vec::new();
        let mut types: Vec<_> = stats.nodes_by_type.iter().collect();
        types.sort_by_key(|(k, _)| *k);
        for (type_name, count) in &types {
            parts.push(format!("{type_name}: {count}"));
        }
        out.push_str(&format!("  {}\n", parts.join(", ")));
    }

    out.push_str(&format!("Total connections: {}\n", stats.total_connections));
    out.push('\n');

    // Path analysis
    out.push_str("--- Path Analysis ---\n");
    out.push_str(&format!(
        "Total paths (Start->End): {}\n",
        stats.total_paths
    ));
    if stats.total_paths > 0 {
        out.push_str(&format!("Longest: {} steps\n", stats.longest_path));
        out.push_str(&format!("Shortest: {} steps\n", stats.shortest_path));
    }
    out.push('\n');

    // Branching
    out.push_str("--- Branching ---\n");
    out.push_str(&format!("Max fan-out: {}\n", stats.max_fan_out));
    if stats.avg_choices > 0.0 {
        out.push_str(&format!(
            "Avg choices per Choice node: {:.1}\n",
            stats.avg_choices
        ));
    }
    out.push('\n');

    // Connectivity
    out.push_str("--- Connectivity ---\n");
    out.push_str(&format!("Unreachable nodes: {}\n", stats.unreachable_count));
    out.push_str(&format!("Dead ends: {}\n", stats.dead_end_count));

    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::graph::DialogueGraph;
    use crate::model::node::Node;
    use crate::validation::analytics::analyze_graph;

    #[test]
    fn csv_contains_expected_rows() {
        let mut graph = DialogueGraph::new();
        graph.add_node(Node::new_start([0.0, 0.0]));
        graph.add_node(Node::new_dialogue([100.0, 0.0]));
        let stats = analyze_graph(&graph);
        let csv = export_analytics_csv(&stats, "Test");

        assert!(csv.starts_with("Metric,Value\n"));
        assert!(csv.contains("Project,Test\n"));
        assert!(csv.contains("Total Nodes,2\n"));
        assert!(csv.contains("Start,1\n"));
        assert!(csv.contains("Dialogue,1\n"));
    }

    #[test]
    fn text_contains_expected_sections() {
        let mut graph = DialogueGraph::new();
        graph.add_node(Node::new_start([0.0, 0.0]));
        let stats = analyze_graph(&graph);
        let text = export_analytics_text(&stats, "MyProject");

        assert!(text.contains("=== TaleNode Analytics Report ==="));
        assert!(text.contains("Project: MyProject"));
        assert!(text.contains("--- Node Summary ---"));
        assert!(text.contains("--- Path Analysis ---"));
        assert!(text.contains("--- Branching ---"));
        assert!(text.contains("--- Connectivity ---"));
    }

    #[test]
    fn empty_graph_produces_valid_output() {
        let graph = DialogueGraph::new();
        let stats = analyze_graph(&graph);

        let csv = export_analytics_csv(&stats, "Empty");
        assert!(csv.contains("Total Nodes,0"));

        let text = export_analytics_text(&stats, "Empty");
        assert!(text.contains("Total nodes: 0"));
    }
}
