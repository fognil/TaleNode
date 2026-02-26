mod builtins;
pub mod eval;
pub mod expr;
mod expr_parser;
mod expr_tokenizer;
pub mod interpolate;

use std::collections::HashMap;

use crate::model::node::{
    CompareOp, ConditionData, ConditionExpr, EventActionType, EventData, VariableValue,
};
use crate::model::variable::Variable;

use self::eval::eval_to_bool;

/// Runtime variable context for playtest mode.
#[derive(Debug, Clone, Default)]
pub struct ScriptContext {
    vars: HashMap<String, VariableValue>,
}

impl ScriptContext {
    /// Initialize context from graph variable definitions using their defaults.
    pub fn from_variables(variables: &[Variable]) -> Self {
        let mut vars = HashMap::new();
        for v in variables {
            vars.insert(v.name.clone(), v.default_value.clone());
        }
        Self { vars }
    }

    pub fn get(&self, name: &str) -> Option<&VariableValue> {
        self.vars.get(name)
    }

    pub fn set(&mut self, name: &str, value: VariableValue) {
        self.vars.insert(name.to_string(), value);
    }

    /// Get all variables as (name, value) pairs, sorted by name.
    pub fn all_vars(&self) -> Vec<(&str, &VariableValue)> {
        let mut pairs: Vec<_> = self.vars.iter().map(|(k, v)| (k.as_str(), v)).collect();
        pairs.sort_by_key(|(k, _)| *k);
        pairs
    }

    /// Export all variables as owned (name, value) pairs for checkpointing.
    pub fn to_pairs(&self) -> Vec<(String, VariableValue)> {
        let mut pairs: Vec<_> = self.vars.iter().map(|(k, v)| (k.clone(), v.clone())).collect();
        pairs.sort_by(|(a, _), (b, _)| a.cmp(b));
        pairs
    }

    /// Restore from owned (name, value) pairs.
    pub fn from_pairs(pairs: Vec<(String, VariableValue)>) -> Self {
        Self {
            vars: pairs.into_iter().collect(),
        }
    }
}

/// Evaluate a Condition node's data against the runtime context.
/// Returns true for the "True" branch, false for the "False" branch.
pub fn evaluate_condition(ctx: &ScriptContext, data: &ConditionData) -> bool {
    let Some(var_val) = ctx.get(&data.variable_name) else {
        return false; // Undefined variable → false
    };
    compare_values(var_val, &data.operator, &data.value)
}

/// Evaluate a ConditionExpr (used on choice options).
pub fn evaluate_condition_expr(ctx: &ScriptContext, expr: &ConditionExpr) -> bool {
    let Some(var_val) = ctx.get(&expr.variable_name) else {
        return false;
    };
    compare_values(var_val, &expr.operator, &expr.value)
}

fn compare_values(var_val: &VariableValue, op: &CompareOp, target: &VariableValue) -> bool {
    match (var_val, target) {
        (VariableValue::Bool(a), VariableValue::Bool(b)) => match op {
            CompareOp::Eq => a == b,
            CompareOp::Neq => a != b,
            _ => false,
        },
        (VariableValue::Int(a), VariableValue::Int(b)) => match op {
            CompareOp::Eq => a == b,
            CompareOp::Neq => a != b,
            CompareOp::Gt => a > b,
            CompareOp::Lt => a < b,
            CompareOp::Gte => a >= b,
            CompareOp::Lte => a <= b,
            CompareOp::Contains => false,
        },
        (VariableValue::Float(a), VariableValue::Float(b)) => match op {
            CompareOp::Eq => (a - b).abs() < f64::EPSILON,
            CompareOp::Neq => (a - b).abs() >= f64::EPSILON,
            CompareOp::Gt => a > b,
            CompareOp::Lt => a < b,
            CompareOp::Gte => a >= b,
            CompareOp::Lte => a <= b,
            CompareOp::Contains => false,
        },
        // Int vs Float cross-comparison
        (VariableValue::Int(a), VariableValue::Float(b)) => {
            let af = *a as f64;
            match op {
                CompareOp::Eq => (af - b).abs() < f64::EPSILON,
                CompareOp::Neq => (af - b).abs() >= f64::EPSILON,
                CompareOp::Gt => af > *b,
                CompareOp::Lt => af < *b,
                CompareOp::Gte => af >= *b,
                CompareOp::Lte => af <= *b,
                CompareOp::Contains => false,
            }
        }
        (VariableValue::Float(a), VariableValue::Int(b)) => {
            let bf = *b as f64;
            match op {
                CompareOp::Eq => (a - bf).abs() < f64::EPSILON,
                CompareOp::Neq => (a - bf).abs() >= f64::EPSILON,
                CompareOp::Gt => *a > bf,
                CompareOp::Lt => *a < bf,
                CompareOp::Gte => *a >= bf,
                CompareOp::Lte => *a <= bf,
                CompareOp::Contains => false,
            }
        }
        (VariableValue::Text(a), VariableValue::Text(b)) => match op {
            CompareOp::Eq => a == b,
            CompareOp::Neq => a != b,
            CompareOp::Gt => a > b,
            CompareOp::Lt => a < b,
            CompareOp::Gte => a >= b,
            CompareOp::Lte => a <= b,
            CompareOp::Contains => a.contains(b.as_str()),
        },
        // Bool truthiness for non-bool comparisons
        (VariableValue::Bool(a), other) | (other, VariableValue::Bool(a)) => {
            let bval = eval_to_bool(other);
            match op {
                CompareOp::Eq => *a == bval,
                CompareOp::Neq => *a != bval,
                _ => false,
            }
        }
        // Mismatched types that aren't covered above
        _ => matches!(op, CompareOp::Neq),
    }
}

