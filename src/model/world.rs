use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Category of a world entity.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum EntityCategory {
    #[default]
    Item,
    Location,
    Lore,
    Character,
    Custom(String),
}

impl EntityCategory {
    pub const ALL: [EntityCategory; 4] = [
        Self::Item,
        Self::Location,
        Self::Lore,
        Self::Character,
    ];

    pub fn label(&self) -> &str {
        match self {
            Self::Item => "Item",
            Self::Location => "Location",
            Self::Lore => "Lore",
            Self::Character => "Character",
            Self::Custom(s) => s.as_str(),
        }
    }
}

/// A key-value property on a world entity.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntityProperty {
    #[serde(default)]
    pub key: String,
    #[serde(default)]
    pub value: String,
}

/// A world-building entity (item, location, lore entry, etc.).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorldEntity {
    pub id: Uuid,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub category: EntityCategory,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub properties: Vec<EntityProperty>,
    #[serde(default)]
    pub icon: Option<String>,
}

impl WorldEntity {
    pub fn new(name: impl Into<String>, category: EntityCategory) -> Self {
        Self {
            id: Uuid::new_v4(),
            name: name.into(),
            category,
            description: String::new(),
            tags: Vec::new(),
            properties: Vec::new(),
            icon: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn entity_defaults() {
        let e = WorldEntity::new("Sword", EntityCategory::Item);
        assert_eq!(e.name, "Sword");
        assert_eq!(e.category, EntityCategory::Item);
        assert!(e.description.is_empty());
        assert!(e.tags.is_empty());
        assert!(e.properties.is_empty());
        assert!(e.icon.is_none());
    }

    #[test]
    fn entity_ids_unique() {
        let a = WorldEntity::new("a", EntityCategory::Item);
        let b = WorldEntity::new("b", EntityCategory::Location);
        assert_ne!(a.id, b.id);
    }

    #[test]
    fn category_labels() {
        assert_eq!(EntityCategory::Item.label(), "Item");
        assert_eq!(EntityCategory::Location.label(), "Location");
        assert_eq!(EntityCategory::Lore.label(), "Lore");
        assert_eq!(EntityCategory::Character.label(), "Character");
        assert_eq!(EntityCategory::Custom("Weapon".into()).label(), "Weapon");
    }

    #[test]
    fn entity_roundtrip() {
        let mut e = WorldEntity::new("Tavern", EntityCategory::Location);
        e.description = "A cozy tavern".to_string();
        e.tags.push("social".to_string());
        e.properties.push(EntityProperty {
            key: "capacity".to_string(),
            value: "30".to_string(),
        });
        let json = serde_json::to_string(&e).unwrap();
        let loaded: WorldEntity = serde_json::from_str(&json).unwrap();
        assert_eq!(loaded.name, "Tavern");
        assert_eq!(loaded.properties.len(), 1);
        assert_eq!(loaded.properties[0].key, "capacity");
    }

    #[test]
    fn backward_compat_missing_fields() {
        let json = r#"{"id":"00000000-0000-0000-0000-000000000001","name":"Old"}"#;
        let loaded: WorldEntity = serde_json::from_str(json).unwrap();
        assert_eq!(loaded.category, EntityCategory::Item);
        assert!(loaded.tags.is_empty());
        assert!(loaded.properties.is_empty());
    }
}
