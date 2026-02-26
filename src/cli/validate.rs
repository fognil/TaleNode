use crate::model::project::Project;
use crate::validation::validator::{validate, Severity};

const USAGE: &str = "\
Usage: talenode validate <input.talenode>
       talenode validate --help

Runs validation checks on a .talenode project file.
Prints errors and warnings. Exits with code 1 if any errors found.";

pub fn run_validate(args: &[String]) -> Result<(), String> {
    if args.is_empty() || args.iter().any(|a| a == "--help" || a == "-h") {
        println!("{USAGE}");
        return Ok(());
    }

    let input_path = &args[0];

    let content = std::fs::read_to_string(input_path)
        .map_err(|e| format!("Failed to read '{input_path}': {e}"))?;

    let project = Project::load_from_string(&content)
        .map_err(|e| format!("Failed to parse '{input_path}': {e}"))?;

    let warnings = validate(&project.graph);

    if warnings.is_empty() {
        println!("No issues found in '{input_path}'.");
        return Ok(());
    }

    let mut error_count = 0;
    let mut warning_count = 0;

    for w in &warnings {
        let prefix = match w.severity {
            Severity::Error => {
                error_count += 1;
                "ERROR"
            }
            Severity::Warning => {
                warning_count += 1;
                "WARN"
            }
        };
        let node_info = w
            .node_id
            .map(|id| format!(" [node {}]", &id.to_string()[..8]))
            .unwrap_or_default();
        println!("  {prefix}{node_info}: {}", w.message);
    }

    println!("\n{error_count} error(s), {warning_count} warning(s)");

    if error_count > 0 {
        Err(format!("Validation failed with {error_count} error(s)"))
    } else {
        Ok(())
    }
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
        assert!(run_validate(&["--help".to_string()]).is_ok());
    }

    #[test]
    fn nonexistent_file_returns_error() {
        let result = run_validate(&["nonexistent.talenode".to_string()]);
        assert!(result.is_err());
    }

    #[test]
    fn valid_graph_no_errors() {
        let mut project = Project::default();
        let start = Node::new_start([0.0, 0.0]);
        let end = Node::new_end([200.0, 0.0]);
        let (s_out, e_in) = (start.outputs[0].id, end.inputs[0].id);
        let (s_id, e_id) = (start.id, end.id);
        project.graph.add_node(start);
        project.graph.add_node(end);
        project.graph.add_connection(s_id, s_out, e_id, e_in);

        let path = write_temp_project(&project, "talenode_validate_ok.talenode");
        let result = run_validate(&[path.to_string_lossy().to_string()]);
        assert!(result.is_ok());
        std::fs::remove_file(&path).ok();
    }

    #[test]
    fn empty_graph_reports_issues() {
        let project = Project::default();
        let path = write_temp_project(&project, "talenode_validate_empty.talenode");
        // Empty graph has no start node -> error
        let result = run_validate(&[path.to_string_lossy().to_string()]);
        assert!(result.is_err());
        std::fs::remove_file(&path).ok();
    }
}
