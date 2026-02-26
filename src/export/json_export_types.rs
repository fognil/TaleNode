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
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub quests: Vec<ExportedQuest>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub world_entities: Vec<ExportedWorldEntity>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub timelines: Vec<ExportedTimeline>,
}

#[derive(Debug, Serialize)]
pub struct ExportedQuest {
    pub name: String,
    #[serde(skip_serializing_if = "String::is_empty")]
    pub description: String,
    pub objectives: Vec<ExportedObjective>,
}

#[derive(Debug, Serialize)]
pub struct ExportedObjective {
    pub text: String,
    pub optional: bool,
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
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub relationships: Vec<ExportedRelationship>,
}

#[derive(Debug, Serialize)]
pub struct ExportedRelationship {
    pub name: String,
    pub default_value: i32,
    pub min: i32,
    pub max: i32,
}

#[derive(Debug, Serialize)]
pub struct ExportedWorldEntity {
    pub name: String,
    pub category: String,
    #[serde(skip_serializing_if = "String::is_empty")]
    pub description: String,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub tags: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub properties: Vec<ExportedEntityProperty>,
}

#[derive(Debug, Serialize)]
pub struct ExportedEntityProperty {
    pub key: String,
    pub value: String,
}

#[derive(Debug, Serialize)]
pub struct ExportedTimeline {
    pub name: String,
    #[serde(skip_serializing_if = "String::is_empty")]
    pub description: String,
    pub steps: Vec<ExportedTimelineStep>,
    #[serde(skip_serializing_if = "std::ops::Not::not")]
    pub loop_playback: bool,
}

#[derive(Debug, Serialize)]
pub struct ExportedTimelineStep {
    pub action: serde_json::Value,
    #[serde(skip_serializing_if = "is_zero_f32")]
    pub delay: f32,
}

fn is_zero_f32(v: &f32) -> bool { *v == 0.0 }

#[derive(Debug, Serialize)]
pub struct ExportedNode {
    pub id: String,
    #[serde(rename = "type")]
    pub node_type: String,
    #[serde(flatten)]
    pub data: serde_json::Value,
}
