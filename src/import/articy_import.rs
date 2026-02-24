use std::collections::{HashMap, HashSet};
use uuid::Uuid;

use crate::model::character::Character;
use crate::model::graph::DialogueGraph;
use crate::model::node::*;
use crate::model::node::Node;
use crate::model::port::{Port, PortId};
use crate::model::variable::{Variable, VariableType};

use super::articy_parse::{ArticyProject, parse_articy};

/// Import an articy:draft XML export into a DialogueGraph.
pub fn import_articy(xml: &str) -> Result<DialogueGraph, String> {
    let project = parse_articy(xml)?;
    build_graph(project)
}

fn build_graph(project: ArticyProject) -> Result<DialogueGraph, String> {
    let mut graph = DialogueGraph::new();

    // Map articy entity ID -> Character UUID
    let mut entity_map: HashMap<String, Uuid> = HashMap::new();
    for entity in &project.entities {
        let ch = Character::new(&entity.name);
        entity_map.insert(entity.id.clone(), ch.id);
        graph.characters.push(ch);
    }

    // Variables
    for var in &project.variables {
        let (var_type, default_value) = convert_articy_variable(&var.var_type, &var.value);
        graph.variables.push(Variable {
            id: Uuid::new_v4(),
            name: var.name.clone(),
            var_type,
            default_value,
        });
    }

    // Build pin -> (node_uuid, port_uuid) map and create nodes
    // Key: articy pin ID, Value: (talenode node uuid, talenode port uuid)
    let mut pin_map: HashMap<String, (Uuid, Uuid)> = HashMap::new();

    // Track which pins have incoming/outgoing connections (for Start/End detection)
    let mut pins_with_incoming: HashSet<String> = HashSet::new();
    let mut pins_with_outgoing: HashSet<String> = HashSet::new();
    for conn in &project.connections {
        pins_with_outgoing.insert(conn.source_pin.clone());
        pins_with_incoming.insert(conn.target_pin.clone());
    }

    // Collect all input pin IDs per node for "has incoming" check
    let mut node_has_incoming: HashSet<String> = HashSet::new(); // articy node ID
    let mut node_has_outgoing: HashSet<String> = HashSet::new();

    // Check fragments
    for frag in &project.fragments {
        for pin in &frag.input_pins {
            if pins_with_incoming.contains(pin) {
                node_has_incoming.insert(frag.id.clone());
            }
        }
        for pin in &frag.output_pins {
            if pins_with_outgoing.contains(pin) {
                node_has_outgoing.insert(frag.id.clone());
            }
        }
    }
    // Check hubs
    for hub in &project.hubs {
        for pin in &hub.input_pins {
            if pins_with_incoming.contains(pin) {
                node_has_incoming.insert(hub.id.clone());
            }
        }
        for pin in &hub.output_pins {
            if pins_with_outgoing.contains(pin) {
                node_has_outgoing.insert(hub.id.clone());
            }
        }
    }

    let mut y_offset: f32 = 0.0;

    // Create nodes for fragments
    for frag in &project.fragments {
        let is_start = !node_has_incoming.contains(&frag.id);
        let is_end = !node_has_outgoing.contains(&frag.id);

        let node = if is_start {
            Node::new_start([0.0, y_offset])
        } else if is_end {
            let mut n = Node::new_end([0.0, y_offset]);
            if let NodeType::End(ref mut data) = n.node_type {
                if !frag.text.is_empty() {
                    data.tag = frag.text.clone();
                }
            }
            n
        } else {
            let mut n = Node::new_dialogue([0.0, y_offset]);
            if let NodeType::Dialogue(ref mut data) = n.node_type {
                data.text = frag.text.clone();
                if let Some(ref speaker_id) = frag.speaker_id {
                    data.speaker_id = entity_map.get(speaker_id).copied();
                }
            }
            n
        };

        register_pins(&node, &frag.input_pins, &frag.output_pins, &mut pin_map);
        graph.add_node(node);
        y_offset += 200.0;
    }

    // Create nodes for hubs
    for hub in &project.hubs {
        let mut choices = Vec::new();
        let mut outputs = Vec::new();

        for (i, _) in hub.output_pins.iter().enumerate() {
            let label = format!("Option {}", i + 1);
            choices.push(ChoiceOption {
                id: Uuid::new_v4(),
                text: label.clone(),
                condition: None,
            });
            outputs.push(Port::output_with_label(&label));
        }
        if outputs.is_empty() {
            outputs.push(Port::output_with_label("Option 1"));
            choices.push(ChoiceOption {
                id: Uuid::new_v4(),
                text: "Option 1".to_string(),
                condition: None,
            });
        }

        let node = Node {
            id: Uuid::new_v4(),
            node_type: NodeType::Choice(ChoiceData {
                prompt: hub.display_name.clone(),
                choices,
            }),
            position: [0.0, y_offset],
            collapsed: false,
            inputs: vec![Port::input()],
            outputs,
        };

        register_pins(&node, &hub.input_pins, &hub.output_pins, &mut pin_map);
        graph.add_node(node);
        y_offset += 200.0;
    }

    // Wire connections using pin map
    for conn in &project.connections {
        let Some(&(from_node, from_port)) = pin_map.get(&conn.source_pin) else {
            continue;
        };
        let Some(&(to_node, to_port)) = pin_map.get(&conn.target_pin) else {
            continue;
        };
        graph.add_connection(from_node, PortId(from_port), to_node, PortId(to_port));
    }

    Ok(graph)
}

