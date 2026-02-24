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
    fn serialization_roundtrip() {
        let node = Node::new_dialogue([50.0, 100.0]);
        let json = serde_json::to_string(&node).unwrap();
        let deserialized: Node = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.id, node.id);
        assert_eq!(deserialized.position, node.position);
    }
}
