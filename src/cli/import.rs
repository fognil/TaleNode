use crate::import::{articy_import, chatmapper_import, ink_import, yarn_import};
use crate::model::project::Project;

const IMPORT_FORMATS: &[&str] = &["yarn", "ink", "articy", "chatmapper"];

const USAGE: &str = "\
Usage: talenode import <format> <input> [-o output.talenode]
       talenode import --list
       talenode import --help

Formats: yarn, ink, articy, chatmapper

Converts external dialogue formats into a .talenode project file.
Output defaults to stdout if -o is not specified.";

pub fn run_import(args: &[String]) -> Result<(), String> {
    if args.is_empty() || args.iter().any(|a| a == "--help" || a == "-h") {
        println!("{USAGE}");
        return Ok(());
    }

    if args.iter().any(|a| a == "--list") {
        println!("Available import formats:");
        for fmt in IMPORT_FORMATS {
            println!("  {fmt}");
        }
        return Ok(());
    }

    if args.len() < 2 {
        return Err(format!(
            "Expected: talenode import <format> <input> [-o output.talenode]\n\n{USAGE}"
        ));
    }

    let format = &args[0];
    let input_path = &args[1];

    let output_path = args
        .windows(2)
        .find(|w| w[0] == "-o")
        .map(|w| w[1].clone());

    if !IMPORT_FORMATS.contains(&format.as_str()) {
        return Err(format!(
            "Unknown import format: '{format}'\nAvailable: {}",
            IMPORT_FORMATS.join(", ")
        ));
    }

    let content = std::fs::read_to_string(input_path)
        .map_err(|e| format!("Failed to read '{input_path}': {e}"))?;

    let graph = match format.as_str() {
        "yarn" => yarn_import::import_yarn(&content)?,
        "ink" => ink_import::import_ink(&content)?,
        "articy" => articy_import::import_articy(&content)?,
        "chatmapper" => chatmapper_import::import_chatmapper(&content)?,
        _ => unreachable!(),
    };

    let name = std::path::Path::new(input_path)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("Imported")
        .to_string();

    let project = Project {
        version: "1.0".to_string(),
        name,
        graph,
        versions: Vec::new(),
        dock_layout: None,
    };

    let json = project
        .save_to_string()
        .map_err(|e| format!("Failed to serialize project: {e}"))?;

    if let Some(path) = output_path {
        std::fs::write(&path, &json)
            .map_err(|e| format!("Failed to write '{path}': {e}"))?;
        eprintln!("Imported {format} -> {path}");
    } else {
        print!("{json}");
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn help_flag() {
        assert!(run_import(&["--help".to_string()]).is_ok());
    }

    #[test]
    fn list_flag() {
        assert!(run_import(&["--list".to_string()]).is_ok());
    }

    #[test]
    fn missing_args_returns_error() {
        let result = run_import(&["yarn".to_string()]);
        assert!(result.is_err());
    }

    #[test]
    fn unknown_format_returns_error() {
        let result = run_import(&["docx".to_string(), "test.yarn".to_string()]);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Unknown import format"));
    }

    #[test]
    fn nonexistent_file_returns_error() {
        let result = run_import(&["yarn".to_string(), "nonexistent.yarn".to_string()]);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Failed to read"));
    }

    #[test]
    fn import_yarn_to_file() {
        let yarn_content = "\
title: Start
---
Speaker: Hello world!
===
";
        let tmp_in = std::env::temp_dir().join("talenode_import_test.yarn");
        std::fs::write(&tmp_in, yarn_content).unwrap();

        let tmp_out = std::env::temp_dir().join("talenode_import_test.talenode");
        let result = run_import(&[
            "yarn".to_string(),
            tmp_in.to_string_lossy().to_string(),
            "-o".to_string(),
            tmp_out.to_string_lossy().to_string(),
        ]);
        assert!(result.is_ok(), "Import failed: {:?}", result);
        assert!(tmp_out.exists());

        let content = std::fs::read_to_string(&tmp_out).unwrap();
        let loaded = Project::load_from_string(&content).unwrap();
        assert!(!loaded.graph.nodes.is_empty());

        std::fs::remove_file(&tmp_in).ok();
        std::fs::remove_file(&tmp_out).ok();
    }
}
