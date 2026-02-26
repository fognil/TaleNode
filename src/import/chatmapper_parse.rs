use std::collections::HashMap;

/// Parsed actor from Chat Mapper XML.
#[derive(Debug)]
pub(super) struct CmActor {
    pub id: String,
    pub name: String,
}

/// Parsed variable from Chat Mapper XML.
#[derive(Debug)]
pub(super) struct CmVariable {
    pub name: String,
    pub var_type: String,
    pub initial_value: String,
}

/// Parsed dialogue entry from Chat Mapper XML.
#[derive(Debug)]
pub(super) struct CmEntry {
    pub id: String,
    pub conv_id: String,
    pub is_root: bool,
    pub actor_id: Option<String>,
    pub text: String,
    pub outgoing_links: Vec<CmLink>,
}

/// A link from one dialogue entry to another.
#[derive(Debug)]
pub(super) struct CmLink {
    pub dest_conv_id: String,
    pub dest_dialog_id: String,
}

/// All data parsed from a Chat Mapper project XML.
#[derive(Debug)]
pub(super) struct CmProject {
    pub actors: Vec<CmActor>,
    pub variables: Vec<CmVariable>,
    pub entries: Vec<CmEntry>,
}

pub(super) fn parse_chatmapper(xml: &str) -> Result<CmProject, String> {
    let doc = roxmltree::Document::parse(xml)
        .map_err(|e| format!("XML parse error: {e}"))?;
    let root = doc.root_element();

    let assets = find_child(&root, "Assets")
        .ok_or("Missing <Assets> element")?;

    let actors = parse_actors(&assets);
    let variables = parse_variables(&assets);
    let entries = parse_conversations(&assets)?;

    Ok(CmProject { actors, variables, entries })
}

fn parse_actors(assets: &roxmltree::Node) -> Vec<CmActor> {
    let mut result = Vec::new();
    let Some(actors_node) = find_child(assets, "Actors") else {
        return result;
    };
    for actor in actors_node.children().filter(|n| n.has_tag_name("Actor")) {
        let id = actor.attribute("ID").unwrap_or("").to_string();
        let name = get_field_value(&actor, "Name").unwrap_or_default();
        if !name.is_empty() {
            result.push(CmActor { id, name });
        }
    }
    result
}

fn parse_variables(assets: &roxmltree::Node) -> Vec<CmVariable> {
    let mut result = Vec::new();
    let Some(vars_node) = find_child(assets, "Variables") else {
        return result;
    };
    for var in vars_node.children().filter(|n| n.has_tag_name("Variable")) {
        let name = get_field_value(&var, "Name").unwrap_or_default();
        let var_type = get_field_value(&var, "Type").unwrap_or_default();
        let initial = get_field_value(&var, "Initial Value").unwrap_or_default();
        if !name.is_empty() {
            result.push(CmVariable {
                name,
                var_type,
                initial_value: initial,
            });
        }
    }
    result
}

fn parse_conversations(assets: &roxmltree::Node) -> Result<Vec<CmEntry>, String> {
    let mut all_entries = Vec::new();
    let Some(convs_node) = find_child(assets, "Conversations") else {
        return Ok(all_entries);
    };
    for conv in convs_node.children().filter(|n| n.has_tag_name("Conversation")) {
        let conv_id = conv.attribute("ID").unwrap_or("0").to_string();
        let Some(entries_node) = find_child(&conv, "DialogEntries") else {
            continue;
        };
        for entry in entries_node.children().filter(|n| n.has_tag_name("DialogEntry")) {
            let entry_id = entry.attribute("ID").unwrap_or("0").to_string();
            let is_root = entry.attribute("IsRoot").unwrap_or("false") == "true";
            let actor_id = get_field_value(&entry, "Actor");
            let text = get_field_value(&entry, "Dialogue Text")
                .or_else(|| get_field_value(&entry, "Menu Text"))
                .unwrap_or_default();

            let outgoing_links = parse_outgoing_links(&entry);

            all_entries.push(CmEntry {
                id: entry_id,
                conv_id: conv_id.clone(),
                is_root,
                actor_id,
                text,
                outgoing_links,
            });
        }
    }
    Ok(all_entries)
}

