use uuid::Uuid;

use crate::actions::history::UndoHistory;
use crate::model::graph::DialogueGraph;
use crate::model::node::NodeType;
use crate::ui::canvas::CanvasState;

/// Saved state of the parent graph when entering a sub-graph.
pub(super) struct SubGraphFrame {
    pub graph: DialogueGraph,
    pub canvas: CanvasState,
    pub selected_nodes: Vec<Uuid>,
    pub history: UndoHistory,
    /// The SubGraph node we entered in the parent graph.
    pub node_id: Uuid,
    /// Display name for breadcrumb.
    pub name: String,
}

impl super::TaleNodeApp {
    /// Enter a SubGraph node: push parent state and switch to child graph.
    pub(super) fn enter_subgraph(&mut self, node_id: Uuid) {
        let Some(node) = self.graph.nodes.get(&node_id) else {
            return;
        };
        let child_graph = match &node.node_type {
            NodeType::SubGraph(data) => data.child_graph.clone(),
            _ => return,
        };
        let name = node.title().to_string();

        let frame = SubGraphFrame {
            graph: self.graph.clone(),
            canvas: self.canvas.clone(),
            selected_nodes: std::mem::take(&mut self.selected_nodes),
            history: std::mem::take(&mut self.history),
            node_id,
            name,
        };
        self.subgraph_stack.push(frame);
        self.graph = child_graph;
        self.canvas = CanvasState::default();
        self.interaction = super::InteractionState::Idle;
        self.context_menu_pos = None;
    }

    /// Exit current sub-graph: save child back into parent, restore parent state.
    pub(super) fn exit_subgraph(&mut self) {
        let Some(mut frame) = self.subgraph_stack.pop() else {
            return;
        };

        // Save modified child graph back into parent's SubGraph node
        if let Some(parent_node) = frame.graph.nodes.get_mut(&frame.node_id) {
            if let NodeType::SubGraph(ref mut data) = parent_node.node_type {
                data.child_graph = std::mem::take(&mut self.graph);
            }
        }

        self.graph = frame.graph;
        self.canvas = frame.canvas;
        self.selected_nodes = frame.selected_nodes;
        self.history = frame.history;
        self.interaction = super::InteractionState::Idle;
        self.context_menu_pos = None;
    }

    /// Get breadcrumb labels for the current subgraph navigation path.
    pub(super) fn breadcrumb_labels(&self) -> Vec<&str> {
        self.subgraph_stack.iter().map(|f| f.name.as_str()).collect()
    }

    /// Whether we are currently inside a sub-graph.
    pub(super) fn is_in_subgraph(&self) -> bool {
        !self.subgraph_stack.is_empty()
    }
}
