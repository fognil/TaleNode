use crate::export::{
    analytics_export, bark_export, document_export, html_export, json_export, locale_export,
    screenplay_export, voice_export, xml_export, yarn_export,
};
use crate::model::project::Project;
use crate::validation::analytics::analyze_graph;

const FORMATS: &[&str] = &[
    "json",
    "xml",
    "yarn",
    "html",
    "markdown",
    "rtf",
    "screenplay",
    "voice-csv",
    "locale-csv",
    "bark-csv",
    "analytics",
];

const USAGE: &str = "\
Usage: talenode export <format> <input.talenode> [-o output]
       talenode export --list
       talenode export --help

Formats: json, xml, yarn, html, markdown, rtf, screenplay,
         voice-csv, locale-csv, bark-csv, analytics

Options:
  -o <path>   Write output to file (default: stdout)
  --list      List available export formats
  --help      Show this help message";

pub fn run_export(args: &[String]) -> Result<(), String> {
    if args.is_empty() || args.iter().any(|a| a == "--help" || a == "-h") {
        println!("{USAGE}");
        return Ok(());
    }

    if args.iter().any(|a| a == "--list") {
        println!("Available export formats:");
        for fmt in FORMATS {
            println!("  {fmt}");
        }
        return Ok(());
    }

    if args.len() < 2 {
        return Err(format!(
            "Expected: talenode export <format> <input.talenode> [-o output]\n\n{USAGE}"
        ));
    }

    let format = &args[0];
    let input_path = &args[1];

    let output_path = args
        .windows(2)
        .find(|w| w[0] == "-o")
        .map(|w| w[1].clone());

    if !FORMATS.contains(&format.as_str()) {
        return Err(format!(
            "Unknown format: '{format}'\nAvailable: {}",
            FORMATS.join(", ")
        ));
    }

    let content = std::fs::read_to_string(input_path)
        .map_err(|e| format!("Failed to read '{input_path}': {e}"))?;

    let project = Project::load_from_string(&content)
        .map_err(|e| format!("Failed to parse '{input_path}': {e}"))?;

    let graph = &project.graph;
    let name = &project.name;

    let output = match format.as_str() {
        "json" => json_export::export_json(graph, name)
            .map_err(|e| format!("JSON export error: {e}"))?,
        "xml" => xml_export::export_xml(graph, name)?,
        "yarn" => yarn_export::export_yarn(graph),
        "html" => html_export::export_html(graph, name),
        "markdown" => document_export::export_markdown(graph, name),
        "rtf" => document_export::export_rtf(graph, name),
        "screenplay" => screenplay_export::export_screenplay(graph, name),
        "voice-csv" => voice_export::export_voice_csv(graph, name),
        "locale-csv" => locale_export::export_locale_csv(graph),
        "bark-csv" => bark_export::export_bark_csv(graph),
        "analytics" => {
            let stats = analyze_graph(graph);
            analytics_export::export_analytics_text(&stats, name)
        }
        _ => unreachable!(),
    };

    if let Some(path) = output_path {
        std::fs::write(&path, &output)
            .map_err(|e| format!("Failed to write '{path}': {e}"))?;
        eprintln!("Exported {format} -> {path}");
    } else {
        print!("{output}");
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn help_flag() {
        assert!(run_export(&["--help".to_string()]).is_ok());
    }

    #[test]
    fn list_flag() {
        assert!(run_export(&["--list".to_string()]).is_ok());
    }

    #[test]
    fn missing_args_returns_error() {
        let result = run_export(&["json".to_string()]);
        assert!(result.is_err());
    }

    #[test]
    fn unknown_format_returns_error() {
        let result = run_export(&["docx".to_string(), "test.talenode".to_string()]);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Unknown format"));
    }

    #[test]
    fn nonexistent_file_returns_error() {
        let result = run_export(&["json".to_string(), "nonexistent.talenode".to_string()]);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Failed to read"));
    }

    #[test]
    fn export_json_to_file() {
        let mut project = Project::default();
        project.name = "CLI Test".to_string();
        let json = project.save_to_string().unwrap();

        let tmp = std::env::temp_dir().join("talenode_cli_test.talenode");
        std::fs::write(&tmp, &json).unwrap();

        let out = std::env::temp_dir().join("talenode_cli_test_out.json");
        let result = run_export(&[
            "json".to_string(),
            tmp.to_string_lossy().to_string(),
            "-o".to_string(),
            out.to_string_lossy().to_string(),
        ]);
        assert!(result.is_ok());
        assert!(out.exists());
        let content = std::fs::read_to_string(&out).unwrap();
        assert!(content.contains("CLI Test"));

        std::fs::remove_file(&tmp).ok();
        std::fs::remove_file(&out).ok();
    }

    #[test]
    fn export_all_formats() {
        let project = Project::default();
        let json = project.save_to_string().unwrap();
        let tmp = std::env::temp_dir().join("talenode_cli_all_fmts.talenode");
        std::fs::write(&tmp, &json).unwrap();
        let input = tmp.to_string_lossy().to_string();

        for fmt in FORMATS {
            let out_path = std::env::temp_dir().join(format!("talenode_cli_{fmt}.out"));
            let result = run_export(&[
                fmt.to_string(),
                input.clone(),
                "-o".to_string(),
                out_path.to_string_lossy().to_string(),
            ]);
            assert!(result.is_ok(), "Format '{fmt}' failed: {:?}", result);
            assert!(out_path.exists(), "Output missing for format '{fmt}'");
            std::fs::remove_file(&out_path).ok();
        }
        std::fs::remove_file(&tmp).ok();
    }
}
