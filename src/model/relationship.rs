use serde::{Deserialize, Serialize};

/// A named relationship/affinity value for a character.
/// Used for reputation, friendship, romance, fear, etc.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Relationship {
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub value: i32,
    #[serde(default = "default_min")]
    pub min: i32,
    #[serde(default = "default_max")]
    pub max: i32,
}

fn default_min() -> i32 {
    -100
}

fn default_max() -> i32 {
    100
}

impl Relationship {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            value: 0,
            min: -100,
            max: 100,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn relationship_defaults() {
        let r = Relationship::new("Friendship");
        assert_eq!(r.name, "Friendship");
        assert_eq!(r.value, 0);
        assert_eq!(r.min, -100);
        assert_eq!(r.max, 100);
    }

    #[test]
    fn relationship_roundtrip() {
        let mut r = Relationship::new("Trust");
        r.value = 50;
        r.min = 0;
        r.max = 200;
        let json = serde_json::to_string(&r).unwrap();
        let loaded: Relationship = serde_json::from_str(&json).unwrap();
        assert_eq!(loaded.name, "Trust");
        assert_eq!(loaded.value, 50);
        assert_eq!(loaded.min, 0);
        assert_eq!(loaded.max, 200);
    }

    #[test]
    fn backward_compat_missing_fields() {
        let json = r#"{"name":"Fear"}"#;
        let loaded: Relationship = serde_json::from_str(json).unwrap();
        assert_eq!(loaded.name, "Fear");
        assert_eq!(loaded.value, 0);
        assert_eq!(loaded.min, -100);
        assert_eq!(loaded.max, 100);
    }
}
