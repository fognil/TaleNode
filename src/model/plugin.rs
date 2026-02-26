use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

/// Type of plugin capability.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum PluginType {
    #[default]
    Export,
    Import,
    ExportImport,
}

impl PluginType {
    pub fn label(self) -> &'static str {
        match self {
            Self::Export => "Export",
            Self::Import => "Import",
            Self::ExportImport => "Export + Import",
        }
    }
}

/// Metadata parsed from a plugin's `plugin.json` manifest.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginManifest {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub version: String,
    #[serde(default)]
    pub author: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub plugin_type: PluginType,
    #[serde(default)]
    pub entry_point: String,
    /// Resolved directory containing this plugin (not serialized in plugin.json).
    #[serde(skip)]
    pub plugin_dir: PathBuf,
}

/// Load a single plugin manifest from a directory.
pub fn load_manifest(dir: &Path) -> Result<PluginManifest, String> {
    let manifest_path = dir.join("plugin.json");
    let content = std::fs::read_to_string(&manifest_path)
        .map_err(|e| format!("Cannot read {}: {e}", manifest_path.display()))?;
    let mut manifest: PluginManifest = serde_json::from_str(&content)
        .map_err(|e| format!("Invalid plugin.json in {}: {e}", dir.display()))?;
    manifest.plugin_dir = dir.to_path_buf();
    Ok(manifest)
}

/// Discover plugins from a list of directories.
/// Each directory is scanned for subdirectories containing `plugin.json`.
pub fn discover_plugins(dirs: &[PathBuf]) -> Vec<PluginManifest> {
    let mut plugins = Vec::new();
    for dir in dirs {
        let Ok(entries) = std::fs::read_dir(dir) else { continue };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() && path.join("plugin.json").exists() {
                if let Ok(manifest) = load_manifest(&path) {
                    plugins.push(manifest);
                }
            }
        }
    }
    plugins
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deserialize_manifest() {
        let json = r#"{
            "id": "my-plugin",
            "name": "My Plugin",
            "version": "1.0",
            "author": "Test",
            "description": "A test plugin",
            "plugin_type": "Export",
            "entry_point": "template.txt"
        }"#;
        let m: PluginManifest = serde_json::from_str(json).unwrap();
        assert_eq!(m.id, "my-plugin");
        assert_eq!(m.name, "My Plugin");
        assert_eq!(m.plugin_type, PluginType::Export);
        assert_eq!(m.entry_point, "template.txt");
    }

    #[test]
    fn plugin_type_labels() {
        assert_eq!(PluginType::Export.label(), "Export");
        assert_eq!(PluginType::Import.label(), "Import");
        assert_eq!(PluginType::ExportImport.label(), "Export + Import");
    }

    #[test]
    fn discover_empty_dir() {
        let tmp = std::env::temp_dir().join("talenode_test_discover_empty");
        let _ = std::fs::create_dir_all(&tmp);
        let plugins = discover_plugins(&[tmp.clone()]);
        assert!(plugins.is_empty());
        let _ = std::fs::remove_dir_all(&tmp);
    }
}
