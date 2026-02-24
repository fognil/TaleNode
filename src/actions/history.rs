use crate::model::graph::DialogueGraph;

const MAX_HISTORY: usize = 100;

/// Snapshot-based undo/redo history.
/// Stores full graph snapshots for simplicity and reliability.
pub struct UndoHistory {
    undo_stack: Vec<DialogueGraph>,
    redo_stack: Vec<DialogueGraph>,
}

impl Default for UndoHistory {
    fn default() -> Self {
        Self::new()
    }
}

impl UndoHistory {
    pub fn new() -> Self {
        Self {
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
        }
    }

    /// Save the current graph state before a mutation.
    /// Call this BEFORE modifying the graph.
    pub fn save_snapshot(&mut self, graph: &DialogueGraph) {
        self.undo_stack.push(graph.clone());
        if self.undo_stack.len() > MAX_HISTORY {
            self.undo_stack.remove(0);
        }
        // New action clears redo stack
        self.redo_stack.clear();
    }

    /// Undo: restore the previous graph state.
    /// Returns the restored graph, or None if nothing to undo.
    pub fn undo(&mut self, current: &DialogueGraph) -> Option<DialogueGraph> {
        if let Some(prev) = self.undo_stack.pop() {
            self.redo_stack.push(current.clone());
            Some(prev)
        } else {
            None
        }
    }

    /// Redo: restore the next graph state.
    /// Returns the restored graph, or None if nothing to redo.
    pub fn redo(&mut self, current: &DialogueGraph) -> Option<DialogueGraph> {
        if let Some(next) = self.redo_stack.pop() {
            self.undo_stack.push(current.clone());
            Some(next)
        } else {
            None
        }
    }

    pub fn can_undo(&self) -> bool {
        !self.undo_stack.is_empty()
    }

    pub fn can_redo(&self) -> bool {
        !self.redo_stack.is_empty()
    }

    pub fn clear(&mut self) {
        self.undo_stack.clear();
        self.redo_stack.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::node::Node;

    #[test]
    fn undo_redo_basic() {
        let mut history = UndoHistory::new();
        let mut graph = DialogueGraph::new();

        // State 1: empty graph
        history.save_snapshot(&graph);

        // State 2: add a node
        graph.add_node(Node::new_start([0.0, 0.0]));
        assert_eq!(graph.nodes.len(), 1);

        // Undo -> back to empty
        let restored = history.undo(&graph).unwrap();
        assert_eq!(restored.nodes.len(), 0);
        graph = restored;

        // Redo -> back to 1 node
        let restored = history.redo(&graph).unwrap();
        assert_eq!(restored.nodes.len(), 1);
    }

    #[test]
    fn new_action_clears_redo() {
        let mut history = UndoHistory::new();
        let mut graph = DialogueGraph::new();

        history.save_snapshot(&graph);
        graph.add_node(Node::new_start([0.0, 0.0]));

        // Undo
        graph = history.undo(&graph).unwrap();

        // New action (not redo)
        history.save_snapshot(&graph);
        graph.add_node(Node::new_dialogue([100.0, 0.0]));

        // Redo should be empty now
        assert!(!history.can_redo());
    }

    #[test]
    fn max_history_limit() {
        let mut history = UndoHistory::new();
        let graph = DialogueGraph::new();

        for _ in 0..150 {
            history.save_snapshot(&graph);
        }

        assert!(history.undo_stack.len() <= 100);
    }

    #[test]
    fn undo_empty_returns_none() {
        let mut history = UndoHistory::new();
        let graph = DialogueGraph::new();
        assert!(history.undo(&graph).is_none());
    }

    #[test]
    fn redo_empty_returns_none() {
        let mut history = UndoHistory::new();
        let graph = DialogueGraph::new();
        assert!(history.redo(&graph).is_none());
    }

    #[test]
    fn can_undo_reflects_state() {
        let mut history = UndoHistory::new();
        let graph = DialogueGraph::new();
        assert!(!history.can_undo());
        history.save_snapshot(&graph);
        assert!(history.can_undo());
    }

    #[test]
    fn clear_resets_stacks() {
        let mut history = UndoHistory::new();
        let mut graph = DialogueGraph::new();

        history.save_snapshot(&graph);
        graph.add_node(Node::new_start([0.0, 0.0]));
        let _restored = history.undo(&graph).unwrap();
        // Both stacks have entries now
        assert!(history.can_redo());

        history.clear();
        assert!(!history.can_undo());
        assert!(!history.can_redo());
    }

    #[test]
    fn multiple_undo_redo_sequence() {
        let mut history = UndoHistory::new();
        let mut graph = DialogueGraph::new();

        // State 0: empty
        history.save_snapshot(&graph);
        graph.add_node(Node::new_start([0.0, 0.0]));
        // State 1: 1 node

        history.save_snapshot(&graph);
        graph.add_node(Node::new_dialogue([100.0, 0.0]));
        // State 2: 2 nodes

        history.save_snapshot(&graph);
        graph.add_node(Node::new_end([200.0, 0.0]));
        // State 3: 3 nodes

        // Undo 3 times
        graph = history.undo(&graph).unwrap();
        assert_eq!(graph.nodes.len(), 2);
        graph = history.undo(&graph).unwrap();
        assert_eq!(graph.nodes.len(), 1);
        let restored = history.undo(&graph).unwrap();
        assert_eq!(restored.nodes.len(), 0);
        graph = restored;

        // Redo 2 times
        graph = history.redo(&graph).unwrap();
        assert_eq!(graph.nodes.len(), 1);
        graph = history.redo(&graph).unwrap();
        assert_eq!(graph.nodes.len(), 2);
    }

    #[test]
    fn undo_preserves_connections() {
        let mut history = UndoHistory::new();
        let mut graph = DialogueGraph::new();

        let start = Node::new_start([0.0, 0.0]);
        let end = Node::new_end([200.0, 0.0]);
        let start_out = start.outputs[0].id;
        let end_in = end.inputs[0].id;
        let start_id = start.id;
        let end_id = end.id;
        graph.add_node(start);
        graph.add_node(end);
        graph.add_connection(start_id, start_out, end_id, end_in);

        // Snapshot the connected graph, then remove a node
        history.save_snapshot(&graph);
        graph.remove_node(end_id);
        assert!(graph.connections.is_empty());

        // Undo should restore the connection
        graph = history.undo(&graph).unwrap();
        assert_eq!(graph.connections.len(), 1);
    }
}
