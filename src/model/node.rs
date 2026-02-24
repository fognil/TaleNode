use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

use super::port::Port;

/// A node in the dialogue graph.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Node {
    pub id: Uuid,
    pub node_type: NodeType,
    /// Position on the canvas in canvas coordinates.
    pub position: [f32; 2],
    #[serde(default)]
    pub collapsed: bool,
    /// Input ports for this node.
    pub inputs: Vec<Port>,
    /// Output ports for this node.
    pub outputs: Vec<Port>,
}

impl Node {
    pub fn new_start(position: [f32; 2]) -> Self {
        Self {
            id: Uuid::new_v4(),
            node_type: NodeType::Start,
            position,
            collapsed: false,
            inputs: vec![],
            outputs: vec![Port::output()],
        }
    }

    pub fn new_dialogue(position: [f32; 2]) -> Self {
        Self {
            id: Uuid::new_v4(),
            node_type: NodeType::Dialogue(DialogueData::default()),
            position,
            collapsed: false,
            inputs: vec![Port::input()],
            outputs: vec![Port::output()],
        }
    }

    pub fn new_end(position: [f32; 2]) -> Self {
        Self {
            id: Uuid::new_v4(),
            node_type: NodeType::End(EndData::default()),
            position,
            collapsed: false,
            inputs: vec![Port::input()],
            outputs: vec![],
        }
    }

    pub fn new_choice(position: [f32; 2]) -> Self {
        let choice1 = ChoiceOption {
            id: Uuid::new_v4(),
            text: "Option 1".to_string(),
            condition: None,
        };
        let choice2 = ChoiceOption {
            id: Uuid::new_v4(),
            text: "Option 2".to_string(),
            condition: None,
        };
        Self {
            id: Uuid::new_v4(),
            node_type: NodeType::Choice(ChoiceData {
                prompt: String::new(),
                choices: vec![choice1, choice2],
            }),
            position,
            collapsed: false,
            inputs: vec![Port::input()],
            outputs: vec![
                Port::output_with_label("Option 1"),
                Port::output_with_label("Option 2"),
            ],
        }
    }

    pub fn new_condition(position: [f32; 2]) -> Self {
        Self {
            id: Uuid::new_v4(),
            node_type: NodeType::Condition(ConditionData {
                variable_name: String::new(),
                operator: CompareOp::Eq,
                value: VariableValue::Bool(true),
            }),
            position,
            collapsed: false,
            inputs: vec![Port::input()],
            outputs: vec![
                Port::output_with_label("True"),
                Port::output_with_label("False"),
            ],
        }
    }

    pub fn new_event(position: [f32; 2]) -> Self {
        Self {
            id: Uuid::new_v4(),
            node_type: NodeType::Event(EventData {
                actions: vec![],
            }),
            position,
            collapsed: false,
            inputs: vec![Port::input()],
            outputs: vec![Port::output()],
        }
    }

    pub fn new_random(position: [f32; 2]) -> Self {
        let b1 = RandomBranch {
            id: Uuid::new_v4(),
            weight: 0.5,
        };
        let b2 = RandomBranch {
            id: Uuid::new_v4(),
            weight: 0.5,
        };
        Self {
            id: Uuid::new_v4(),
            node_type: NodeType::Random(RandomData {
                branches: vec![b1, b2],
            }),
            position,
            collapsed: false,
            inputs: vec![Port::input()],
            outputs: vec![
                Port::output_with_label("50%"),
                Port::output_with_label("50%"),
            ],
        }
    }

    /// Add a choice option to a Choice node. Returns the new output port id.
    pub fn add_choice(&mut self) -> Option<Uuid> {
        if let NodeType::Choice(ref mut data) = self.node_type {
            let idx = data.choices.len() + 1;
            let option = ChoiceOption {
                id: Uuid::new_v4(),
                text: format!("Option {idx}"),
                condition: None,
            };
            let port = Port::output_with_label(&option.text);
            let port_uuid = port.id.0;
            data.choices.push(option);
            self.outputs.push(port);
            Some(port_uuid)
        } else {
            None
        }
    }

    /// Remove a choice option by index. Returns removed connections' port id.
    pub fn remove_choice(&mut self, index: usize) -> Option<Uuid> {
        if let NodeType::Choice(ref mut data) = self.node_type {
            if data.choices.len() <= 1 || index >= data.choices.len() {
                return None; // Keep at least 1 choice
            }
            data.choices.remove(index);
            // Output index 0.. maps to choices (outputs align with choices)
            if index < self.outputs.len() {
                let port = self.outputs.remove(index);
                return Some(port.id.0);
            }
        }
        None
    }

    /// Add a random branch. Returns the new output port id.
    pub fn add_random_branch(&mut self) -> Option<Uuid> {
        if let NodeType::Random(ref mut data) = self.node_type {
            let branch = RandomBranch {
                id: Uuid::new_v4(),
                weight: 0.0,
            };
            data.branches.push(branch);
            let port = Port::output_with_label("0%");
            let port_uuid = port.id.0;
            self.outputs.push(port);
            Some(port_uuid)
        } else {
            None
        }
    }

