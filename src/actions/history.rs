use std::time::Instant;

use crate::model::graph::DialogueGraph;

const MAX_HISTORY: usize = 100;
/// Keep this many recent snapshots uncompressed for fast undo.
const KEEP_UNCOMPRESSED: usize = 5;
/// Minimum interval between snapshots for debouncing rapid edits.
const DEBOUNCE_MS: u128 = 300;

/// A snapshot that is either a full in-memory graph or compressed JSON bytes.
enum Snapshot {
    Full(Box<DialogueGraph>),
    Compressed(Vec<u8>),
}

impl Snapshot {
    fn into_graph(self) -> Option<DialogueGraph> {
        match self {
            Snapshot::Full(g) => Some(*g),
            Snapshot::Compressed(bytes) => serde_json::from_slice(&bytes).ok(),
        }
    }

    fn compress(graph: DialogueGraph) -> Self {
        match serde_json::to_vec(&graph) {
            Ok(bytes) => Snapshot::Compressed(bytes),
            Err(_) => Snapshot::Full(Box::new(graph)),
        }
    }
}

/// Snapshot-based undo/redo history with compression for old entries.
pub struct UndoHistory {
    undo_stack: Vec<Snapshot>,
    redo_stack: Vec<Snapshot>,
    last_snapshot_time: Option<Instant>,
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
            last_snapshot_time: None,
        }
    }

    /// Compress older entries when the uncompressed tail exceeds the threshold.
    fn compress_old_entries(stack: &mut [Snapshot]) {
        let len = stack.len();
        if len <= KEEP_UNCOMPRESSED {
            return;
        }
        let compress_up_to = len - KEEP_UNCOMPRESSED;
        for entry in stack.iter_mut().take(compress_up_to) {
            if matches!(entry, Snapshot::Full(_)) {
                let old = std::mem::replace(entry, Snapshot::Compressed(Vec::new()));
                if let Snapshot::Full(graph) = old {
                    *entry = Snapshot::compress(*graph);
                }
            }
        }
    }

    /// Save the current graph state before a mutation.
    /// Call this BEFORE modifying the graph.
    pub fn save_snapshot(&mut self, graph: &DialogueGraph) {
        self.undo_stack.push(Snapshot::Full(Box::new(graph.clone())));
        if self.undo_stack.len() > MAX_HISTORY {
            self.undo_stack.remove(0);
        }
        Self::compress_old_entries(&mut self.undo_stack);
        self.redo_stack.clear();
        self.last_snapshot_time = Some(Instant::now());
    }

    /// Save a snapshot only if enough time has passed since the last one.
    /// Returns true if the snapshot was actually saved.
    pub fn save_snapshot_debounced(&mut self, graph: &DialogueGraph) -> bool {
        if let Some(last) = self.last_snapshot_time {
            if last.elapsed().as_millis() < DEBOUNCE_MS {
                return false;
            }
        }
        self.save_snapshot(graph);
        true
    }

    /// Push an already-cloned graph onto the undo stack.
    pub fn push_undo(&mut self, graph: DialogueGraph) {
        self.undo_stack.push(Snapshot::Full(Box::new(graph)));
        if self.undo_stack.len() > MAX_HISTORY {
            self.undo_stack.remove(0);
        }
        Self::compress_old_entries(&mut self.undo_stack);
        self.redo_stack.clear();
        self.last_snapshot_time = Some(Instant::now());
    }

    /// Undo: restore the previous graph state.
    pub fn undo(&mut self, current: &DialogueGraph) -> Option<DialogueGraph> {
        let snapshot = self.undo_stack.pop()?;
        self.redo_stack.push(Snapshot::Full(Box::new(current.clone())));
        Self::compress_old_entries(&mut self.redo_stack);
        snapshot.into_graph()
    }

    /// Redo: restore the next graph state.
    pub fn redo(&mut self, current: &DialogueGraph) -> Option<DialogueGraph> {
        let snapshot = self.redo_stack.pop()?;
        self.undo_stack.push(Snapshot::Full(Box::new(current.clone())));
        Self::compress_old_entries(&mut self.undo_stack);
        snapshot.into_graph()
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
        self.last_snapshot_time = None;
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

        history.save_snapshot(&graph);
        graph.remove_node(end_id);
        assert!(graph.connections.is_empty());

        graph = history.undo(&graph).unwrap();
        assert_eq!(graph.connections.len(), 1);
    }

    #[test]
    fn old_snapshots_get_compressed() {
        let mut history = UndoHistory::new();
        let graph = DialogueGraph::new();

        // Push more than KEEP_UNCOMPRESSED snapshots
        for _ in 0..10 {
            history.save_snapshot(&graph);
        }

        // Oldest entries should be compressed
        let compressed_count = history
            .undo_stack
            .iter()
            .filter(|s| matches!(s, Snapshot::Compressed(_)))
            .count();
        assert!(compressed_count > 0);

        // Most recent KEEP_UNCOMPRESSED should be full
        let full_count = history
            .undo_stack
            .iter()
            .rev()
            .take(KEEP_UNCOMPRESSED)
            .filter(|s| matches!(s, Snapshot::Full(_)))
            .count();
        assert_eq!(full_count, KEEP_UNCOMPRESSED);
    }

    #[test]
    fn undo_through_compressed_snapshots() {
        let mut history = UndoHistory::new();
        let mut graph = DialogueGraph::new();

        // Build up snapshots with different node counts
        for i in 0..10 {
            history.save_snapshot(&graph);
            graph.add_node(Node::new_dialogue([i as f32 * 50.0, 0.0]));
        }
        // graph has 10 nodes, undo stack has 10 entries (some compressed)

        // Undo all the way back
        for expected in (0..10).rev() {
            graph = history.undo(&graph).unwrap();
            assert_eq!(graph.nodes.len(), expected);
        }
    }

    #[test]
    fn debounce_skips_rapid_snapshots() {
        let mut history = UndoHistory::new();
        let graph = DialogueGraph::new();

        // First snapshot always saves
        assert!(history.save_snapshot_debounced(&graph));
        // Immediate second call should be debounced
        assert!(!history.save_snapshot_debounced(&graph));
        assert_eq!(history.undo_stack.len(), 1);
    }
}
