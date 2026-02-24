/// Parsed entity (character) from articy XML.
pub(super) struct ArticyEntity {
    pub id: String,
    pub name: String,
}

/// Parsed dialogue fragment from articy XML.
pub(super) struct ArticyFragment {
    pub id: String,
    pub speaker_id: Option<String>,
    pub text: String,
    pub input_pins: Vec<String>,
    pub output_pins: Vec<String>,
}

/// Parsed hub (branching node) from articy XML.
pub(super) struct ArticyHub {
    pub id: String,
    pub display_name: String,
    pub input_pins: Vec<String>,
    pub output_pins: Vec<String>,
}

/// Parsed connection from articy XML.
pub(super) struct ArticyConnection {
    pub source_pin: String,
    pub target_pin: String,
}

/// Parsed variable from articy XML.
pub(super) struct ArticyVariable {
    pub name: String,
    pub var_type: String,
    pub value: String,
}

/// All data parsed from an articy:draft XML export.
pub(super) struct ArticyProject {
    pub entities: Vec<ArticyEntity>,
    pub fragments: Vec<ArticyFragment>,
    pub hubs: Vec<ArticyHub>,
    pub connections: Vec<ArticyConnection>,
    pub variables: Vec<ArticyVariable>,
}

pub(super) fn parse_articy(xml: &str) -> Result<ArticyProject, String> {
    let doc = roxmltree::Document::parse(xml)
        .map_err(|e| format!("XML parse error: {e}"))?;
    let root = doc.root_element();

    let content = find_child(&root, "Content")
        .ok_or("Missing <Content> element")?;

    let entities = parse_entities(&content);
    let fragments = parse_fragments(&content);
    let hubs = parse_hubs(&content);
    let connections = parse_connections(&content);
    let variables = parse_variables(&content);

    Ok(ArticyProject { entities, fragments, hubs, connections, variables })
}

fn parse_entities(content: &roxmltree::Node) -> Vec<ArticyEntity> {
    let mut result = Vec::new();
    let Some(entities_node) = find_child(content, "Entities") else {
        return result;
    };
    for entity in entities_node.children().filter(|n| n.has_tag_name("Entity")) {
        let id = entity.attribute("Id").unwrap_or("").to_string();
        let name = child_text(&entity, "DisplayName")
            .or_else(|| entity.attribute("DisplayName").map(|s| s.to_string()))
            .unwrap_or_default();
        if !name.is_empty() {
            result.push(ArticyEntity { id, name });
        }
    }
    result
}

fn parse_fragments(content: &roxmltree::Node) -> Vec<ArticyFragment> {
    let mut result = Vec::new();
    let Some(frags_node) = find_child(content, "DialogueFragments") else {
        return result;
    };
    for frag in frags_node.children().filter(|n| n.has_tag_name("DialogueFragment")) {
        let id = frag.attribute("Id").unwrap_or("").to_string();
        let speaker_id = frag.attribute("Speaker").map(|s| s.to_string());
        let text = child_text(&frag, "Text").unwrap_or_default();
        let (input_pins, output_pins) = parse_pins(&frag);
        result.push(ArticyFragment { id, speaker_id, text, input_pins, output_pins });
    }
    result
}

fn parse_hubs(content: &roxmltree::Node) -> Vec<ArticyHub> {
    let mut result = Vec::new();
    let Some(hubs_node) = find_child(content, "Hubs") else {
        return result;
    };
    for hub in hubs_node.children().filter(|n| n.has_tag_name("Hub")) {
        let id = hub.attribute("Id").unwrap_or("").to_string();
        let display_name = hub.attribute("DisplayName")
            .map(|s| s.to_string())
            .or_else(|| child_text(&hub, "DisplayName"))
            .unwrap_or_else(|| "Choice".to_string());
        let (input_pins, output_pins) = parse_pins(&hub);
        result.push(ArticyHub { id, display_name, input_pins, output_pins });
    }
    result
}

fn parse_connections(content: &roxmltree::Node) -> Vec<ArticyConnection> {
    let mut result = Vec::new();
    let Some(conns_node) = find_child(content, "Connections") else {
        return result;
    };
    for conn in conns_node.children().filter(|n| n.has_tag_name("Connection")) {
        let source = conn.attribute("Source").unwrap_or("").to_string();
        let target = conn.attribute("Target").unwrap_or("").to_string();
        if !source.is_empty() && !target.is_empty() {
            result.push(ArticyConnection { source_pin: source, target_pin: target });
        }
    }
    result
}

fn parse_variables(content: &roxmltree::Node) -> Vec<ArticyVariable> {
    let mut result = Vec::new();
    let Some(globals_node) = find_child(content, "GlobalVariables") else {
        return result;
    };
    for vset in globals_node.children().filter(|n| n.has_tag_name("VariableSet")) {
        let set_name = vset.attribute("Name").unwrap_or("");
        for var in vset.children().filter(|n| n.has_tag_name("Variable")) {
            let name = var.attribute("Name").unwrap_or("").to_string();
            let var_type = var.attribute("Type").unwrap_or("String").to_string();
            let value = var.attribute("Value").unwrap_or("").to_string();
            if !name.is_empty() {
                let full_name = if set_name.is_empty() {
                    name
                } else {
                    format!("{set_name}.{name}")
                };
                result.push(ArticyVariable { name: full_name, var_type, value });
            }
        }
    }
    result
}

fn parse_pins(node: &roxmltree::Node) -> (Vec<String>, Vec<String>) {
    let mut inputs = Vec::new();
    let mut outputs = Vec::new();
    let Some(pins_node) = find_child(node, "Pins") else {
        return (inputs, outputs);
    };
    for pin in pins_node.children() {
        if pin.has_tag_name("InputPin") {
            if let Some(id) = pin.attribute("Id") {
                inputs.push(id.to_string());
            }
        } else if pin.has_tag_name("OutputPin") {
            if let Some(id) = pin.attribute("Id") {
                outputs.push(id.to_string());
            }
        }
    }
    (inputs, outputs)
}

// --- XML helpers ---

fn find_child<'a>(node: &'a roxmltree::Node, tag: &str) -> Option<roxmltree::Node<'a, 'a>> {
    node.children().find(|n| n.has_tag_name(tag))
}

fn child_text(node: &roxmltree::Node, tag: &str) -> Option<String> {
    find_child(node, tag).and_then(|n| n.text().map(|s| s.to_string()))
}
