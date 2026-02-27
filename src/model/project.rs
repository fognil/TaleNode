use serde::{Deserialize, Serialize};

use super::graph::DialogueGraph;
pub use super::version::VersionSnapshot;
use super::version::{format_timestamp, MAX_VERSIONS};

/// The full project file format (.talenode).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Project {
    pub version: String,
    pub name: String,
    pub graph: DialogueGraph,
    #[serde(default)]
    pub versions: Vec<VersionSnapshot>,
    #[serde(default)]
    pub dock_layout: Option<serde_json::Value>,
}

impl Default for Project {
    fn default() -> Self {
        Self {
            version: "1.0".to_string(),
            name: "Untitled".to_string(),
            graph: DialogueGraph::new(),
            versions: Vec::new(),
            dock_layout: None,
        }
    }
}

impl Project {
    pub fn save_to_string(&self) -> Result<String, serde_json::Error> {
        let mut normalized = self.clone();
        normalized.graph.normalize();
        for v in &mut normalized.versions {
            v.graph.normalize();
        }
        serde_json::to_string_pretty(&normalized)
    }

    /// Save with versions split into a separate sidecar string.
    /// Returns `(main_json, Some(versions_json))` or `(main_json, None)`.
    pub fn save_split(&self) -> Result<(String, Option<String>), serde_json::Error> {
        let mut normalized = self.clone();
        normalized.graph.normalize();
        for v in &mut normalized.versions {
            v.graph.normalize();
        }
        let versions = std::mem::take(&mut normalized.versions);
        let main_json = serde_json::to_string_pretty(&normalized)?;
        let versions_json = if versions.is_empty() {
            None
        } else {
            Some(serde_json::to_string_pretty(&versions)?)
        };
        Ok((main_json, versions_json))
    }

    /// Merge externally loaded version snapshots into this project.
    pub fn merge_versions(&mut self, json: &str) -> Result<(), serde_json::Error> {
        let versions: Vec<VersionSnapshot> = serde_json::from_str(json)?;
        self.versions = versions;
        Ok(())
    }

    pub fn load_from_string(json: &str) -> Result<Self, serde_json::Error> {
        let mut project: Self = serde_json::from_str(json)?;
        project.graph.rebuild_connection_index();
        Ok(project)
    }

    /// Create a version snapshot of the current graph.
    pub fn create_version(&mut self, description: String) {
        let id = self.versions.last().map_or(1, |v| v.id + 1);
        let timestamp = format_timestamp();
        self.versions.push(VersionSnapshot {
            id,
            timestamp,
            description,
            graph: self.graph.clone(),
        });
        // Trim oldest if over limit
        if self.versions.len() > MAX_VERSIONS {
            self.versions.remove(0);
        }
    }

    #[cfg(test)]
    pub fn list_versions(&self) -> &[VersionSnapshot] {
        &self.versions
    }

    /// Restore graph from a version snapshot. Returns the old graph for undo.
    pub fn restore_version(&mut self, version_id: usize) -> Option<DialogueGraph> {
        let snapshot = self.versions.iter().find(|v| v.id == version_id)?;
        let old = self.graph.clone();
        self.graph = snapshot.graph.clone();
        Some(old)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::character::Character;
    use crate::model::node::*;
    use crate::model::variable::Variable;

    #[test]
    fn save_load_roundtrip() {
        let mut project = Project::default();
        project.name = "Test Project".to_string();
        project.graph.add_node(Node::new_start([100.0, 200.0]));
        project.graph.add_node(Node::new_dialogue([300.0, 200.0]));

        let json = project.save_to_string().unwrap();
        let loaded = Project::load_from_string(&json).unwrap();

        assert_eq!(loaded.name, "Test Project");
        assert_eq!(loaded.graph.nodes.len(), 2);
    }

    #[test]
    fn default_project_values() {
        let p = Project::default();
        assert_eq!(p.version, "1.0");
        assert_eq!(p.name, "Untitled");
        assert!(p.graph.nodes.is_empty());
        assert!(p.graph.connections.is_empty());
    }

    #[test]
    fn load_invalid_json_returns_err() {
        assert!(Project::load_from_string("not json").is_err());
    }

    #[test]
    fn load_empty_string_returns_err() {
        assert!(Project::load_from_string("").is_err());
    }

    #[test]
    fn roundtrip_with_variables_and_characters() {
        let mut project = Project::default();
        project.graph.variables.push(Variable::new_bool("flag", true));
        project
            .graph
            .characters
            .push(Character::new("Merchant"));

        let json = project.save_to_string().unwrap();
        let loaded = Project::load_from_string(&json).unwrap();
        assert_eq!(loaded.graph.variables.len(), 1);
        assert_eq!(loaded.graph.characters.len(), 1);
        assert_eq!(loaded.graph.characters[0].name, "Merchant");
    }

    #[test]
    fn backward_compat_missing_optional_fields() {
        // Minimal JSON with no variables, characters, or groups
        let json = r#"{
            "version": "1.0",
            "name": "Old Project",
            "graph": {
                "nodes": {},
                "connections": []
            }
        }"#;
        let loaded = Project::load_from_string(json).unwrap();
        assert_eq!(loaded.name, "Old Project");
        assert!(loaded.graph.variables.is_empty());
        assert!(loaded.graph.characters.is_empty());
        assert!(loaded.graph.groups.is_empty());
    }

