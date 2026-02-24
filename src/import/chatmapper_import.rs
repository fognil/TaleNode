use std::collections::HashMap;
use uuid::Uuid;

use crate::model::character::Character;
use crate::model::graph::DialogueGraph;
use crate::model::node::*;
use crate::model::node::Node;
use crate::model::port::Port;
use crate::model::variable::{Variable, VariableType};

use super::chatmapper_parse::{
    CmEntry, CmProject, build_entry_map, entry_key, parse_chatmapper,
};

/// Import a Chat Mapper XML file into a DialogueGraph.
pub fn import_chatmapper(xml: &str) -> Result<DialogueGraph, String> {
    let project = parse_chatmapper(xml)?;
    build_graph(project)
}

fn build_graph(project: CmProject) -> Result<DialogueGraph, String> {
    let mut graph = DialogueGraph::new();

    // Map CM actor ID -> Character UUID
    let mut actor_map: HashMap<String, Uuid> = HashMap::new();
    for actor in &project.actors {
        let ch = Character::new(&actor.name);
        actor_map.insert(actor.id.clone(), ch.id);
        graph.characters.push(ch);
    }
    // Also map actor names for the name -> UUID lookup
    let mut actor_name_map: HashMap<String, String> = HashMap::new();
    for actor in &project.actors {
        actor_name_map.insert(actor.name.clone(), actor.id.clone());
    }

    // Variables
    for var in &project.variables {
        let (var_type, default_value) = convert_cm_variable(&var.var_type, &var.initial_value);
        graph.variables.push(Variable {
            id: Uuid::new_v4(),
            name: var.name.clone(),
            var_type,
            default_value,
        });
    }

    let entry_map = build_entry_map(&project.entries);

    // Create TaleNode nodes for each CM entry
    let mut node_ids: HashMap<String, Uuid> = HashMap::new();

    for (i, entry) in project.entries.iter().enumerate() {
        let key = entry_key(&entry.conv_id, &entry.id);
        let x = conv_column(&entry.conv_id) * 400.0;
        let y = i as f32 * 200.0;

        let node = create_node_for_entry(entry, [x, y], &actor_map, &project.entries, &entry_map);
        let node_id = node.id;
        node_ids.insert(key, node_id);
        graph.add_node(node);
    }

    // Wire connections
    for entry in &project.entries {
        let from_key = entry_key(&entry.conv_id, &entry.id);
        let Some(&from_id) = node_ids.get(&from_key) else { continue };

        for (port_idx, link) in entry.outgoing_links.iter().enumerate() {
            let to_key = entry_key(&link.dest_conv_id, &link.dest_dialog_id);
            let Some(&to_id) = node_ids.get(&to_key) else { continue };
            connect_nodes(&mut graph, from_id, port_idx, to_id);
        }
    }

    Ok(graph)
}

fn create_node_for_entry(
    entry: &CmEntry,
    position: [f32; 2],
    actor_map: &HashMap<String, Uuid>,
    all_entries: &[CmEntry],
    entry_map: &HashMap<String, usize>,
) -> Node {
    if entry.is_root {
        return Node::new_start(position);
    }

    if entry.outgoing_links.is_empty() {
        let mut node = Node::new_end(position);
        if let NodeType::End(ref mut data) = node.node_type {
            if !entry.text.is_empty() {
                data.tag = entry.text.clone();
            }
        }
        return node;
    }

    if entry.outgoing_links.len() >= 2 {
        return create_choice_node(entry, position, all_entries, entry_map);
    }

    // Single outgoing link = dialogue node
    let mut node = Node::new_dialogue(position);
    if let NodeType::Dialogue(ref mut data) = node.node_type {
        data.text = entry.text.clone();
        if let Some(ref actor_id_str) = entry.actor_id {
            if let Some(&char_uuid) = actor_map.get(actor_id_str) {
                data.speaker_id = Some(char_uuid);
            }
        }
    }
    node
}

fn create_choice_node(
    entry: &CmEntry,
    position: [f32; 2],
    all_entries: &[CmEntry],
    entry_map: &HashMap<String, usize>,
) -> Node {
    let mut choices = Vec::new();
    let mut outputs = Vec::new();

    for link in &entry.outgoing_links {
        let link_key = entry_key(&link.dest_conv_id, &link.dest_dialog_id);
        let option_text = entry_map
            .get(&link_key)
            .and_then(|&idx| all_entries.get(idx))
            .map(|e| {
                if e.text.is_empty() {
                    format!("Option {}", choices.len() + 1)
                } else {
                    e.text.clone()
                }
            })
            .unwrap_or_else(|| format!("Option {}", choices.len() + 1));

        choices.push(ChoiceOption {
            id: Uuid::new_v4(),
            text: option_text.clone(),
            condition: None,
        });
        outputs.push(Port::output_with_label(&option_text));
    }

    Node {
        id: Uuid::new_v4(),
        node_type: NodeType::Choice(ChoiceData {
            prompt: entry.text.clone(),
            choices,
        }),
        position,
        collapsed: false,
        inputs: vec![Port::input()],
        outputs,
    }
}

