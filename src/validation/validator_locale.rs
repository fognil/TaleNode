use std::collections::HashSet;

use crate::model::graph::DialogueGraph;

use super::validator::{Severity, ValidationWarning};

/// Warn when extra locales are configured but strings lack translations.
pub(super) fn check_untranslated_strings(
    graph: &DialogueGraph,
    warnings: &mut Vec<ValidationWarning>,
) {
    if !graph.locale.has_extra_locales() {
        return;
    }
    let strings = crate::model::locale::collect_translatable_strings(graph);
    for loc in &graph.locale.extra_locales {
        let missing: usize = strings
            .iter()
            .filter(|s| {
                graph
                    .locale
                    .get_translation(&s.key, loc)
                    .is_none_or(|t| t.is_empty())
            })
            .count();
        if missing > 0 {
            warnings.push(ValidationWarning {
                node_id: None,
                message: format!("Locale '{loc}': {missing} untranslated string(s)"),
                severity: Severity::Warning,
            });
        }
    }
}

/// Warn when translations reference keys that no longer exist in the graph.
pub(super) fn check_orphaned_translations(
    graph: &DialogueGraph,
    warnings: &mut Vec<ValidationWarning>,
) {
    if graph.locale.translations.is_empty() {
        return;
    }
    let strings = crate::model::locale::collect_translatable_strings(graph);
    let valid_keys: HashSet<&str> = strings.iter().map(|s| s.key.as_str()).collect();
    let orphaned = graph
        .locale
        .translations
        .keys()
        .filter(|k| !valid_keys.contains(k.as_str()))
        .count();
    if orphaned > 0 {
        warnings.push(ValidationWarning {
            node_id: None,
            message: format!("{orphaned} orphaned translation key(s) (nodes deleted)"),
            severity: Severity::Warning,
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::node::{Node, NodeType};
    use crate::validation::validator::validate;

    #[test]
    fn untranslated_strings_warning() {
        let mut graph = DialogueGraph::new();
        let mut dlg = Node::new_dialogue([0.0, 0.0]);
        if let NodeType::Dialogue(ref mut d) = dlg.node_type {
            d.text = "Hello".to_string();
        }
        graph.add_node(dlg);
        graph.locale.add_locale("fr".to_string());
        let warnings = validate(&graph);
        assert!(warnings.iter().any(|w| w.message.contains("untranslated")));
    }

    #[test]
    fn no_warning_when_fully_translated() {
        let mut graph = DialogueGraph::new();
        let mut dlg = Node::new_dialogue([0.0, 0.0]);
        let dlg_key = format!("dlg_{}", &dlg.id.to_string()[..8]);
        if let NodeType::Dialogue(ref mut d) = dlg.node_type {
            d.text = "Hello".to_string();
        }
        graph.add_node(dlg);
        graph.locale.add_locale("fr".to_string());
        graph
            .locale
            .set_translation(dlg_key, "fr".to_string(), "Bonjour".to_string());
        let warnings = validate(&graph);
        assert!(!warnings.iter().any(|w| w.message.contains("untranslated")));
    }

    #[test]
    fn orphaned_translation_warning() {
        let mut graph = DialogueGraph::new();
        graph.locale.set_translation(
            "dlg_deleted".to_string(),
            "fr".to_string(),
            "Bonjour".to_string(),
        );
        let warnings = validate(&graph);
        assert!(warnings.iter().any(|w| w.message.contains("orphaned")));
    }

    #[test]
    fn no_warning_without_locales() {
        let mut graph = DialogueGraph::new();
        let mut dlg = Node::new_dialogue([0.0, 0.0]);
        if let NodeType::Dialogue(ref mut d) = dlg.node_type {
            d.text = "Hello".to_string();
        }
        graph.add_node(dlg);
        // No extra locales — should not produce locale warnings
        let warnings = validate(&graph);
        assert!(!warnings.iter().any(|w| w.message.contains("untranslated")));
    }
}