    /// Remove a random branch by index.
    pub fn remove_random_branch(&mut self, index: usize) -> Option<Uuid> {
        if let NodeType::Random(ref mut data) = self.node_type {
            if data.branches.len() <= 1 || index >= data.branches.len() {
                return None;
            }
            data.branches.remove(index);
            if index < self.outputs.len() {
                let port = self.outputs.remove(index);
                return Some(port.id.0);
            }
        }
        None
    }

    pub fn title(&self) -> &str {
        match &self.node_type {
            NodeType::Start => "Start",
            NodeType::Dialogue(data) => {
                if data.speaker_name.is_empty() {
                    "Dialogue"
                } else {
                    &data.speaker_name
                }
            }
            NodeType::Choice(_) => "Choice",
            NodeType::Condition(_) => "Condition",
            NodeType::Event(_) => "Event",
            NodeType::Random(_) => "Random",
            NodeType::End(_) => "End",
        }
    }
}

/// The type-specific data for each node.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NodeType {
    Start,
    Dialogue(DialogueData),
    Choice(ChoiceData),
    Condition(ConditionData),
    Event(EventData),
    Random(RandomData),
    End(EndData),
}

// --- Dialogue ---

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DialogueData {
    #[serde(default)]
    pub speaker_id: Option<Uuid>,
    #[serde(default)]
    pub speaker_name: String,
    #[serde(default)]
    pub text: String,
    #[serde(default)]
    pub emotion: String,
    #[serde(default)]
    pub portrait_override: Option<String>,
    #[serde(default)]
    pub audio_clip: Option<String>,
    #[serde(default)]
    pub metadata: HashMap<String, String>,
}

impl Default for DialogueData {
    fn default() -> Self {
        Self {
            speaker_id: None,
            speaker_name: String::new(),
            text: String::new(),
            emotion: "neutral".to_string(),
            portrait_override: None,
            audio_clip: None,
            metadata: HashMap::new(),
        }
    }
}

