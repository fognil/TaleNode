use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::node::VariableValue;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum VariableType {
    #[default]
    Bool,
    Int,
    Float,
    Text,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Variable {
    pub id: Uuid,
    pub name: String,
    pub var_type: VariableType,
    pub default_value: VariableValue,
}

impl Variable {
    pub fn new_bool(name: impl Into<String>, default: bool) -> Self {
        Self {
            id: Uuid::new_v4(),
            name: name.into(),
            var_type: VariableType::Bool,
            default_value: VariableValue::Bool(default),
        }
    }

    pub fn new_int(name: impl Into<String>, default: i64) -> Self {
        Self {
            id: Uuid::new_v4(),
            name: name.into(),
            var_type: VariableType::Int,
            default_value: VariableValue::Int(default),
        }
    }
}
