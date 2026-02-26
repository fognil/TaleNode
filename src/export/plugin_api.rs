use crate::model::graph::DialogueGraph;
use crate::model::plugin::PluginManifest;

/// Run an export plugin: reads the template file, substitutes placeholders,
/// and returns the resulting string.
pub fn run_export_plugin(
    plugin: &PluginManifest,
    graph: &DialogueGraph,
    name: &str,
) -> Result<String, String> {
    let template_path = plugin.plugin_dir.join(&plugin.entry_point);
    let template = std::fs::read_to_string(&template_path)
        .map_err(|e| format!("Cannot read template {}: {e}", template_path.display()))?;

    let json = super::json_export::export_json(graph, name)
        .map_err(|e| format!("Export failed: {e}"))?;

    let result = template
        .replace("{{json}}", &json)
        .replace("{{name}}", name)
        .replace("{{version}}", &plugin.version);

    Ok(result)
}

/// Run an import plugin: attempts to parse the input as a JSON DialogueGraph.
pub fn run_import_plugin(
    _plugin: &PluginManifest,
    input: &str,
) -> Result<DialogueGraph, String> {
    serde_json::from_str::<DialogueGraph>(input)
        .map_err(|e| format!("Import parse error: {e}"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn template_substitution() {
        let tmp = std::env::temp_dir().join("talenode_test_plugin_api");
        let _ = std::fs::create_dir_all(&tmp);
        let tmpl_path = tmp.join("template.txt");
        {
            let mut f = std::fs::File::create(&tmpl_path).unwrap();
            write!(f, "name={{{{name}}}} ver={{{{version}}}} data={{{{json}}}}").unwrap();
        }
        let manifest = PluginManifest {
            id: "test".into(),
            name: "Test Plugin".into(),
            version: "2.0".into(),
            author: String::new(),
            description: String::new(),
            plugin_type: crate::model::plugin::PluginType::Export,
            entry_point: "template.txt".into(),
            plugin_dir: tmp.clone(),
        };
        let graph = DialogueGraph::new();
        let result = run_export_plugin(&manifest, &graph, "MyDialogue").unwrap();
        assert!(result.contains("name=MyDialogue"));
        assert!(result.contains("ver=2.0"));
        // JSON is pretty-printed so has spaces
        assert!(result.contains("\"version\": \"1.0\""));
        let _ = std::fs::remove_dir_all(&tmp);
    }

    #[test]
    fn missing_template_error() {
        let manifest = PluginManifest {
            id: "bad".into(),
            name: "Bad".into(),
            version: String::new(),
            author: String::new(),
            description: String::new(),
            plugin_type: crate::model::plugin::PluginType::Export,
            entry_point: "missing.txt".into(),
            plugin_dir: std::env::temp_dir(),
        };
        let graph = DialogueGraph::new();
        let result = run_export_plugin(&manifest, &graph, "test");
        assert!(result.is_err());
    }
}
