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

#[cfg(test)]
mod tests {
    use super::*;

    fn full_xml() -> String {
        r#"<?xml version="1.0"?>
<ArticyData>
  <Content>
    <Entities>
      <Entity Id="ent_1" DisplayName="AttrNPC">
        <DisplayName>Hero</DisplayName>
      </Entity>
      <Entity Id="ent_2">
        <DisplayName>Villain</DisplayName>
      </Entity>
    </Entities>
    <DialogueFragments>
      <DialogueFragment Id="frag_1" Speaker="ent_1">
        <Text>Hello world</Text>
        <Pins>
          <InputPin Id="pin_in_1"/>
          <OutputPin Id="pin_out_1"/>
        </Pins>
      </DialogueFragment>
    </DialogueFragments>
    <Hubs>
      <Hub Id="hub_1" DisplayName="Branch A">
        <Pins>
          <InputPin Id="pin_h_in"/>
          <OutputPin Id="pin_h_out1"/>
          <OutputPin Id="pin_h_out2"/>
        </Pins>
      </Hub>
    </Hubs>
    <Connections>
      <Connection Source="pin_out_1" Target="pin_h_in"/>
      <Connection Source="" Target="pin_h_in"/>
      <Connection Source="pin_h_out1" Target="pin_in_1"/>
    </Connections>
    <GlobalVariables>
      <VariableSet Name="Quest">
        <Variable Name="Started" Type="Boolean" Value="false"/>
        <Variable Name="Gold" Type="Integer" Value="100"/>
      </VariableSet>
    </GlobalVariables>
  </Content>
</ArticyData>"#.to_string()
    }

    #[test]
    fn parse_valid_project() {
        let proj = parse_articy(&full_xml()).unwrap();
        assert_eq!(proj.entities.len(), 2);
        assert_eq!(proj.fragments.len(), 1);
        assert_eq!(proj.hubs.len(), 1);
        // Connection with empty Source is skipped
        assert_eq!(proj.connections.len(), 2);
        assert_eq!(proj.variables.len(), 2);
    }

    #[test]
    fn parse_missing_content_returns_error() {
        let xml = r#"<?xml version="1.0"?><ArticyData><Other/></ArticyData>"#;
        match parse_articy(xml) {
            Err(e) => assert!(e.contains("Content"), "expected Content mention, got: {e}"),
            Ok(_) => panic!("expected error for missing Content"),
        }
    }

    #[test]
    fn parse_invalid_xml_returns_error() {
        match parse_articy("<broken><<>") {
            Err(e) => assert!(e.contains("XML parse error")),
            Ok(_) => panic!("expected error for malformed XML"),
        }
    }

    #[test]
    fn parse_entities_extracts_names() {
        // Child element DisplayName takes priority over attribute
        let proj = parse_articy(&full_xml()).unwrap();
        assert_eq!(proj.entities[0].name, "Hero");
        assert_eq!(proj.entities[1].name, "Villain");
        assert_eq!(proj.entities[0].id, "ent_1");
    }

    #[test]
    fn parse_fragments_with_speaker_and_pins() {
        let proj = parse_articy(&full_xml()).unwrap();
        let frag = &proj.fragments[0];
        assert_eq!(frag.id, "frag_1");
        assert_eq!(frag.speaker_id.as_deref(), Some("ent_1"));
        assert_eq!(frag.text, "Hello world");
        assert_eq!(frag.input_pins, vec!["pin_in_1"]);
        assert_eq!(frag.output_pins, vec!["pin_out_1"]);
    }

    #[test]
    fn parse_hubs_with_display_name() {
        let proj = parse_articy(&full_xml()).unwrap();
        let hub = &proj.hubs[0];
        assert_eq!(hub.id, "hub_1");
        assert_eq!(hub.display_name, "Branch A");
        assert_eq!(hub.input_pins, vec!["pin_h_in"]);
        assert_eq!(hub.output_pins, vec!["pin_h_out1", "pin_h_out2"]);
    }

    #[test]
    fn parse_connections_filters_empty_source() {
        let proj = parse_articy(&full_xml()).unwrap();
        // The connection with Source="" should be filtered out
        assert_eq!(proj.connections.len(), 2);
        assert_eq!(proj.connections[0].source_pin, "pin_out_1");
        assert_eq!(proj.connections[0].target_pin, "pin_h_in");
        assert_eq!(proj.connections[1].source_pin, "pin_h_out1");
    }

    #[test]
    fn parse_variables_with_set_prefix() {
        let proj = parse_articy(&full_xml()).unwrap();
        assert_eq!(proj.variables[0].name, "Quest.Started");
        assert_eq!(proj.variables[0].var_type, "Boolean");
        assert_eq!(proj.variables[0].value, "false");
        assert_eq!(proj.variables[1].name, "Quest.Gold");
        assert_eq!(proj.variables[1].var_type, "Integer");
        assert_eq!(proj.variables[1].value, "100");
    }

    #[test]
    fn parse_empty_sections() {
        let xml = r#"<?xml version="1.0"?>
<ArticyData>
  <Content>
    <Entities/>
    <DialogueFragments/>
    <Hubs/>
    <Connections/>
    <GlobalVariables/>
  </Content>
</ArticyData>"#;
        let proj = parse_articy(xml).unwrap();
        assert!(proj.entities.is_empty());
        assert!(proj.fragments.is_empty());
        assert!(proj.hubs.is_empty());
        assert!(proj.connections.is_empty());
        assert!(proj.variables.is_empty());
    }
}
