use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::port::PortId;

/// A connection (wire) between two ports on two different nodes.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Connection {
    pub id: Uuid,
    pub from_node: Uuid,
    pub from_port: PortId,
    pub to_node: Uuid,
    pub to_port: PortId,
}

impl Connection {
    pub fn new(from_node: Uuid, from_port: PortId, to_node: Uuid, to_port: PortId) -> Self {
        Self {
            id: Uuid::new_v4(),
            from_node,
            from_port,
            to_node,
            to_port,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn connection_ids_are_unique() {
        let a = Connection::new(Uuid::new_v4(), PortId::new(), Uuid::new_v4(), PortId::new());
        let b = Connection::new(Uuid::new_v4(), PortId::new(), Uuid::new_v4(), PortId::new());
        assert_ne!(a.id, b.id);
    }

    #[test]
    fn connection_stores_fields_correctly() {
        let from_node = Uuid::new_v4();
        let from_port = PortId::new();
        let to_node = Uuid::new_v4();
        let to_port = PortId::new();
        let c = Connection::new(from_node, from_port, to_node, to_port);
        assert_eq!(c.from_node, from_node);
        assert_eq!(c.from_port, from_port);
        assert_eq!(c.to_node, to_node);
        assert_eq!(c.to_port, to_port);
    }

    #[test]
    fn connection_serialization_roundtrip() {
        let c = Connection::new(Uuid::new_v4(), PortId::new(), Uuid::new_v4(), PortId::new());
        let json = serde_json::to_string(&c).unwrap();
        let loaded: Connection = serde_json::from_str(&json).unwrap();
        assert_eq!(loaded.id, c.id);
        assert_eq!(loaded.from_node, c.from_node);
        assert_eq!(loaded.to_port, c.to_port);
    }
}