    #[test]
    fn create_and_restore_version() {
        let mut project = Project::default();
        project.graph.add_node(Node::new_start([0.0, 0.0]));
        project.create_version("v1".to_string());
        assert_eq!(project.list_versions().len(), 1);
        assert_eq!(project.list_versions()[0].description, "v1");

        // Modify graph
        project.graph.add_node(Node::new_dialogue([100.0, 0.0]));
        assert_eq!(project.graph.nodes.len(), 2);

        // Restore to v1
        let old = project.restore_version(1).unwrap();
        assert_eq!(old.nodes.len(), 2); // returned old graph
        assert_eq!(project.graph.nodes.len(), 1); // restored
    }

    #[test]
    fn version_max_limit() {
        let mut project = Project::default();
        for i in 0..25 {
            project.create_version(format!("v{i}"));
        }
        assert_eq!(project.versions.len(), 20);
        // Oldest were trimmed
        assert_eq!(project.versions[0].description, "v5");
    }

    #[test]
    fn restore_nonexistent_version_returns_none() {
        let mut project = Project::default();
        assert!(project.restore_version(999).is_none());
    }

    #[test]
    fn versions_serialize_roundtrip() {
        let mut project = Project::default();
        project.graph.add_node(Node::new_start([0.0, 0.0]));
        project.create_version("snapshot".to_string());
        let json = project.save_to_string().unwrap();
        let loaded = Project::load_from_string(&json).unwrap();
        assert_eq!(loaded.versions.len(), 1);
        assert_eq!(loaded.versions[0].graph.nodes.len(), 1);
    }

    #[test]
    fn save_split_separates_versions() {
        let mut project = Project::default();
        project.graph.add_node(Node::new_start([0.0, 0.0]));
        project.create_version("v1".to_string());
        let (main_json, versions_json) = project.save_split().unwrap();
        // Main JSON has empty versions
        let loaded = Project::load_from_string(&main_json).unwrap();
        assert!(loaded.versions.is_empty());
        // Sidecar has the versions
        let vj = versions_json.unwrap();
        let mut full = loaded;
        full.merge_versions(&vj).unwrap();
        assert_eq!(full.versions.len(), 1);
        assert_eq!(full.versions[0].description, "v1");
    }

    #[test]
    fn save_split_no_versions_returns_none() {
        let project = Project::default();
        let (_main, versions) = project.save_split().unwrap();
        assert!(versions.is_none());
    }

    #[test]
    fn deterministic_serialization() {
        let mut project = Project::default();
        project.graph.variables.push(Variable::new_bool("zzz", false));
        project.graph.variables.push(Variable::new_bool("aaa", true));
        project.graph.add_node(Node::new_start([0.0, 0.0]));
        let json1 = project.save_to_string().unwrap();
        let json2 = project.save_to_string().unwrap();
        assert_eq!(json1, json2);
        // Variables should be sorted by name
        let idx_a = json1.find("\"aaa\"").unwrap();
        let idx_z = json1.find("\"zzz\"").unwrap();
        assert!(idx_a < idx_z, "variables should be sorted by name");
    }
}
