use std::collections::{BTreeSet, HashSet};
use std::mem;

use uuid::Uuid;

use crate::model::graph::DialogueGraph;
use crate::model::node::NodeType;

/// Actions the filter bar can request.
pub enum FilterAction {
    None,
    Changed,
    Clear,
}

/// Get discriminants for all filterable node types paired with labels.
fn type_discriminants() -> Vec<(&'static str, mem::Discriminant<NodeType>)> {
    use crate::model::node_types::*;
    vec![
        ("Dialogue", mem::discriminant(&NodeType::Dialogue(DialogueData::default()))),
        ("Choice", mem::discriminant(&NodeType::Choice(ChoiceData { prompt: String::new(), choices: Vec::new() }))),
        ("Condition", mem::discriminant(&NodeType::Condition(ConditionData { variable_name: String::new(), operator: CompareOp::Eq, value: VariableValue::default() }))),
        ("Event", mem::discriminant(&NodeType::Event(EventData { actions: Vec::new() }))),
        ("Random", mem::discriminant(&NodeType::Random(RandomData { branches: Vec::new() }))),
    ]
}

/// Draw the filter bar and return the action taken.
pub fn show_filter_bar(
    ui: &mut egui::Ui,
    graph: &DialogueGraph,
    active_tags: &mut Vec<String>,
    active_types: &mut HashSet<mem::Discriminant<NodeType>>,
    filter_active: bool,
) -> FilterAction {
    let mut action = FilterAction::None;

    ui.horizontal(|ui| {
        ui.label("Filter:");

        let all_tags: BTreeSet<&str> = graph
            .node_tags
            .values()
            .flat_map(|tags| tags.iter().map(String::as_str))
            .collect();

        egui::ComboBox::from_id_salt("filter_tags")
            .selected_text(if active_tags.is_empty() { "Tags..." } else { "Tags (active)" })
            .show_ui(ui, |ui| {
                for tag in &all_tags {
                    let mut checked = active_tags.contains(&tag.to_string());
                    if ui.checkbox(&mut checked, *tag).changed() {
                        if checked {
                            active_tags.push(tag.to_string());
                        } else {
                            active_tags.retain(|t| t != tag);
                        }
                        action = FilterAction::Changed;
                    }
                }
            });

        for (label, disc) in type_discriminants() {
            let mut checked = active_types.contains(&disc);
            if ui.checkbox(&mut checked, label).changed() {
                if checked { active_types.insert(disc); } else { active_types.remove(&disc); }
                action = FilterAction::Changed;
            }
        }

        if filter_active && ui.small_button("Clear").clicked() {
            action = FilterAction::Clear;
        }
    });

    action
}

/// Recompute the set of visible node IDs based on current filter settings.
pub fn compute_visible_nodes(
    graph: &DialogueGraph,
    tags: &[String],
    types: &HashSet<mem::Discriminant<NodeType>>,
) -> HashSet<Uuid> {
    let filter_by_tags = !tags.is_empty();
    let filter_by_types = !types.is_empty();

    graph
        .nodes
        .values()
        .filter(|node| {
            let pass_type = !filter_by_types || types.contains(&mem::discriminant(&node.node_type));
            let pass_tag = !filter_by_tags
                || graph
                    .get_tags(node.id)
                    .iter()
                    .any(|t| tags.contains(t));
            // Start and End nodes always pass (structural)
            let is_structural = matches!(
                node.node_type,
                NodeType::Start | NodeType::End(_) | NodeType::SubGraph(_)
            );
            is_structural || (pass_type && pass_tag)
        })
        .map(|node| node.id)
        .collect()
}
