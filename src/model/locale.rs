use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use uuid::Uuid;

use super::graph::DialogueGraph;
use super::node::NodeType;

/// Localization settings for the dialogue graph.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocaleSettings {
    #[serde(default = "default_locale")]
    pub default_locale: String,
    #[serde(default)]
    pub extra_locales: Vec<String>,
    /// string_key -> { locale_code -> translated_text }
    #[serde(default)]
    pub translations: BTreeMap<String, BTreeMap<String, String>>,
}

fn default_locale() -> String {
    "en".to_string()
}

impl Default for LocaleSettings {
    fn default() -> Self {
        Self {
            default_locale: default_locale(),
            extra_locales: Vec::new(),
            translations: BTreeMap::new(),
        }
    }
}

impl LocaleSettings {
    pub fn add_locale(&mut self, locale: String) {
        if locale != self.default_locale && !self.extra_locales.contains(&locale) {
            self.extra_locales.push(locale);
        }
    }

    pub fn remove_locale(&mut self, locale: &str) {
        self.extra_locales.retain(|l| l != locale);
        // Remove all translations for this locale
        for translations in self.translations.values_mut() {
            translations.remove(locale);
        }
    }

    pub fn get_translation(&self, key: &str, locale: &str) -> Option<&str> {
        self.translations
            .get(key)
            .and_then(|m| m.get(locale))
            .map(|s| s.as_str())
    }

    pub fn set_translation(&mut self, key: String, locale: String, text: String) {
        if text.is_empty() {
            if let Some(m) = self.translations.get_mut(&key) {
                m.remove(&locale);
                if m.is_empty() {
                    self.translations.remove(&key);
                }
            }
        } else {
            self.translations
                .entry(key)
                .or_default()
                .insert(locale, text);
        }
    }

    pub fn has_extra_locales(&self) -> bool {
        !self.extra_locales.is_empty()
    }
}

/// A translatable string extracted from the graph.
#[derive(Debug, Clone)]
pub struct TranslatableString {
    pub key: String,
    pub string_type: StringType,
    pub default_text: String,
    pub node_id: Uuid,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StringType {
    Dialogue,
    ChoicePrompt,
    ChoiceOption,
}

impl StringType {
    pub fn label(self) -> &'static str {
        match self {
            Self::Dialogue => "dialogue",
            Self::ChoicePrompt => "prompt",
            Self::ChoiceOption => "option",
        }
    }
}

/// Generate the string key for a node's UUID (first 8 hex chars).
fn uuid8(id: Uuid) -> String {
    id.to_string()[..8].to_string()
}

/// Collect all translatable strings from a graph.
pub fn collect_translatable_strings(graph: &DialogueGraph) -> Vec<TranslatableString> {
    let mut result = Vec::new();
    let mut nodes: Vec<_> = graph.nodes.values().collect();
    nodes.sort_by(|a, b| {
        a.position[1]
            .partial_cmp(&b.position[1])
            .unwrap_or(std::cmp::Ordering::Equal)
            .then(
                a.position[0]
                    .partial_cmp(&b.position[0])
                    .unwrap_or(std::cmp::Ordering::Equal),
            )
    });

    for node in nodes {
        let short = uuid8(node.id);
        match &node.node_type {
            NodeType::Dialogue(data) => {
                result.push(TranslatableString {
                    key: format!("dlg_{short}"),
                    string_type: StringType::Dialogue,
                    default_text: data.text.clone(),
                    node_id: node.id,
                });
            }
            NodeType::Choice(data) => {
                result.push(TranslatableString {
                    key: format!("choice_{short}"),
                    string_type: StringType::ChoicePrompt,
                    default_text: data.prompt.clone(),
                    node_id: node.id,
                });
                for (i, opt) in data.choices.iter().enumerate() {
                    result.push(TranslatableString {
                        key: format!("opt_{short}_{i}"),
                        string_type: StringType::ChoiceOption,
                        default_text: opt.text.clone(),
                        node_id: node.id,
                    });
                }
            }
            _ => {}
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::node::Node;

    #[test]
    fn default_locale_settings() {
        let ls = LocaleSettings::default();
        assert_eq!(ls.default_locale, "en");
        assert!(ls.extra_locales.is_empty());
        assert!(ls.translations.is_empty());
    }

    #[test]
    fn add_and_remove_locale() {
        let mut ls = LocaleSettings::default();
        ls.add_locale("fr".to_string());
        ls.add_locale("ja".to_string());
        assert_eq!(ls.extra_locales.len(), 2);
        // Duplicate ignored
        ls.add_locale("fr".to_string());
        assert_eq!(ls.extra_locales.len(), 2);
        // Default locale ignored
        ls.add_locale("en".to_string());
        assert_eq!(ls.extra_locales.len(), 2);
        // Remove also cleans translations
        ls.set_translation("dlg_abc".to_string(), "fr".to_string(), "Bonjour".to_string());
        ls.remove_locale("fr");
        assert_eq!(ls.extra_locales.len(), 1);
        assert!(ls.get_translation("dlg_abc", "fr").is_none());
    }

    #[test]
    fn get_set_translation() {
        let mut ls = LocaleSettings::default();
        ls.set_translation("dlg_abc".to_string(), "fr".to_string(), "Bonjour".to_string());
        assert_eq!(ls.get_translation("dlg_abc", "fr"), Some("Bonjour"));
        assert!(ls.get_translation("dlg_abc", "ja").is_none());
        // Setting empty removes
        ls.set_translation("dlg_abc".to_string(), "fr".to_string(), String::new());
        assert!(ls.get_translation("dlg_abc", "fr").is_none());
        assert!(ls.translations.is_empty());
    }

    #[test]
    fn collect_strings_from_graph() {
        let mut graph = DialogueGraph::new();
        let mut dlg = Node::new_dialogue([0.0, 100.0]);
        if let NodeType::Dialogue(ref mut d) = dlg.node_type {
            d.text = "Hello!".to_string();
        }
        let mut choice = Node::new_choice([0.0, 200.0]);
        if let NodeType::Choice(ref mut c) = choice.node_type {
            c.prompt = "What now?".to_string();
        }
        graph.add_node(dlg);
        graph.add_node(choice);
        // Start node should not produce strings
        graph.add_node(Node::new_start([0.0, 0.0]));

        let strings = collect_translatable_strings(&graph);
        assert_eq!(strings.len(), 4); // dlg text + choice prompt + 2 options
        assert!(strings.iter().any(|s| s.default_text == "Hello!"));
        assert!(strings.iter().any(|s| s.default_text == "What now?"));
    }

    #[test]
    fn serialization_roundtrip() {
        let mut ls = LocaleSettings::default();
        ls.add_locale("fr".to_string());
        ls.set_translation("dlg_abc".to_string(), "fr".to_string(), "Bonjour".to_string());
        let json = serde_json::to_string(&ls).unwrap();
        let loaded: LocaleSettings = serde_json::from_str(&json).unwrap();
        assert_eq!(loaded.default_locale, "en");
        assert_eq!(loaded.extra_locales, vec!["fr"]);
        assert_eq!(loaded.get_translation("dlg_abc", "fr"), Some("Bonjour"));
    }
}
