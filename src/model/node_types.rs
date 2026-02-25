use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

use super::graph::DialogueGraph;

// --- NodeType enum ---

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
    SubGraph(SubGraphData),
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
    #[serde(default)]
    pub variable_name: String,
    #[serde(default)]
    pub operator: CompareOp,
    #[serde(default)]
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
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
    #[serde(default)]
    pub action_type: EventActionType,
    #[serde(default)]
    pub key: String,
    #[serde(default)]
    pub value: VariableValue,
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum EventActionType {
    #[default]
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
    #[serde(default = "Uuid::new_v4")]
    pub id: Uuid,
    #[serde(default)]
    pub weight: f32,
}

// --- End ---

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct EndData {
    #[serde(default)]
    pub tag: String,
}

// --- SubGraph ---

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubGraphData {
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub child_graph: DialogueGraph,
}

impl Default for SubGraphData {
    fn default() -> Self {
        let mut graph = DialogueGraph::new();
        let start = super::node::Node::new_start([100.0, 200.0]);
        graph.add_node(start);
        Self {
            name: String::new(),
            child_graph: graph,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn variable_value_default() {
        assert_eq!(VariableValue::default(), VariableValue::Bool(false));
    }

    #[test]
    fn compare_op_default() {
        assert_eq!(CompareOp::default(), CompareOp::Eq);
    }

    #[test]
    fn event_action_type_default() {
        assert!(matches!(EventActionType::default(), EventActionType::SetVariable));
    }
}
