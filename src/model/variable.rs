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
    #[serde(default = "Uuid::new_v4")]
    pub id: Uuid,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub var_type: VariableType,
    #[serde(default)]
    pub default_value: VariableValue,
}

impl Variable {
    #[allow(dead_code)]
    pub fn new_bool(name: impl Into<String>, default: bool) -> Self {
        Self {
            id: Uuid::new_v4(),
            name: name.into(),
            var_type: VariableType::Bool,
            default_value: VariableValue::Bool(default),
        }
    }

    #[allow(dead_code)]
    pub fn new_int(name: impl Into<String>, default: i64) -> Self {
        Self {
            id: Uuid::new_v4(),
            name: name.into(),
            var_type: VariableType::Int,
            default_value: VariableValue::Int(default),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_bool_variable() {
        let v = Variable::new_bool("has_key", true);
        assert_eq!(v.name, "has_key");
        assert_eq!(v.var_type, VariableType::Bool);
        assert_eq!(v.default_value, VariableValue::Bool(true));
    }

    #[test]
    fn new_int_variable() {
        let v = Variable::new_int("gold", 500);
        assert_eq!(v.name, "gold");
        assert_eq!(v.var_type, VariableType::Int);
        assert_eq!(v.default_value, VariableValue::Int(500));
    }

    #[test]
    fn variable_ids_are_unique() {
        let a = Variable::new_bool("a", false);
        let b = Variable::new_bool("b", false);
        assert_ne!(a.id, b.id);
    }

    #[test]
    fn variable_type_default_is_bool() {
        assert_eq!(VariableType::default(), VariableType::Bool);
    }

    #[test]
    fn variable_serialization_roundtrip() {
        let v = Variable::new_int("score", 42);
        let json = serde_json::to_string(&v).unwrap();
        let loaded: Variable = serde_json::from_str(&json).unwrap();
        assert_eq!(loaded.id, v.id);
        assert_eq!(loaded.name, "score");
        assert_eq!(loaded.var_type, VariableType::Int);
        assert_eq!(loaded.default_value, VariableValue::Int(42));
    }
}
