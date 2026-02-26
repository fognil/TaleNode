use serde::Serialize;

/// Exported dialogue JSON for game engines.
#[derive(Debug, Serialize)]
pub struct ExportedDialogue {
    pub version: String,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_locale: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub locales: Option<Vec<String>>,
    pub variables: Vec<ExportedVariable>,
    pub characters: Vec<ExportedCharacter>,
    pub nodes: Vec<ExportedNode>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub strings: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub barks: Vec<ExportedBarkGroup>,
}

#[derive(Debug, Serialize)]
pub struct ExportedBarkGroup {
    pub character: String,
    pub lines: Vec<ExportedBarkLine>,
}

#[derive(Debug, Serialize)]
pub struct ExportedBarkLine {
    pub text: String,
    pub weight: f32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub condition: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ExportedVariable {
    pub name: String,
    #[serde(rename = "type")]
    pub var_type: String,
    pub default: serde_json::Value,
}

#[derive(Debug, Serialize)]
pub struct ExportedCharacter {
    pub id: String,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub portrait: Option<String>,
    pub color: String,
}

#[derive(Debug, Serialize)]
pub struct ExportedNode {
    pub id: String,
    #[serde(rename = "type")]
    pub node_type: String,
    #[serde(flatten)]
    pub data: serde_json::Value,
}
