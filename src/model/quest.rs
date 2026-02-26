use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Possible quest states.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum QuestStatus {
    #[default]
    NotStarted,
    InProgress,
    Completed,
    Failed,
}

impl QuestStatus {
    pub const ALL: [QuestStatus; 4] = [
        Self::NotStarted,
        Self::InProgress,
        Self::Completed,
        Self::Failed,
    ];

    pub fn label(self) -> &'static str {
        match self {
            Self::NotStarted => "Not Started",
            Self::InProgress => "In Progress",
            Self::Completed => "Completed",
            Self::Failed => "Failed",
        }
    }
}

/// A single quest objective / task.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Objective {
    pub id: Uuid,
    #[serde(default)]
    pub text: String,
    #[serde(default)]
    pub completed: bool,
    #[serde(default)]
    pub optional: bool,
}

impl Objective {
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            id: Uuid::new_v4(),
            text: text.into(),
            completed: false,
            optional: false,
        }
    }
}

/// A quest with objectives that can be triggered via event nodes.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Quest {
    pub id: Uuid,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub objectives: Vec<Objective>,
    #[serde(default)]
    pub status: QuestStatus,
}

impl Quest {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            id: Uuid::new_v4(),
            name: name.into(),
            description: String::new(),
            objectives: Vec::new(),
            status: QuestStatus::NotStarted,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn quest_defaults() {
        let q = Quest::new("Find the key");
        assert_eq!(q.name, "Find the key");
        assert!(q.description.is_empty());
        assert!(q.objectives.is_empty());
        assert_eq!(q.status, QuestStatus::NotStarted);
    }

    #[test]
    fn quest_ids_unique() {
        let a = Quest::new("a");
        let b = Quest::new("b");
        assert_ne!(a.id, b.id);
    }

    #[test]
    fn objective_defaults() {
        let o = Objective::new("Collect 10 gems");
        assert_eq!(o.text, "Collect 10 gems");
        assert!(!o.completed);
        assert!(!o.optional);
    }

    #[test]
    fn quest_roundtrip() {
        let mut q = Quest::new("Main Quest");
        q.description = "Save the world".to_string();
        q.objectives.push(Objective::new("Find sword"));
        q.status = QuestStatus::InProgress;
        let json = serde_json::to_string(&q).unwrap();
        let loaded: Quest = serde_json::from_str(&json).unwrap();
        assert_eq!(loaded.name, "Main Quest");
        assert_eq!(loaded.objectives.len(), 1);
        assert_eq!(loaded.status, QuestStatus::InProgress);
    }

    #[test]
    fn backward_compat_missing_fields() {
        let json = r#"{"id":"00000000-0000-0000-0000-000000000001","name":"Old"}"#;
        let loaded: Quest = serde_json::from_str(json).unwrap();
        assert_eq!(loaded.status, QuestStatus::NotStarted);
        assert!(loaded.objectives.is_empty());
    }
}