fn parse_outgoing_links(entry: &roxmltree::Node) -> Vec<CmLink> {
    let mut links = Vec::new();
    let Some(links_node) = find_child(entry, "OutgoingLinks") else {
        return links;
    };
    for link in links_node.children().filter(|n| n.has_tag_name("Link")) {
        let dest_conv = child_text(&link, "DestinationConvoID").unwrap_or_default();
        let dest_dialog = child_text(&link, "DestinationDialogID").unwrap_or_default();
        links.push(CmLink {
            dest_conv_id: dest_conv,
            dest_dialog_id: dest_dialog,
        });
    }
    links
}

// --- XML helpers ---

fn find_child<'a>(node: &'a roxmltree::Node, tag: &str) -> Option<roxmltree::Node<'a, 'a>> {
    node.children().find(|n| n.has_tag_name(tag))
}

fn child_text(node: &roxmltree::Node, tag: &str) -> Option<String> {
    find_child(node, tag).and_then(|n| n.text().map(|s| s.to_string()))
}

/// Get the value of a Chat Mapper Field by title.
pub(super) fn get_field_value(node: &roxmltree::Node, title: &str) -> Option<String> {
    let fields = find_child(node, "Fields")?;
    for field in fields.children().filter(|n| n.has_tag_name("Field")) {
        let Some(title_node) = find_child(&field, "Title") else { continue };
        if title_node.text().is_some_and(|t| t == title) {
            if let Some(val_node) = find_child(&field, "Value") {
                return val_node.text().map(|s| s.to_string());
            }
        }
    }
    None
}

/// Build a lookup key for dialogue entries: "conv_id:entry_id".
pub(super) fn entry_key(conv_id: &str, entry_id: &str) -> String {
    let mut key = String::with_capacity(conv_id.len() + 1 + entry_id.len());
    key.push_str(conv_id);
    key.push(':');
    key.push_str(entry_id);
    key
}

/// Build entry_key -> index lookup map.
pub(super) fn build_entry_map(entries: &[CmEntry]) -> HashMap<String, usize> {
    let mut map = HashMap::new();
    for (i, e) in entries.iter().enumerate() {
        map.insert(entry_key(&e.conv_id, &e.id), i);
    }
    map
}

#[cfg(test)]
mod tests {
    use super::*;