fn connect_nodes(graph: &mut DialogueGraph, from_id: Uuid, from_port_idx: usize, to_id: Uuid) {
    let from_port = {
        let node = match graph.nodes.get(&from_id) {
            Some(n) => n,
            None => return,
        };
        match node.outputs.get(from_port_idx) {
            Some(p) => p.id,
            None => return,
        }
    };
    let to_port = {
        let node = match graph.nodes.get(&to_id) {
            Some(n) => n,
            None => return,
        };
        match node.inputs.first() {
            Some(p) => p.id,
            None => return,
        }
    };
    graph.add_connection(from_id, from_port, to_id, to_port);
}

fn convert_cm_variable(type_str: &str, initial: &str) -> (VariableType, VariableValue) {
    match type_str.to_lowercase().as_str() {
        "boolean" | "bool" => {
            let val = initial.eq_ignore_ascii_case("true");
            (VariableType::Bool, VariableValue::Bool(val))
        }
        "number" | "int" | "integer" => {
            let val = initial.parse::<i64>().unwrap_or(0);
            (VariableType::Int, VariableValue::Int(val))
        }
        "float" | "double" => {
            (VariableType::Float, VariableValue::Float(
                initial.parse::<f64>().unwrap_or(0.0),
            ))
        }
        _ => {
            (VariableType::Text, VariableValue::Text(initial.to_string()))
        }
    }
}

fn conv_column(conv_id: &str) -> f32 {
    conv_id.parse::<f32>().unwrap_or(0.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    const SIMPLE_XML: &str = r#"<?xml version="1.0"?>
<ChatMapperProject Title="Test" Version="1.0">
  <Assets>
    <Actors>
      <Actor ID="1"><Fields>
        <Field><Title>Name</Title><Value>Guard</Value></Field>
      </Fields></Actor>
    </Actors>
    <Variables></Variables>
    <Conversations>
      <Conversation ID="1">
        <DialogEntries>
          <DialogEntry ID="0" IsRoot="true" IsGroup="false">
            <Fields>
              <Field><Title>Dialogue Text</Title><Value></Value></Field>
              <Field><Title>Actor</Title><Value>1</Value></Field>
            </Fields>
            <OutgoingLinks>
              <Link><DestinationConvoID>1</DestinationConvoID>
                <DestinationDialogID>1</DestinationDialogID></Link>
            </OutgoingLinks>
          </DialogEntry>
          <DialogEntry ID="1" IsRoot="false" IsGroup="false">
            <Fields>
              <Field><Title>Dialogue Text</Title><Value>Hello there!</Value></Field>
              <Field><Title>Actor</Title><Value>1</Value></Field>
            </Fields>
            <OutgoingLinks></OutgoingLinks>
          </DialogEntry>
        </DialogEntries>
      </Conversation>
    </Conversations>
  </Assets>
</ChatMapperProject>"#;

    #[test]
    fn import_simple_conversation() {
        let graph = import_chatmapper(SIMPLE_XML).expect("should parse");
        assert!(graph.nodes.len() >= 2);
        assert_eq!(graph.characters.len(), 1);
        assert_eq!(graph.characters[0].name, "Guard");
    }

    #[test]
    fn import_creates_connections() {
        let graph = import_chatmapper(SIMPLE_XML).expect("should parse");
        assert!(!graph.connections.is_empty());
    }

    #[test]
    fn import_variables() {
        let xml = r#"<?xml version="1.0"?>
<ChatMapperProject Title="Test" Version="1.0">
  <Assets>
    <Actors></Actors>
    <Variables>
      <Variable><Fields>
        <Field><Title>Name</Title><Value>gold</Value></Field>
        <Field><Title>Type</Title><Value>Number</Value></Field>
        <Field><Title>Initial Value</Title><Value>100</Value></Field>
      </Fields></Variable>
    </Variables>
    <Conversations></Conversations>
  </Assets>
</ChatMapperProject>"#;
        let graph = import_chatmapper(xml).expect("should parse");
        assert_eq!(graph.variables.len(), 1);
        assert_eq!(graph.variables[0].name, "gold");
        assert_eq!(graph.variables[0].var_type, VariableType::Int);
    }

    #[test]
    fn import_malformed_xml() {
        assert!(import_chatmapper("not xml").is_err());
    }

    #[test]
    fn import_missing_assets() {
        let xml = r#"<?xml version="1.0"?><Root></Root>"#;
        assert!(import_chatmapper(xml).is_err());
    }
}
