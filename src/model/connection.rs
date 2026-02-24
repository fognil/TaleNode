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
}
