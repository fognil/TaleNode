use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Review status for a node in the writer→reviewer→approved pipeline.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum ReviewStatus {
    #[default]
    Draft,
    NeedsReview,
    Approved,
}

impl ReviewStatus {
    /// Human-readable label for display.
    pub fn label(self) -> &'static str {
        match self {
            Self::Draft => "Draft",
            Self::NeedsReview => "Needs Review",
            Self::Approved => "Approved",
        }
    }

    /// All possible statuses, for iteration in UI.
    pub fn all() -> &'static [ReviewStatus] {
        &[Self::Draft, Self::NeedsReview, Self::Approved]
    }
}

/// A text comment attached to a node.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeComment {
    pub id: Uuid,
    pub node_id: Uuid,
    #[serde(default)]
    pub author: String,
    pub text: String,
}

impl NodeComment {
    pub fn new(node_id: Uuid, text: String) -> Self {
        Self {
            id: Uuid::new_v4(),
            node_id,
            author: String::new(),
            text,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_is_draft() {
        assert_eq!(ReviewStatus::default(), ReviewStatus::Draft);
    }

    #[test]
    fn all_returns_three() {
        assert_eq!(ReviewStatus::all().len(), 3);
    }

    #[test]
    fn serialization_roundtrip() {
        let comment = NodeComment::new(Uuid::new_v4(), "Fix this line".to_string());
        let json = serde_json::to_string(&comment).unwrap();
        let loaded: NodeComment = serde_json::from_str(&json).unwrap();
        assert_eq!(loaded.id, comment.id);
        assert_eq!(loaded.text, "Fix this line");
    }

    #[test]
    fn review_status_roundtrip() {
        for status in ReviewStatus::all() {
            let json = serde_json::to_string(status).unwrap();
            let loaded: ReviewStatus = serde_json::from_str(&json).unwrap();
            assert_eq!(loaded, *status);
        }
    }
}
