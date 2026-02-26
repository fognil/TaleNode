use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// An action that can occur in a timeline step.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TimelineAction {
    Dialogue { node_id: Option<Uuid> },
    Camera { target: String, duration: f32 },
    Animation { target: String, clip: String },
    Audio { clip: String, volume: f32 },
    Wait { seconds: f32 },
    SetVariable { key: String, value: String },
    Custom { action_type: String, data: String },
}

impl Default for TimelineAction {
    fn default() -> Self {
        Self::Wait { seconds: 1.0 }
    }
}

impl TimelineAction {
    pub const LABELS: &[&str] = &[
        "Dialogue", "Camera", "Animation", "Audio", "Wait", "SetVariable", "Custom",
    ];

    pub fn label(&self) -> &'static str {
        match self {
            Self::Dialogue { .. } => "Dialogue",
            Self::Camera { .. } => "Camera",
            Self::Animation { .. } => "Animation",
            Self::Audio { .. } => "Audio",
            Self::Wait { .. } => "Wait",
            Self::SetVariable { .. } => "SetVariable",
            Self::Custom { .. } => "Custom",
        }
    }

    pub fn from_label(label: &str) -> Self {
        match label {
            "Dialogue" => Self::Dialogue { node_id: None },
            "Camera" => Self::Camera { target: String::new(), duration: 1.0 },
            "Animation" => Self::Animation { target: String::new(), clip: String::new() },
            "Audio" => Self::Audio { clip: String::new(), volume: 1.0 },
            "Wait" => Self::Wait { seconds: 1.0 },
            "SetVariable" => Self::SetVariable { key: String::new(), value: String::new() },
            _ => Self::Custom { action_type: String::new(), data: String::new() },
        }
    }
}

/// A single step in a timeline sequence.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimelineStep {
    pub id: Uuid,
    #[serde(default)]
    pub action: TimelineAction,
    #[serde(default)]
    pub delay: f32,
}

impl TimelineStep {
    pub fn new(action: TimelineAction) -> Self {
        Self {
            id: Uuid::new_v4(),
            action,
            delay: 0.0,
        }
    }
}

/// A named sequence of timeline steps for cutscene/dialogue sequencing.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Timeline {
    pub id: Uuid,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub steps: Vec<TimelineStep>,
    #[serde(default)]
    pub loop_playback: bool,
}

impl Timeline {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            id: Uuid::new_v4(),
            name: name.into(),
            description: String::new(),
            steps: Vec::new(),
            loop_playback: false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn timeline_defaults() {
        let t = Timeline::new("Intro");
        assert_eq!(t.name, "Intro");
        assert!(t.description.is_empty());
        assert!(t.steps.is_empty());
        assert!(!t.loop_playback);
    }

    #[test]
    fn timeline_ids_unique() {
        let a = Timeline::new("a");
        let b = Timeline::new("b");
        assert_ne!(a.id, b.id);
    }

    #[test]
    fn step_defaults() {
        let s = TimelineStep::new(TimelineAction::Wait { seconds: 2.0 });
        assert_eq!(s.delay, 0.0);
        assert_eq!(s.action.label(), "Wait");
    }

    #[test]
    fn action_labels() {
        assert_eq!(TimelineAction::Dialogue { node_id: None }.label(), "Dialogue");
        assert_eq!(TimelineAction::Camera { target: String::new(), duration: 1.0 }.label(), "Camera");
        assert_eq!(TimelineAction::Audio { clip: String::new(), volume: 1.0 }.label(), "Audio");
    }

    #[test]
    fn from_label_roundtrip() {
        for label in TimelineAction::LABELS {
            let action = TimelineAction::from_label(label);
            assert_eq!(action.label(), *label);
        }
    }

    #[test]
    fn timeline_roundtrip() {
        let mut t = Timeline::new("Cutscene 1");
        t.description = "Opening cutscene".to_string();
        t.steps.push(TimelineStep::new(TimelineAction::Camera {
            target: "player".to_string(), duration: 2.0,
        }));
        t.steps.push(TimelineStep::new(TimelineAction::Wait { seconds: 1.0 }));
        t.loop_playback = true;
        let json = serde_json::to_string(&t).unwrap();
        let loaded: Timeline = serde_json::from_str(&json).unwrap();
        assert_eq!(loaded.name, "Cutscene 1");
        assert_eq!(loaded.steps.len(), 2);
        assert!(loaded.loop_playback);
    }

    #[test]
    fn backward_compat_missing_fields() {
        let json = r#"{"id":"00000000-0000-0000-0000-000000000001","name":"Old"}"#;
        let loaded: Timeline = serde_json::from_str(json).unwrap();
        assert!(loaded.steps.is_empty());
        assert!(!loaded.loop_playback);
    }
}
