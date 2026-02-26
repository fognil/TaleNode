use crate::model::project::Project;
use crate::validation::analytics::analyze_graph;

const USAGE: &str = "\
Usage: talenode info <input.talenode>
       talenode info --help

Shows project statistics: node counts, connections, characters,
variables, locales, and path analysis.";

pub fn run_info(args: &[String]) -> Result<(), String> {
    if args.is_empty() || args.iter().any(|a| a == "--help" || a == "-h") {
        println!("{USAGE}");
        return Ok(());
    }

    let input_path = &args[0];

    let content = std::fs::read_to_string(input_path)
        .map_err(|e| format!("Failed to read '{input_path}': {e}"))?;

    let project = Project::load_from_string(&content)
        .map_err(|e| format!("Failed to parse '{input_path}': {e}"))?;

    let graph = &project.graph;
    let stats = analyze_graph(graph);

    println!("Project: {}", project.name);
    println!("Version: {}", project.version);
    println!();
    println!("Nodes:       {}", stats.total_nodes);
    for (ntype, count) in &stats.nodes_by_type {
        println!("  {ntype}: {count}");
    }
    println!("Connections: {}", stats.total_connections);
    println!("Characters:  {}", graph.characters.len());
    println!("Variables:   {}", graph.variables.len());

    if graph.locale.has_extra_locales() {
        let mut locales = vec![graph.locale.default_locale.clone()];
        locales.extend(graph.locale.extra_locales.clone());
        println!("Locales:     {}", locales.join(", "));
    }

    if !graph.groups.is_empty() {
        println!("Groups:      {}", graph.groups.len());
    }

    println!();
    println!("Paths:       {}", stats.total_paths);
    if stats.total_paths > 0 {
        println!("  longest:   {} nodes", stats.longest_path);
        println!("  shortest:  {} nodes", stats.shortest_path);
    }
    println!("Max fan-out: {}", stats.max_fan_out);
    if stats.avg_choices > 0.0 {
        println!("Avg choices: {:.1}", stats.avg_choices);
    }
    if stats.unreachable_count > 0 {
        println!("Unreachable: {}", stats.unreachable_count);
    }
    if stats.dead_end_count > 0 {
        println!("Dead ends:   {}", stats.dead_end_count);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::node::Node;

    fn write_temp_project(project: &Project, name: &str) -> std::path::PathBuf {
        let path = std::env::temp_dir().join(name);
        let json = project.save_to_string().unwrap();
        std::fs::write(&path, json).unwrap();
        path
    }

    #[test]
    fn help_flag() {
        assert!(run_info(&["--help".to_string()]).is_ok());
    }

    #[test]
    fn nonexistent_file_returns_error() {
        let result = run_info(&["nonexistent.talenode".to_string()]);
        assert!(result.is_err());
    }

    #[test]
    fn info_shows_stats() {
        let mut project = Project::default();
        project.name = "Test Info".to_string();
        let start = Node::new_start([0.0, 0.0]);
        let end = Node::new_end([200.0, 0.0]);
        let (s_out, e_in) = (start.outputs[0].id, end.inputs[0].id);
        let (s_id, e_id) = (start.id, end.id);
        project.graph.add_node(start);
        project.graph.add_node(end);
        project.graph.add_connection(s_id, s_out, e_id, e_in);

        let path = write_temp_project(&project, "talenode_info_test.talenode");
        let result = run_info(&[path.to_string_lossy().to_string()]);
        assert!(result.is_ok());
        std::fs::remove_file(&path).ok();
    }

    #[test]
    fn info_empty_project() {
        let project = Project::default();
        let path = write_temp_project(&project, "talenode_info_empty.talenode");
        let result = run_info(&[path.to_string_lossy().to_string()]);
        assert!(result.is_ok());
        std::fs::remove_file(&path).ok();
    }
}
