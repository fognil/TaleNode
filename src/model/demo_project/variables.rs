#[cfg(test)]
use crate::model::node_types::VariableValue;
#[cfg(test)]
use crate::model::variable::{Variable, VariableType};

#[cfg(test)]
pub fn build_variables() -> Vec<Variable> {
    vec![
        Variable {
            id: uuid::Uuid::new_v4(),
            name: "player_name".into(),
            var_type: VariableType::Text,
            default_value: VariableValue::Text("Agent".into()),
        },
        Variable::new_int("faction_reputation_solari", 0),
        Variable::new_int("faction_reputation_umbral", 0),
        Variable::new_bool("has_evidence", false),
        Variable::new_bool("has_weapon", false),
        Variable::new_int("credits", 500),
        Variable::new_int("trust_elara", 0),
        Variable::new_int("trust_kael", 0),
        Variable::new_bool("discovered_conspiracy", false),
        Variable::new_int("peace_progress", 0),
        Variable::new_int("chapter", 1),
        Variable::new_int("encounters", 0),
    ]
}