    fn full_xml() -> &'static str {
        r#"<?xml version="1.0"?>
<ChatMapperProject>
  <Assets>
    <Actors>
      <Actor ID="1">
        <Fields><Field><Title>Name</Title><Value>Alice</Value></Field></Fields>
      </Actor>
      <Actor ID="2">
        <Fields><Field><Title>Name</Title><Value>Bob</Value></Field></Fields>
      </Actor>
    </Actors>
    <Variables>
      <Variable>
        <Fields>
          <Field><Title>Name</Title><Value>mood</Value></Field>
          <Field><Title>Type</Title><Value>Text</Value></Field>
          <Field><Title>Initial Value</Title><Value>happy</Value></Field>
        </Fields>
      </Variable>
    </Variables>
    <Conversations>
      <Conversation ID="10">
        <DialogEntries>
          <DialogEntry ID="0" IsRoot="true">
            <Fields>
              <Field><Title>Dialogue Text</Title><Value>Hello!</Value></Field>
              <Field><Title>Actor</Title><Value>1</Value></Field>
            </Fields>
            <OutgoingLinks>
              <Link><DestinationConvoID>10</DestinationConvoID>
                    <DestinationDialogID>1</DestinationDialogID></Link>
            </OutgoingLinks>
          </DialogEntry>
          <DialogEntry ID="1" IsRoot="false">
            <Fields>
              <Field><Title>Dialogue Text</Title><Value>Hi there.</Value></Field>
              <Field><Title>Actor</Title><Value>2</Value></Field>
            </Fields>
            <OutgoingLinks/>
          </DialogEntry>
        </DialogEntries>
      </Conversation>
    </Conversations>
  </Assets>
</ChatMapperProject>"#
    }

    #[test]
    fn parse_valid_project() {
        let proj = parse_chatmapper(full_xml()).unwrap();
        assert_eq!(proj.actors.len(), 2);
        assert_eq!(proj.actors[0].name, "Alice");
        assert_eq!(proj.actors[1].id, "2");
        assert_eq!(proj.variables.len(), 1);
        assert_eq!(proj.variables[0].name, "mood");
        assert_eq!(proj.entries.len(), 2);
        assert_eq!(proj.entries[0].text, "Hello!");
        assert_eq!(proj.entries[0].outgoing_links.len(), 1);
        assert_eq!(proj.entries[0].outgoing_links[0].dest_dialog_id, "1");
        assert_eq!(proj.entries[1].text, "Hi there.");
    }

    #[test]
    fn parse_missing_assets_returns_error() {
        let xml = r#"<?xml version="1.0"?><ChatMapperProject></ChatMapperProject>"#;
        match parse_chatmapper(xml) {
            Err(e) => assert!(e.contains("Assets"), "expected Assets error, got: {e}"),
            Ok(_) => panic!("expected error for missing Assets"),
        }
    }

    #[test]
    fn parse_invalid_xml_returns_error() {
        match parse_chatmapper("<not-closed") {
            Err(e) => assert!(e.contains("XML parse error")),
            Ok(_) => panic!("expected error for malformed XML"),
        }
    }

    #[test]
    fn parse_actors_skips_nameless() {
        let xml = r#"<?xml version="1.0"?>
<ChatMapperProject><Assets>
  <Actors>
    <Actor ID="1"><Fields></Fields></Actor>
    <Actor ID="2">
      <Fields><Field><Title>Name</Title><Value>Valid</Value></Field></Fields>
    </Actor>
  </Actors>
  <Conversations/>
</Assets></ChatMapperProject>"#;
        let proj = parse_chatmapper(xml).unwrap();
        assert_eq!(proj.actors.len(), 1);
        assert_eq!(proj.actors[0].name, "Valid");
    }

    #[test]
    fn parse_variables_extracts_types() {
        let xml = r#"<?xml version="1.0"?>
<ChatMapperProject><Assets>
  <Variables>
    <Variable><Fields>
      <Field><Title>Name</Title><Value>hp</Value></Field>
      <Field><Title>Type</Title><Value>Number</Value></Field>
      <Field><Title>Initial Value</Title><Value>100</Value></Field>
    </Fields></Variable>
    <Variable><Fields>
      <Field><Title>Name</Title><Value>alive</Value></Field>
      <Field><Title>Type</Title><Value>Boolean</Value></Field>
      <Field><Title>Initial Value</Title><Value>true</Value></Field>
    </Fields></Variable>
  </Variables>
  <Conversations/>
</Assets></ChatMapperProject>"#;
        let proj = parse_chatmapper(xml).unwrap();
        assert_eq!(proj.variables.len(), 2);
        assert_eq!(proj.variables[0].var_type, "Number");
        assert_eq!(proj.variables[0].initial_value, "100");
        assert_eq!(proj.variables[1].var_type, "Boolean");
        assert_eq!(proj.variables[1].initial_value, "true");
    }

    #[test]
    fn parse_conversations_with_root_entry() {
        let proj = parse_chatmapper(full_xml()).unwrap();
        assert!(proj.entries[0].is_root);
        assert_eq!(proj.entries[0].conv_id, "10");
        assert_eq!(proj.entries[0].actor_id.as_deref(), Some("1"));
        assert!(!proj.entries[1].is_root);
    }

    #[test]
    fn entry_key_format() {
        assert_eq!(entry_key("10", "3"), "10:3");
        assert_eq!(entry_key("abc", "xyz"), "abc:xyz");
    }

    #[test]
    fn build_entry_map_lookup() {
        let entries = vec![
            CmEntry {
                id: "0".into(), conv_id: "1".into(), is_root: true,
                actor_id: None, text: String::new(), outgoing_links: vec![],
            },
            CmEntry {
                id: "5".into(), conv_id: "2".into(), is_root: false,
                actor_id: None, text: String::new(), outgoing_links: vec![],
            },
        ];
        let map = build_entry_map(&entries);
        assert_eq!(map.len(), 2);
        assert_eq!(map["1:0"], 0);
        assert_eq!(map["2:5"], 1);
    }

    #[test]
    fn parse_empty_sections() {
        let xml = r#"<?xml version="1.0"?>
<ChatMapperProject><Assets>
  <Actors/><Variables/><Conversations/>
</Assets></ChatMapperProject>"#;
        let proj = parse_chatmapper(xml).unwrap();
        assert!(proj.actors.is_empty());
        assert!(proj.variables.is_empty());
        assert!(proj.entries.is_empty());
    }
}
