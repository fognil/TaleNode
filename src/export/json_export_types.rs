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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn is_zero_f32_true() {
        assert!(is_zero_f32(&0.0));
    }

    #[test]
    fn is_zero_f32_false() {
        assert!(!is_zero_f32(&1.5));
    }

    #[test]
    fn exported_variable_type_renamed() {
        let var = ExportedVariable {
            name: "hp".into(),
            var_type: "int".into(),
            default: serde_json::json!(100),
        };
        let json = serde_json::to_value(&var).unwrap();
        assert!(json.get("type").is_some());
        assert!(json.get("var_type").is_none());
        assert_eq!(json["type"], "int");
    }

    #[test]
    fn exported_node_data_flattened() {
        let node = ExportedNode {
            id: "dlg_1".into(),
            node_type: "dialogue".into(),
            data: serde_json::json!({"speaker": "Alice", "text": "Hello"}),
        };
        let json = serde_json::to_value(&node).unwrap();
        assert_eq!(json["id"], "dlg_1");
        assert_eq!(json["type"], "dialogue");
        assert_eq!(json["speaker"], "Alice");
        assert_eq!(json["text"], "Hello");
        // data should not appear as a nested key
        assert!(json.get("data").is_none());
    }

    #[test]
    fn skip_empty_barks() {
        let dialogue = ExportedDialogue {
            version: "1.0".into(),
            name: "Test".into(),
            default_locale: Some("en".into()),
            locales: None,
            variables: vec![],
            characters: vec![],
            nodes: vec![],
            strings: None,
            barks: vec![],
            quests: vec![],
            world_entities: vec![],
            timelines: vec![],
        };
        let json = serde_json::to_value(&dialogue).unwrap();
        assert!(json.get("barks").is_none());
    }

    #[test]
    fn skip_none_locale() {
        let dialogue = ExportedDialogue {
            version: "1.0".into(),
            name: "Test".into(),
            default_locale: None,
            locales: None,
            variables: vec![],
            characters: vec![],
            nodes: vec![],
            strings: None,
            barks: vec![],
            quests: vec![],
            world_entities: vec![],
            timelines: vec![],
        };
        let json = serde_json::to_value(&dialogue).unwrap();
        assert!(json.get("default_locale").is_none());
    }

    #[test]
    fn timeline_step_skip_zero_delay() {
        let step = ExportedTimelineStep {
            action: serde_json::json!({"type": "wait", "seconds": 2.0}),
            delay: 0.0,
        };
        let json = serde_json::to_value(&step).unwrap();
        assert!(json.get("delay").is_none());
    }

    #[test]
    fn timeline_skip_false_loop() {
        let tl = ExportedTimeline {
            name: "Intro".into(),
            description: String::new(),
            steps: vec![],
            loop_playback: false,
        };
        let json = serde_json::to_value(&tl).unwrap();
        assert!(json.get("loop_playback").is_none());
    }

    #[test]
    fn exported_character_skip_empty_relationships() {
        let ch = ExportedCharacter {
            id: "char_1".into(),
            name: "Alice".into(),
            portrait: None,
            color: "#ff0000".into(),
            relationships: vec![],
        };
        let json = serde_json::to_value(&ch).unwrap();
        assert!(json.get("relationships").is_none());
    }
}
