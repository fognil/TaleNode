mod act1;
mod act2;
mod act3;
mod characters;
mod metadata;
mod variables;

#[cfg(test)]
mod tests {
    use crate::model::graph::DialogueGraph;
    use crate::model::node::*;
    use crate::model::project::Project;
    use crate::model::version::VersionSnapshot;

    use super::act1::build_act1;
    use super::act2::build_act2;
    use super::act3::build_act3;
    use super::characters::build_characters;
    use super::metadata::{
        apply_comments, apply_reviews, apply_tags, build_barks,
        build_groups, build_locale, build_quests, build_timelines,
        build_world_entities,
    };
    use super::variables::build_variables;

    #[test]
    #[ignore] // Run manually: cargo test generate_constellation -- --ignored
    fn generate_constellation_project() {
        let chars = build_characters();
        let mut graph = DialogueGraph::new();

        // Characters
        graph.characters = chars.all_cloned();

        // Variables
        graph.variables = build_variables();

        // Build acts (adds nodes + connections to graph)
        let a1 = build_act1(&mut graph, &chars);
        let a2 = build_act2(&mut graph, &chars);
        let a3 = build_act3(&mut graph, &chars);

        // --- Bridge acts together ---
        // Act 1 → Act 2: remove act1 End, wire farewell → Event → hub
        graph.remove_node(a1.act1_end);
        let mut bridge12 = Node::new_event([3000.0, 500.0]);
        if let NodeType::Event(ref mut e) = bridge12.node_type {
            e.actions = vec![EventAction {
                action_type: EventActionType::SetVariable,
                key: "chapter".into(),
                value: VariableValue::Int(2),
            }];
        }
        let b12_in = bridge12.inputs[0].id;
        let b12_out = bridge12.outputs[0].id;
        let b12_id = bridge12.id;
        let fw_out = graph.nodes[&a1.farewell].outputs[0].id;
        let hub_in = graph.nodes[&a2.hub].inputs[0].id;
        graph.add_node(bridge12);
        graph.add_connection(a1.farewell, fw_out, b12_id, b12_in);
        graph.add_connection(b12_id, b12_out, a2.hub, hub_in);

        // Act 2 → Act 3: find saya's event node, remove its End,
        // wire evt_saya → Event(chapter=3) → act3 condition
        let saya_end_id = a2.end_saya;
        // Find the node that connects TO saya's end
        let evt_saya_id = graph.connections.iter()
            .find(|c| c.to_node == saya_end_id)
            .map(|c| c.from_node)
            .unwrap();
        graph.remove_node(saya_end_id);
        let mut bridge23 = Node::new_event([5300.0, 1250.0]);
        if let NodeType::Event(ref mut e) = bridge23.node_type {
            e.actions = vec![EventAction {
                action_type: EventActionType::SetVariable,
                key: "chapter".into(),
                value: VariableValue::Int(3),
            }];
        }
        let b23_in = bridge23.inputs[0].id;
        let b23_out = bridge23.outputs[0].id;
        let b23_id = bridge23.id;
        let es_out = graph.nodes[&evt_saya_id].outputs[0].id;
        let a3_in = graph.nodes[&a3.entry].inputs[0].id;
        graph.add_node(bridge23);
        graph.add_connection(evt_saya_id, es_out, b23_id, b23_in);
        graph.add_connection(b23_id, b23_out, a3.entry, a3_in);

        // Quests
        graph.quests = build_quests();

        // World entities
        graph.world_entities = build_world_entities();

        // Timelines
        graph.timelines = build_timelines(a1.aria_greeting);

        // Barks
        graph.barks = build_barks(&chars);

        // Locale
        graph.locale = build_locale(&graph);

        // Groups
        graph.groups = build_groups(&a1, &a2, &a3);

        // Reviews & comments
        apply_reviews(&mut graph, &a1, &a2);
        apply_comments(&mut graph, &a1, &a2, &a3);

        // Tags
        apply_tags(&mut graph, &a1, &a2, &a3);

        // --- Version snapshots ---
        // v1: early draft with just Act 1
        let mut v1_graph = DialogueGraph::new();
        v1_graph.characters = chars.all_cloned();
        v1_graph.variables = build_variables();
        let _ = build_act1(&mut v1_graph, &chars);
        let snap1 = VersionSnapshot {
            id: 1,
            timestamp: "2026-01-15 10:00:00 UTC".into(),
            description: "Act 1 draft".into(),
            graph: v1_graph,
        };

        // v2: Acts 1-2 complete
        let mut v2_graph = DialogueGraph::new();
        v2_graph.characters = chars.all_cloned();
        v2_graph.variables = build_variables();
        let _ = build_act1(&mut v2_graph, &chars);
        let _ = build_act2(&mut v2_graph, &chars);
        let snap2 = VersionSnapshot {
            id: 2,
            timestamp: "2026-02-01 14:30:00 UTC".into(),
            description: "Acts 1-2 complete".into(),
            graph: v2_graph,
        };

        // Build project
        let project = Project {
            version: "1.0".to_string(),
            name: "The Last Constellation".to_string(),
            graph,
            versions: vec![snap1, snap2],
            dock_layout: None,
        };

        let json = project.save_to_string().unwrap();
        std::fs::write(
            "examples/the_last_constellation.talenode",
            &json,
        )
        .unwrap();

        let node_count = project.graph.nodes.len();
        let conn_count = project.graph.connections.len();
        let char_count = project.graph.characters.len();
        let quest_count = project.graph.quests.len();
        let entity_count = project.graph.world_entities.len();
        let timeline_count = project.graph.timelines.len();
        let group_count = project.graph.groups.len();
        let version_count = project.versions.len();

        println!(
            "Generated the_last_constellation.talenode\n\
             \x20 {} bytes | {} nodes | {} connections\n\
             \x20 {} characters | {} quests | {} world entities\n\
             \x20 {} timelines | {} groups | {} versions",
            json.len(),
            node_count,
            conn_count,
            char_count,
            quest_count,
            entity_count,
            timeline_count,
            group_count,
            version_count,
        );

        // Sanity checks
        assert!(node_count >= 70, "expected 70+ nodes, got {node_count}");
        assert!(conn_count >= 40, "expected 40+ connections, got {conn_count}");
        assert_eq!(char_count, 7);
        assert_eq!(quest_count, 3);
        assert_eq!(entity_count, 6);
        assert_eq!(timeline_count, 2);
        assert_eq!(group_count, 6);
        assert_eq!(version_count, 2);

        // Verify roundtrip
        let loaded = Project::load_from_string(&json).unwrap();
        assert_eq!(loaded.graph.nodes.len(), node_count);
        assert_eq!(loaded.name, "The Last Constellation");
    }
}