/// Execute an Event node's actions, updating the runtime context.
pub fn execute_event(ctx: &mut ScriptContext, data: &EventData) {
    for action in &data.actions {
        if matches!(action.action_type, EventActionType::SetVariable) {
            ctx.set(&action.key, action.value.clone());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::node::{EventAction, EventActionType};

    #[test]
    fn context_from_variables() {
        let vars = vec![
            Variable::new_bool("has_key", false),
            Variable::new_int("gold", 100),
        ];
        let ctx = ScriptContext::from_variables(&vars);
        assert_eq!(ctx.get("has_key"), Some(&VariableValue::Bool(false)));
        assert_eq!(ctx.get("gold"), Some(&VariableValue::Int(100)));
        assert_eq!(ctx.get("nonexistent"), None);
    }

    #[test]
    fn context_set_and_get() {
        let mut ctx = ScriptContext::default();
        ctx.set("x", VariableValue::Int(42));
        assert_eq!(ctx.get("x"), Some(&VariableValue::Int(42)));
        ctx.set("x", VariableValue::Int(99));
        assert_eq!(ctx.get("x"), Some(&VariableValue::Int(99)));
    }

    #[test]
    fn evaluate_condition_int_comparison() {
        let mut ctx = ScriptContext::default();
        ctx.set("gold", VariableValue::Int(50));
        let data = ConditionData {
            variable_name: "gold".to_string(),
            operator: CompareOp::Gte,
            value: VariableValue::Int(100),
        };
        assert!(!evaluate_condition(&ctx, &data));

        ctx.set("gold", VariableValue::Int(100));
        assert!(evaluate_condition(&ctx, &data));
    }

    #[test]
    fn evaluate_condition_undefined_is_false() {
        let ctx = ScriptContext::default();
        let data = ConditionData {
            variable_name: "missing".to_string(),
            operator: CompareOp::Eq,
            value: VariableValue::Bool(true),
        };
        assert!(!evaluate_condition(&ctx, &data));
    }

    #[test]
    fn execute_event_sets_variables() {
        let mut ctx = ScriptContext::default();
        ctx.set("gold", VariableValue::Int(50));
        let data = EventData {
            actions: vec![
                EventAction {
                    action_type: EventActionType::SetVariable,
                    key: "gold".to_string(),
                    value: VariableValue::Int(100),
                },
                EventAction {
                    action_type: EventActionType::SetVariable,
                    key: "flag".to_string(),
                    value: VariableValue::Bool(true),
                },
            ],
        };
        execute_event(&mut ctx, &data);
        assert_eq!(ctx.get("gold"), Some(&VariableValue::Int(100)));
        assert_eq!(ctx.get("flag"), Some(&VariableValue::Bool(true)));
    }

    #[test]
    fn execute_event_ignores_non_set_variable() {
        let mut ctx = ScriptContext::default();
        let data = EventData {
            actions: vec![EventAction {
                action_type: EventActionType::PlaySound,
                key: "boom.wav".to_string(),
                value: VariableValue::Bool(false),
            }],
        };
        execute_event(&mut ctx, &data);
        assert!(ctx.all_vars().is_empty());
    }

    #[test]
    fn evaluate_condition_expr_works() {
        let mut ctx = ScriptContext::default();
        ctx.set("has_key", VariableValue::Bool(true));
        let expr = ConditionExpr {
            variable_name: "has_key".to_string(),
            operator: CompareOp::Eq,
            value: VariableValue::Bool(true),
        };
        assert!(evaluate_condition_expr(&ctx, &expr));
    }

    #[test]
    fn all_vars_sorted() {
        let mut ctx = ScriptContext::default();
        ctx.set("z_var", VariableValue::Int(1));
        ctx.set("a_var", VariableValue::Int(2));
        let vars = ctx.all_vars();
        assert_eq!(vars[0].0, "a_var");
        assert_eq!(vars[1].0, "z_var");
    }
}
