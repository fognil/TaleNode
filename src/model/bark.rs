use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::node_types::VariableValue;

/// A single bark/ambient dialogue line that a character can say
/// outside of the main dialogue tree (e.g., idle chatter, combat barks).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BarkLine {
    pub id: Uuid,
    #[serde(default)]
    pub text: String,
    /// Optional variable name to check before playing this bark.
    #[serde(default)]
    pub condition_variable: Option<String>,
    /// Optional value to compare against the condition variable.
    #[serde(default)]
    pub condition_value: Option<VariableValue>,
    /// Relative weight for random selection (higher = more likely).
    #[serde(default = "default_weight")]
    pub weight: f32,
}

fn default_weight() -> f32 {
    1.0
}

impl BarkLine {
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            id: Uuid::new_v4(),
            text: text.into(),
            condition_variable: None,
            condition_value: None,
            weight: 1.0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bark_line_defaults() {
        let b = BarkLine::new("Hey there!");
        assert_eq!(b.text, "Hey there!");
        assert_eq!(b.weight, 1.0);
        assert!(b.condition_variable.is_none());
        assert!(b.condition_value.is_none());
    }

    #[test]
    fn bark_line_ids_unique() {
        let a = BarkLine::new("a");
        let b = BarkLine::new("b");
        assert_ne!(a.id, b.id);
    }

    #[test]
    fn bark_line_roundtrip() {
        let mut b = BarkLine::new("Watch out!");
        b.weight = 2.5;
        b.condition_variable = Some("alert_level".to_string());
        b.condition_value = Some(VariableValue::Int(3));
        let json = serde_json::to_string(&b).unwrap();
        let loaded: BarkLine = serde_json::from_str(&json).unwrap();
        assert_eq!(loaded.text, "Watch out!");
        assert_eq!(loaded.weight, 2.5);
        assert_eq!(loaded.condition_variable.as_deref(), Some("alert_level"));
    }

    #[test]
    fn backward_compat_missing_fields() {
        let json = r#"{"id":"00000000-0000-0000-0000-000000000001","text":"Hi"}"#;
        let loaded: BarkLine = serde_json::from_str(json).unwrap();
        assert_eq!(loaded.weight, 1.0);
        assert!(loaded.condition_variable.is_none());
    }
}
