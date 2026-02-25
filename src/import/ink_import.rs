use crate::model::graph::DialogueGraph;

/// Import an Ink (.ink) file into a DialogueGraph.
///
/// Supports core Ink: knots, choices (sticky + non-sticky), diverts,
/// variable declarations, simple conditionals, tags, and gathers.
/// Advanced features (tunnels, threads, lists, functions) are skipped.
pub fn import_ink(content: &str) -> Result<DialogueGraph, String> {
    let (global_vars, knots) = super::ink_parse::parse_ink(content);
    super::ink_build::build_ink_graph(global_vars, knots)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::node::NodeType;

    #[test]
    fn import_simple_ink() {
        let ink = "\
=== start ===
Guard: Hello there!
Guard: Welcome to the village.
";
        let graph = import_ink(ink).expect("should parse");
        assert!(graph.nodes.len() >= 3);
        assert_eq!(graph.characters.len(), 1);
        assert_eq!(graph.characters[0].name, "Guard");
    }

    #[test]
    fn import_ink_choices() {
        let ink = "\
=== tavern ===
Bartender: What'll it be?
* Ale please
    Bartender: Coming right up.
* Wine
    Bartender: Fancy taste!
+ Just looking
    Bartender: Take your time.
";
        let graph = import_ink(ink).expect("should parse");
        let choice_count = graph
            .nodes
            .values()
            .filter(|n| matches!(n.node_type, NodeType::Choice(_)))
            .count();
        assert!(choice_count >= 1);
    }

    #[test]
    fn import_ink_variables() {
        let ink = "\
VAR gold = 100
VAR has_key = false
=== start ===
~ gold = 50
You lost some gold.
";
        let graph = import_ink(ink).expect("should parse");
        assert_eq!(graph.variables.len(), 2);
        let event_count = graph
            .nodes
            .values()
            .filter(|n| matches!(n.node_type, NodeType::Event(_)))
            .count();
        assert_eq!(event_count, 1);
    }

    #[test]
    fn import_ink_diverts() {
        let ink = "\
=== start ===
Hello!
-> shop
=== shop ===
Merchant: Welcome to my shop!
";
        let graph = import_ink(ink).expect("should parse");
        assert!(graph.connections.len() >= 2);
    }

    #[test]
    fn import_ink_empty_returns_error() {
        assert!(import_ink("").is_err());
    }

    #[test]
    fn import_ink_preamble_only() {
        let ink = "Hello world!\nThis is a simple story.\n";
        let graph = import_ink(ink).expect("should parse preamble");
        assert!(!graph.nodes.is_empty());
    }

    #[test]
    fn import_ink_multiple_knots() {
        let ink = "\
=== intro ===
Welcome, traveler.
-> market

=== market ===
The market is bustling.
* [Buy bread] I'd like some bread.
    Vendor: Here you go!
* [Leave] Nevermind.
    -> intro
";
        let graph = import_ink(ink).expect("should parse");
        assert!(graph.nodes.len() >= 4);
    }
}
