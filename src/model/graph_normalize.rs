use super::graph::DialogueGraph;
use super::node::NodeType;

impl DialogueGraph {
    /// Sort all Vec collections for deterministic serialization.
    ///
    /// BTreeMap keys are already sorted; this method handles Vecs that
    /// would otherwise serialize in insertion order.
    pub fn normalize(&mut self) {
        // Connections: sort by (from_node, from_port, to_node, to_port)
        self.connections.sort_by(|a, b| {
            (a.from_node, a.from_port.0, a.to_node, a.to_port.0)
                .cmp(&(b.from_node, b.from_port.0, b.to_node, b.to_port.0))
        });

        // Variables: sort by name
        self.variables.sort_by(|a, b| a.name.cmp(&b.name));

        // Characters: sort by name
        self.characters.sort_by(|a, b| a.name.cmp(&b.name));

        // Groups: sort by name
        self.groups.sort_by(|a, b| a.name.cmp(&b.name));

        // Comments: sort by (node_id, id)
        self.comments
            .sort_by(|a, b| (a.node_id, a.id).cmp(&(b.node_id, b.id)));

        // Quests: sort by name
        self.quests.sort_by(|a, b| a.name.cmp(&b.name));

        // World entities: sort by name
        self.world_entities.sort_by(|a, b| a.name.cmp(&b.name));

        // Timelines: sort by name
        self.timelines.sort_by(|a, b| a.name.cmp(&b.name));

        // Sort tags within each node
        for tags in self.node_tags.values_mut() {
            tags.sort();
        }

        // Recurse into SubGraph child graphs
        for node in self.nodes.values_mut() {
            if let NodeType::SubGraph(ref mut data) = node.node_type {
                data.child_graph.normalize();
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::connection::Connection;
    use crate::model::node::Node;
    use crate::model::port::PortId;
    use uuid::Uuid;

    #[test]
    fn normalize_sorts_connections() {
        let mut graph = DialogueGraph::new();
        let id_a = Uuid::nil();
        let id_b = Uuid::max();
        graph.connections.push(Connection {
            id: Uuid::new_v4(),
            from_node: id_b,
            from_port: PortId(Uuid::nil()),
            to_node: id_a,
            to_port: PortId(Uuid::nil()),
        });
        graph.connections.push(Connection {
            id: Uuid::new_v4(),
            from_node: id_a,
            from_port: PortId(Uuid::nil()),
            to_node: id_b,
            to_port: PortId(Uuid::nil()),
        });
        graph.normalize();
        assert!(graph.connections[0].from_node < graph.connections[1].from_node);
    }

    #[test]
    fn normalize_sorts_variables() {
        let mut graph = DialogueGraph::new();
        graph.variables.push(crate::model::variable::Variable {
            id: Uuid::new_v4(),
            name: "zzz".to_string(),
            var_type: crate::model::variable::VariableType::Bool,
            default_value: crate::model::node::VariableValue::Bool(false),
        });
        graph.variables.push(crate::model::variable::Variable {
            id: Uuid::new_v4(),
            name: "aaa".to_string(),
            var_type: crate::model::variable::VariableType::Bool,
            default_value: crate::model::node::VariableValue::Bool(false),
        });
        graph.normalize();
        assert_eq!(graph.variables[0].name, "aaa");
        assert_eq!(graph.variables[1].name, "zzz");
    }

    #[test]
    fn normalize_is_idempotent() {
        let mut graph = DialogueGraph::new();
        graph.add_node(Node::new_start([0.0, 0.0]));
        graph.add_node(Node::new_dialogue([100.0, 100.0]));
        graph.variables.push(crate::model::variable::Variable {
            id: Uuid::new_v4(),
            name: "b".to_string(),
            var_type: crate::model::variable::VariableType::Bool,
            default_value: crate::model::node::VariableValue::Bool(false),
        });
        graph.variables.push(crate::model::variable::Variable {
            id: Uuid::new_v4(),
            name: "a".to_string(),
            var_type: crate::model::variable::VariableType::Bool,
            default_value: crate::model::node::VariableValue::Bool(false),
        });
        graph.normalize();
        let json1 = serde_json::to_string(&graph).unwrap();
        graph.normalize();
        let json2 = serde_json::to_string(&graph).unwrap();
        assert_eq!(json1, json2);
    }

    #[test]
    fn normalize_sorts_tags() {
        let mut graph = DialogueGraph::new();
        let node = Node::new_dialogue([0.0, 0.0]);
        let id = node.id;
        graph.add_node(node);
        graph.add_tag(id, "quest".to_string());
        graph.add_tag(id, "boss".to_string());
        graph.add_tag(id, "act1".to_string());
        graph.normalize();
        assert_eq!(graph.get_tags(id), &["act1", "boss", "quest"]);
    }

    #[test]
    fn example_file_loads_and_resaves_deterministically() {
        let path = "examples/dragon_quest.talenode";
        let json = std::fs::read_to_string(path).unwrap();
        let project = crate::model::project::Project::load_from_string(&json).unwrap();
        assert!(!project.graph.nodes.is_empty());
        // Save twice — output must be identical
        let out1 = project.save_to_string().unwrap();
        let out2 = project.save_to_string().unwrap();
        assert_eq!(out1, out2);
    }
}
