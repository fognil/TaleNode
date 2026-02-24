use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Unique identifier for a port on a node.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PortId(pub Uuid);

impl PortId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl Default for PortId {
    fn default() -> Self {
        Self::new()
    }
}

/// Direction of a port.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PortDirection {
    Input,
    Output,
}

/// A port on a node that can be connected to other ports.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Port {
    pub id: PortId,
    pub direction: PortDirection,
    /// Label shown next to the port (e.g. "True", "False", choice text).
    #[serde(default)]
    pub label: String,
}

impl Port {
    pub fn input() -> Self {
        Self {
            id: PortId::new(),
            direction: PortDirection::Input,
            label: String::new(),
        }
    }

    pub fn output() -> Self {
        Self {
            id: PortId::new(),
            direction: PortDirection::Output,
            label: String::new(),
        }
    }

    pub fn output_with_label(label: impl Into<String>) -> Self {
        Self {
            id: PortId::new(),
            direction: PortDirection::Output,
            label: label.into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn port_ids_are_unique() {
        let a = PortId::new();
        let b = PortId::new();
        assert_ne!(a, b);
    }

    #[test]
    fn input_port_direction() {
        let p = Port::input();
        assert_eq!(p.direction, PortDirection::Input);
        assert!(p.label.is_empty());
    }

    #[test]
    fn output_port_direction() {
        let p = Port::output();
        assert_eq!(p.direction, PortDirection::Output);
        assert!(p.label.is_empty());
    }

    #[test]
    fn output_with_label_stores_label() {
        let p = Port::output_with_label("True");
        assert_eq!(p.direction, PortDirection::Output);
        assert_eq!(p.label, "True");
    }

    #[test]
    fn port_serialization_roundtrip() {
        let p = Port::output_with_label("False");
        let json = serde_json::to_string(&p).unwrap();
        let loaded: Port = serde_json::from_str(&json).unwrap();
        assert_eq!(loaded.id, p.id);
        assert_eq!(loaded.direction, PortDirection::Output);
        assert_eq!(loaded.label, "False");
    }
}
