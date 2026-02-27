use std::collections::BTreeMap;
use uuid::Uuid;

use super::connection::Connection;
use super::port::PortId;

/// O(1) connection lookups by node and port. Not serialized — rebuilt on load.
#[derive(Debug, Clone, Default)]
pub struct ConnectionIndex {
    /// Connection indices grouped by node ID (both from_node and to_node).
    by_node: BTreeMap<Uuid, Vec<usize>>,
    /// Connection index keyed by output port.
    by_from_port: BTreeMap<PortId, usize>,
    /// Connection index keyed by input port.
    by_to_port: BTreeMap<PortId, usize>,
}

impl ConnectionIndex {
    /// Rebuild the entire index from a connections slice.
    pub fn rebuild(connections: &[Connection]) -> Self {
        let mut idx = Self::default();
        for (i, conn) in connections.iter().enumerate() {
            idx.insert(i, conn);
        }
        idx
    }

    /// Check whether an output port already has a connection.
    pub fn has_from_port(&self, port: PortId) -> bool {
        self.by_from_port.contains_key(&port)
    }

    /// Check whether an input port already has a connection.
    pub fn has_to_port(&self, port: PortId) -> bool {
        self.by_to_port.contains_key(&port)
    }

    /// Return connection indices involving a given node.
    /// Used by collapsible groups and future optimizations.
    #[allow(dead_code)]
    pub fn connections_for_node(&self, node_id: Uuid) -> &[usize] {
        self.by_node
            .get(&node_id)
            .map(|v| v.as_slice())
            .unwrap_or(&[])
    }

    fn insert(&mut self, index: usize, conn: &Connection) {
        self.by_node
            .entry(conn.from_node)
            .or_default()
            .push(index);
        self.by_node
            .entry(conn.to_node)
            .or_default()
            .push(index);
        self.by_from_port.insert(conn.from_port, index);
        self.by_to_port.insert(conn.to_port, index);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_conn(from_node: Uuid, to_node: Uuid) -> Connection {
        Connection::new(from_node, PortId::new(), to_node, PortId::new())
    }

    #[test]
    fn rebuild_empty() {
        let idx = ConnectionIndex::rebuild(&[]);
        assert!(idx.by_node.is_empty());
        assert!(idx.by_from_port.is_empty());
        assert!(idx.by_to_port.is_empty());
    }

    #[test]
    fn rebuild_indexes_by_node() {
        let a = Uuid::new_v4();
        let b = Uuid::new_v4();
        let c = Uuid::new_v4();
        let conns = vec![make_conn(a, b), make_conn(a, c)];
        let idx = ConnectionIndex::rebuild(&conns);

        assert_eq!(idx.connections_for_node(a).len(), 2);
        assert_eq!(idx.connections_for_node(b).len(), 1);
        assert_eq!(idx.connections_for_node(c).len(), 1);
        assert!(idx.connections_for_node(Uuid::new_v4()).is_empty());
    }

    #[test]
    fn has_port_checks() {
        let conn = make_conn(Uuid::new_v4(), Uuid::new_v4());
        let fp = conn.from_port;
        let tp = conn.to_port;
        let idx = ConnectionIndex::rebuild(&[conn]);

        assert!(idx.has_from_port(fp));
        assert!(idx.has_to_port(tp));
        assert!(!idx.has_from_port(PortId::new()));
        assert!(!idx.has_to_port(PortId::new()));
    }

    #[test]
    fn connections_for_node_returns_correct_indices() {
        let a = Uuid::new_v4();
        let b = Uuid::new_v4();
        let c = Uuid::new_v4();
        let conns = vec![make_conn(a, b), make_conn(b, c), make_conn(a, c)];
        let idx = ConnectionIndex::rebuild(&conns);

        let a_conns = idx.connections_for_node(a);
        assert_eq!(a_conns.len(), 2);
        assert!(a_conns.contains(&0));
        assert!(a_conns.contains(&2));

        let b_conns = idx.connections_for_node(b);
        assert_eq!(b_conns.len(), 2);
        assert!(b_conns.contains(&0));
        assert!(b_conns.contains(&1));
    }
}