// --- Choice ---

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChoiceData {
    #[serde(default)]
    pub prompt: String,
    #[serde(default)]
    pub choices: Vec<ChoiceOption>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChoiceOption {
    pub id: Uuid,
    #[serde(default)]
    pub text: String,
    #[serde(default)]
    pub condition: Option<ConditionExpr>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConditionExpr {
    pub variable_name: String,
    pub operator: CompareOp,
    pub value: VariableValue,
}

// --- Condition ---

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConditionData {
    #[serde(default)]
    pub variable_name: String,
    #[serde(default)]
    pub operator: CompareOp,
    #[serde(default)]
    pub value: VariableValue,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Serialize, Deserialize)]
pub enum CompareOp {
    #[default]
    Eq,
    Neq,
    Gt,
    Lt,
    Gte,
    Lte,
    Contains,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum VariableValue {
    Bool(bool),
    Int(i64),
    Float(f64),
    Text(String),
}

impl Default for VariableValue {
    fn default() -> Self {
        Self::Bool(false)
    }
}

// --- Event ---

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventData {
    #[serde(default)]
    pub actions: Vec<EventAction>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventAction {
    pub action_type: EventActionType,
    #[serde(default)]
    pub key: String,
    #[serde(default)]
    pub value: VariableValue,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EventActionType {
    SetVariable,
    AddItem,
    RemoveItem,
    PlaySound,
    Custom(String),
}

// --- Random ---

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RandomData {
    #[serde(default)]
    pub branches: Vec<RandomBranch>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RandomBranch {
    pub id: Uuid,
    pub weight: f32,
}

// --- End ---

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct EndData {
    #[serde(default)]
    pub tag: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn start_node_has_no_inputs() {
        let node = Node::new_start([0.0, 0.0]);
        assert!(node.inputs.is_empty());
        assert_eq!(node.outputs.len(), 1);
    }

    #[test]
    fn dialogue_node_has_one_input_one_output() {
        let node = Node::new_dialogue([100.0, 200.0]);
        assert_eq!(node.inputs.len(), 1);
        assert_eq!(node.outputs.len(), 1);
    }

    #[test]
    fn end_node_has_no_outputs() {
        let node = Node::new_end([0.0, 0.0]);
        assert_eq!(node.inputs.len(), 1);
        assert!(node.outputs.is_empty());
    }

    #[test]
    fn node_title_defaults() {
        let start = Node::new_start([0.0, 0.0]);
        assert_eq!(start.title(), "Start");

        let dlg = Node::new_dialogue([0.0, 0.0]);
        assert_eq!(dlg.title(), "Dialogue");

        let end = Node::new_end([0.0, 0.0]);
        assert_eq!(end.title(), "End");
    }

    #[test]
    fn choice_node_has_dynamic_outputs() {
        let mut node = Node::new_choice([0.0, 0.0]);
        assert_eq!(node.inputs.len(), 1);
        assert_eq!(node.outputs.len(), 2);
        node.add_choice();
        assert_eq!(node.outputs.len(), 3);
        node.remove_choice(0);
        assert_eq!(node.outputs.len(), 2);
    }

    #[test]
    fn condition_node_has_true_false_outputs() {
        let node = Node::new_condition([0.0, 0.0]);
        assert_eq!(node.inputs.len(), 1);
        assert_eq!(node.outputs.len(), 2);
        assert_eq!(node.outputs[0].label, "True");
        assert_eq!(node.outputs[1].label, "False");
    }

    #[test]
    fn event_node_has_one_input_one_output() {
        let node = Node::new_event([0.0, 0.0]);
        assert_eq!(node.inputs.len(), 1);
        assert_eq!(node.outputs.len(), 1);
    }

    #[test]
    fn random_node_has_dynamic_branches() {
        let mut node = Node::new_random([0.0, 0.0]);
        assert_eq!(node.outputs.len(), 2);
        node.add_random_branch();
        assert_eq!(node.outputs.len(), 3);
        node.remove_random_branch(0);
        assert_eq!(node.outputs.len(), 2);
    }

    #[test]
    fn serialization_roundtrip() {
        let node = Node::new_dialogue([50.0, 100.0]);
        let json = serde_json::to_string(&node).unwrap();
        let deserialized: Node = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.id, node.id);
        assert_eq!(deserialized.position, node.position);
    }

    #[test]
    fn title_all_node_types() {
        assert_eq!(Node::new_choice([0.0, 0.0]).title(), "Choice");
        assert_eq!(Node::new_condition([0.0, 0.0]).title(), "Condition");
        assert_eq!(Node::new_event([0.0, 0.0]).title(), "Event");
        assert_eq!(Node::new_random([0.0, 0.0]).title(), "Random");
    }

    #[test]
    fn dialogue_title_uses_speaker_name() {
        let mut node = Node::new_dialogue([0.0, 0.0]);
        if let NodeType::Dialogue(ref mut d) = node.node_type {
            d.speaker_name = "Guard".to_string();
        }
        assert_eq!(node.title(), "Guard");
    }

    #[test]
    fn add_choice_on_non_choice_returns_none() {
        let mut node = Node::new_dialogue([0.0, 0.0]);
        assert!(node.add_choice().is_none());
    }

    #[test]
    fn remove_choice_on_non_choice_returns_none() {
        let mut node = Node::new_dialogue([0.0, 0.0]);
        assert!(node.remove_choice(0).is_none());
    }

    #[test]
    fn remove_last_choice_refused() {
        let mut node = Node::new_choice([0.0, 0.0]);
        // Remove first, leaving 1
        node.remove_choice(0);
        // Try to remove the last one — should be refused
        assert!(node.remove_choice(0).is_none());
    }

    #[test]
    fn remove_choice_out_of_bounds() {
        let mut node = Node::new_choice([0.0, 0.0]);
        assert!(node.remove_choice(99).is_none());
    }

    #[test]
    fn add_choice_returns_port_uuid() {
        let mut node = Node::new_choice([0.0, 0.0]);
        let port_uuid = node.add_choice().unwrap();
        // The returned UUID should match the last output port
        assert_eq!(node.outputs.last().unwrap().id.0, port_uuid);
    }

    #[test]
    fn add_random_branch_on_non_random_returns_none() {
        let mut node = Node::new_dialogue([0.0, 0.0]);
        assert!(node.add_random_branch().is_none());
    }

    #[test]
    fn remove_random_branch_on_non_random_returns_none() {
        let mut node = Node::new_dialogue([0.0, 0.0]);
        assert!(node.remove_random_branch(0).is_none());
    }

    #[test]
    fn remove_last_random_branch_refused() {
        let mut node = Node::new_random([0.0, 0.0]);
        node.remove_random_branch(0); // 2 -> 1
        assert!(node.remove_random_branch(0).is_none()); // can't remove last
    }

    #[test]
    fn remove_random_branch_out_of_bounds() {
        let mut node = Node::new_random([0.0, 0.0]);
        assert!(node.remove_random_branch(99).is_none());
    }

    #[test]
    fn variable_value_default() {
        assert_eq!(VariableValue::default(), VariableValue::Bool(false));
    }

    #[test]
    fn compare_op_default() {
        assert_eq!(CompareOp::default(), CompareOp::Eq);
    }

    #[test]
    fn serialization_roundtrip_all_types() {
        let nodes = vec![
            Node::new_start([0.0, 0.0]),
            Node::new_dialogue([0.0, 0.0]),
            Node::new_choice([0.0, 0.0]),
            Node::new_condition([0.0, 0.0]),
            Node::new_event([0.0, 0.0]),
            Node::new_random([0.0, 0.0]),
            Node::new_end([0.0, 0.0]),
        ];
        for node in &nodes {
            let json = serde_json::to_string(node).unwrap();
            let loaded: Node = serde_json::from_str(&json).unwrap();
            assert_eq!(loaded.id, node.id);
            assert_eq!(loaded.inputs.len(), node.inputs.len());
            assert_eq!(loaded.outputs.len(), node.outputs.len());
        }
    }
}