fn register_pins(
    node: &Node,
    articy_input_pins: &[String],
    articy_output_pins: &[String],
    pin_map: &mut HashMap<String, (Uuid, Uuid)>,
) {
    // Map articy input pins to node's input ports
    for (i, articy_pin) in articy_input_pins.iter().enumerate() {
        if let Some(port) = node.inputs.get(i) {
            pin_map.insert(articy_pin.clone(), (node.id, port.id.0));
        } else if let Some(port) = node.inputs.first() {
            // Fallback: map all input pins to the single input port
            pin_map.insert(articy_pin.clone(), (node.id, port.id.0));
        }
    }
    // Map articy output pins to node's output ports
    for (i, articy_pin) in articy_output_pins.iter().enumerate() {
        if let Some(port) = node.outputs.get(i) {
            pin_map.insert(articy_pin.clone(), (node.id, port.id.0));
        } else if let Some(port) = node.outputs.first() {
            pin_map.insert(articy_pin.clone(), (node.id, port.id.0));
        }
    }
}

fn convert_articy_variable(type_str: &str, value: &str) -> (VariableType, VariableValue) {
    match type_str.to_lowercase().as_str() {
        "boolean" | "bool" => {
            let val = value.eq_ignore_ascii_case("true") || value == "1";
            (VariableType::Bool, VariableValue::Bool(val))
        }
        "integer" | "int" => {
            let val = value.parse::<i64>().unwrap_or(0);
            (VariableType::Int, VariableValue::Int(val))
        }
        "float" | "double" => {
            let val = value.parse::<f64>().unwrap_or(0.0);
            (VariableType::Float, VariableValue::Float(val))
        }
        _ => {
            (VariableType::Text, VariableValue::Text(value.to_string()))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SIMPLE_XML: &str = r#"<?xml version="1.0"?>
<ExportPackage>
  <Content>
    <Entities>
      <Entity Id="0x01" DisplayName="Guard"/>
    </Entities>
    <DialogueFragments>
      <DialogueFragment Id="0x10" Speaker="0x01">
        <Text>Hello, traveler!</Text>
        <Pins>
          <InputPin Id="0xA1"/>
          <OutputPin Id="0xA2"/>
        </Pins>
      </DialogueFragment>
      <DialogueFragment Id="0x11">
        <Text>Farewell!</Text>
        <Pins>
          <InputPin Id="0xB1"/>
        </Pins>
      </DialogueFragment>
    </DialogueFragments>
    <Hubs></Hubs>
    <Connections>
      <Connection Source="0xA2" Target="0xB1"/>
    </Connections>
    <GlobalVariables></GlobalVariables>
  </Content>
</ExportPackage>"#;

    #[test]
    fn import_dialogue_fragments() {
        let graph = import_articy(SIMPLE_XML).expect("should parse");
        // 0x10 has no incoming -> Start, 0x11 has no outgoing -> End
        assert!(graph.nodes.len() >= 2);
        assert_eq!(graph.characters.len(), 1);
        assert_eq!(graph.characters[0].name, "Guard");
    }

    #[test]
    fn import_creates_connections() {
        let graph = import_articy(SIMPLE_XML).expect("should parse");
        assert!(!graph.connections.is_empty());
    }

    #[test]
    fn import_hub_branching() {
        let xml = r#"<?xml version="1.0"?>
<ExportPackage>
  <Content>
    <Entities></Entities>
    <DialogueFragments>
      <DialogueFragment Id="0x10">
        <Text>Hi</Text>
        <Pins><OutputPin Id="0xA1"/></Pins>
      </DialogueFragment>
      <DialogueFragment Id="0x20">
        <Text>Path A</Text>
        <Pins><InputPin Id="0xC1"/></Pins>
      </DialogueFragment>
      <DialogueFragment Id="0x21">
        <Text>Path B</Text>
        <Pins><InputPin Id="0xC2"/></Pins>
      </DialogueFragment>
    </DialogueFragments>
    <Hubs>
      <Hub Id="0x30" DisplayName="Branch">
        <Pins>
          <InputPin Id="0xB1"/>
          <OutputPin Id="0xB2" Index="0"/>
          <OutputPin Id="0xB3" Index="1"/>
        </Pins>
      </Hub>
    </Hubs>
    <Connections>
      <Connection Source="0xA1" Target="0xB1"/>
      <Connection Source="0xB2" Target="0xC1"/>
      <Connection Source="0xB3" Target="0xC2"/>
    </Connections>
    <GlobalVariables></GlobalVariables>
  </Content>
</ExportPackage>"#;
        let graph = import_articy(xml).expect("should parse");
        let choice_count = graph.nodes.values().filter(|n| {
            matches!(n.node_type, NodeType::Choice(_))
        }).count();
        assert_eq!(choice_count, 1);
        assert!(graph.connections.len() >= 2);
    }

    #[test]
    fn import_variables() {
        let xml = r#"<?xml version="1.0"?>
<ExportPackage>
  <Content>
    <Entities></Entities>
    <DialogueFragments></DialogueFragments>
    <Hubs></Hubs>
    <Connections></Connections>
    <GlobalVariables>
      <VariableSet Name="Game">
        <Variable Name="gold" Type="Integer" Value="50"/>
        <Variable Name="hasKey" Type="Boolean" Value="false"/>
      </VariableSet>
    </GlobalVariables>
  </Content>
</ExportPackage>"#;
        let graph = import_articy(xml).expect("should parse");
        assert_eq!(graph.variables.len(), 2);
        let names: Vec<&str> = graph.variables.iter().map(|v| v.name.as_str()).collect();
        assert!(names.contains(&"Game.gold"));
        assert!(names.contains(&"Game.hasKey"));
    }

    #[test]
    fn import_malformed_xml() {
        assert!(import_articy("not xml at all").is_err());
    }

    #[test]
    fn import_empty_content() {
        let xml = r#"<?xml version="1.0"?>
<ExportPackage>
  <Content>
    <Entities></Entities>
    <DialogueFragments></DialogueFragments>
    <Hubs></Hubs>
    <Connections></Connections>
    <GlobalVariables></GlobalVariables>
  </Content>
</ExportPackage>"#;
        let graph = import_articy(xml).expect("should parse empty");
        assert!(graph.nodes.is_empty());
    }
}
