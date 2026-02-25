/// Comprehensive example project generation for dragon_quest.talenode.

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use crate::model::character::Character;
    use crate::model::graph::DialogueGraph;
    use crate::model::group::NodeGroup;
    use crate::model::node::*;
    use crate::model::project::Project;
    use crate::model::review::{NodeComment, ReviewStatus};
    use crate::model::variable::{Variable, VariableType};
    use crate::model::version::VersionSnapshot;

    fn dlg(
        n: &mut Node,
        char: &Character,
        text: &str,
        emotion: &str,
    ) {
        if let NodeType::Dialogue(ref mut d) = n.node_type {
            d.speaker_id = Some(char.id);
            d.speaker_name = char.name.clone();
            d.text = text.to_string();
            d.emotion = emotion.to_string();
        }
    }

    #[test]
    #[ignore] // Run manually: cargo test generate_example -- --ignored
    fn generate_example_project() {
        let mut g = DialogueGraph::new();

        // --- Characters (4) ---
        fn chr(name: &str, color: [u8; 4], portrait: &str) -> Character {
            Character { id: uuid::Uuid::new_v4(), name: name.into(), color, portrait_path: portrait.into() }
        }
        let elder = chr("Elder Aldric", [74, 144, 217, 255], "portraits/elder.png");
        let merchant = chr("Merchant Bria", [217, 165, 74, 255], "portraits/merchant.png");
        let guard = chr("Captain Rook", [76, 175, 80, 255], "portraits/guard.png");
        let dragon = chr("Dragon Ignis", [229, 57, 53, 255], "portraits/dragon.png");
        for c in [&elder, &merchant, &guard, &dragon] { g.characters.push(c.clone()); }

        // --- Variables (7 — Bool, Int, Float, Text) ---
        g.variables.extend([
            Variable::new_bool("quest_accepted", false), Variable::new_int("gold", 50),
            Variable { id: uuid::Uuid::new_v4(), name: "reputation".into(),
                var_type: VariableType::Float, default_value: VariableValue::Float(0.5) },
            Variable { id: uuid::Uuid::new_v4(), name: "player_name".into(),
                var_type: VariableType::Text, default_value: VariableValue::Text("Adventurer".into()) },
            Variable::new_bool("has_sword", false), Variable::new_bool("has_potion", false),
            Variable::new_bool("dragon_wounded", false),
        ]);

        // === NODES ===

        // 1. Start
        let start = Node::new_start([100.0, 400.0]);

        // 2. Elder greets (neutral, audio, {variable} interpolation)
        let mut n_greet = Node::new_dialogue([400.0, 400.0]);
        dlg(&mut n_greet, &elder, "Welcome, {player_name}! I am Elder Aldric of Thornvale.", "neutral");
        if let NodeType::Dialogue(ref mut d) = n_greet.node_type {
            d.audio_clip = Some("audio/elder_01.wav".to_string());
        }

        // 3. Elder warns (scared, portrait_override)
        let mut n_warn = Node::new_dialogue([700.0, 400.0]);
        dlg(&mut n_warn, &elder,
            "A terrible dragon has been sighted near the northern caves. \
             Our village lives in fear. We need someone brave enough to face it.",
            "scared");
        if let NodeType::Dialogue(ref mut d) = n_warn.node_type {
            d.portrait_override = Some("portraits/elder_worried.png".to_string());
        }

        // 4. Choice: 4 options (one with ConditionExpr)
        let mut n_choice = Node::new_choice([1050.0, 400.0]);
        n_choice.add_choice(); // adds 3rd option
        n_choice.add_choice(); // adds 4th option
        if let NodeType::Choice(ref mut c) = n_choice.node_type {
            c.prompt = "How will you respond to the Elder's plea?".to_string();
            c.choices[0].text = "I'll slay the dragon for you!".to_string();
            c.choices[1].text = "I need to buy supplies first.".to_string();
            c.choices[2].text = "Tell me what you know about this dragon.".to_string();
            c.choices[2].condition = Some(ConditionExpr {
                variable_name: "reputation".to_string(),
                operator: CompareOp::Gte,
                value: VariableValue::Float(0.3),
            });
            c.choices[3].text = "I'm not interested. Goodbye.".to_string();
        }
        for (i, label) in ["I'll slay the dragon for you!", "I need to buy supplies first.",
            "Tell me what you know about this dragon.", "I'm not interested. Goodbye."].iter().enumerate() {
            n_choice.outputs[i].label = label.to_string();
        }

        // 5. Event: quest start (SetVariable + PlaySound)
        let mut n_evt_quest = Node::new_event([1400.0, 100.0]);
        if let NodeType::Event(ref mut e) = n_evt_quest.node_type {
            e.actions = vec![
                EventAction { action_type: EventActionType::SetVariable, key: "quest_accepted".into(), value: VariableValue::Bool(true) },
                EventAction { action_type: EventActionType::PlaySound, key: "quest_accept.wav".into(), value: VariableValue::Bool(true) },
            ];
        }

        // 6. Elder happy (happy, metadata)
        let mut n_happy = Node::new_dialogue([1700.0, 100.0]);
        dlg(&mut n_happy, &elder,
            "Bless you, brave soul! The village will remember your courage. \
             Head north through the Darkwood Forest.", "happy");
        if let NodeType::Dialogue(ref mut d) = n_happy.node_type {
            d.metadata.insert("quest_hook".to_string(), "true".to_string());
        }

        // 7. Condition: has_sword == true
        let mut n_cond_sword = Node::new_condition([2000.0, 100.0]);
        if let NodeType::Condition(ref mut c) = n_cond_sword.node_type {
            c.variable_name = "has_sword".to_string();
            c.operator = CompareOp::Eq;
            c.value = VariableValue::Bool(true);
        }

        // 8. SubGraph: Forest Path
        let mut n_sub = Node::new_subgraph([2300.0, 0.0]);
        if let NodeType::SubGraph(ref mut s) = n_sub.node_type {
            s.name = "Forest Path".to_string();
            // Build child graph: Start → Guard dialogue → End
            s.child_graph = DialogueGraph::new();
            let sg_start = Node::new_start([100.0, 200.0]);
            let mut sg_guard = Node::new_dialogue([400.0, 200.0]);
            dlg(&mut sg_guard, &guard,
                "Halt! The forest ahead is dangerous. Stay on the marked path \
                 and watch for the dragon's patrols.", "neutral");
            if let NodeType::Dialogue(ref mut d) = sg_guard.node_type {
                d.audio_clip = Some("audio/guard_01.wav".to_string());
            }
            let mut sg_end = Node::new_end([700.0, 200.0]);
            if let NodeType::End(ref mut e) = sg_end.node_type {
                e.tag = "forest_cleared".to_string();
            }
            let (so, gi, go, ei) = (
                sg_start.outputs[0].id, sg_guard.inputs[0].id,
                sg_guard.outputs[0].id, sg_end.inputs[0].id,
            );
            let (sid, gid, eid) = (sg_start.id, sg_guard.id, sg_end.id);
            s.child_graph.add_node(sg_start);
            s.child_graph.add_node(sg_guard);
            s.child_graph.add_node(sg_end);
            s.child_graph.add_connection(sid, so, gid, gi);
            s.child_graph.add_connection(gid, go, eid, ei);
        }

        // 9. Elder suggests merchant (neutral)
        let mut n_suggest = Node::new_dialogue([2300.0, 200.0]);
        dlg(&mut n_suggest, &elder,
            "You may want to visit Bria's shop first. A good sword could save \
             your life against the dragon.", "neutral");

        // 10. Merchant greets (happy)
        let mut n_mgreet = Node::new_dialogue([1400.0, 350.0]);
        dlg(&mut n_mgreet, &merchant,
            "Welcome to Bria's Armory! I have the finest blades in Thornvale. \
             What catches your eye?", "happy");

        // 11. Condition: gold >= 30
        let mut n_cond_gold = Node::new_condition([1700.0, 350.0]);
        if let NodeType::Condition(ref mut c) = n_cond_gold.node_type {
            c.variable_name = "gold".to_string();
            c.operator = CompareOp::Gte;
            c.value = VariableValue::Int(30);
        }

        // 12. Event: buy sword (SetVariable + AddItem + RemoveItem + PlaySound)
        let mut n_evt_buy = Node::new_event([2000.0, 300.0]);
        if let NodeType::Event(ref mut e) = n_evt_buy.node_type {
            e.actions = vec![
                EventAction { action_type: EventActionType::SetVariable, key: "has_sword".into(), value: VariableValue::Bool(true) },
                EventAction { action_type: EventActionType::AddItem, key: "iron_sword".into(), value: VariableValue::Int(1) },
                EventAction { action_type: EventActionType::RemoveItem, key: "old_dagger".into(), value: VariableValue::Int(1) },
                EventAction { action_type: EventActionType::PlaySound, key: "purchase.wav".into(), value: VariableValue::Bool(true) },
            ];
        }

        // 13. Merchant thanks (happy)
        let mut n_mthanks = Node::new_dialogue([2300.0, 300.0]);
        dlg(&mut n_mthanks, &merchant,
            "Excellent choice! This iron sword has served many an adventurer well. \
             Good luck out there!", "happy");

        // 14. Merchant sorry (sad)
        let mut n_msorry = Node::new_dialogue([2000.0, 450.0]);
        dlg(&mut n_msorry, &merchant,
            "I'm sorry, friend. You'll need at least 30 gold for a proper sword. \
             Come back when you've earned some more.", "sad");

        // 15. Event: knowledge gained (Custom action)
        let mut n_evt_know = Node::new_event([1400.0, 650.0]);
        if let NodeType::Event(ref mut e) = n_evt_know.node_type {
            e.actions = vec![EventAction {
                action_type: EventActionType::Custom("knowledge_gained".into()),
                key: "dragon_lore".into(), value: VariableValue::Bool(true),
            }];
        }

        // 16. Random: 3 rumor branches (40/35/25)
        let mut n_random = Node::new_random([1700.0, 650.0]);
        n_random.add_random_branch(); // adds 3rd branch
        if let NodeType::Random(ref mut r) = n_random.node_type {
            r.branches[0].weight = 0.4;
            r.branches[1].weight = 0.35;
            r.branches[2].weight = 0.25;
        }
        n_random.outputs[0].label = "40%".to_string();
        n_random.outputs[1].label = "35%".to_string();
        n_random.outputs[2].label = "25%".to_string();

        // 17-19. Rumor dialogues (scared, surprised, angry — showcasing emotions)
        let mut n_fire = Node::new_dialogue([2000.0, 550.0]);
        dlg(&mut n_fire, &elder,
            "The dragon breathes fire hot enough to melt stone. \
             Whatever you do, don't let it corner you.", "scared");

        let mut n_treasure = Node::new_dialogue([2000.0, 650.0]);
        dlg(&mut n_treasure, &elder,
            "They say the dragon sits atop a mountain of gold and jewels. \
             Perhaps that's worth the risk?", "surprised");

        let mut n_water = Node::new_dialogue([2000.0, 750.0]);
        dlg(&mut n_water, &elder,
            "I've heard that dragon once destroyed an entire garrison. \
             The soldiers never stood a chance!", "angry");

        // 20. Elder decline (sad)
        let mut n_decline = Node::new_dialogue([1400.0, 950.0]);
        dlg(&mut n_decline, &elder,
            "I understand. Not everyone is meant for such dangers. \
             Safe travels, stranger.", "sad");

        // 21-28. End nodes
        let ends: Vec<(&str, [f32; 2])> = vec![
            ("quest_begun", [2600.0, 0.0]),
            ("need_supplies", [2600.0, 200.0]),
            ("bought_supplies", [2600.0, 300.0]),
            ("no_gold", [2300.0, 450.0]),
            ("rumor_fire", [2300.0, 550.0]),
            ("rumor_treasure", [2300.0, 650.0]),
            ("rumor_water", [2300.0, 750.0]),
            ("declined", [1700.0, 950.0]),
        ];
        let mut end_nodes: Vec<Node> = ends.iter().map(|(tag, pos)| {
            let mut n = Node::new_end(*pos);
            if let NodeType::End(ref mut e) = n.node_type { e.tag = tag.to_string(); }
            n
        }).collect();

        // Collect all port IDs before moving nodes
        let p = |n: &Node, dir: &str, idx: usize| -> crate::model::port::PortId {
            match dir { "i" => n.inputs[idx].id, _ => n.outputs[idx].id }
        };

        // Build connection pairs: (from_node, from_port, to_node, to_port)
        type C<'a> = (&'a Node, usize, &'a Node, usize); // (from, out_idx, to, in_idx)
        let conns: Vec<C> = vec![
            (&start, 0, &n_greet, 0),           // start → greet
            (&n_greet, 0, &n_warn, 0),           // greet → warn
            (&n_warn, 0, &n_choice, 0),          // warn → choice
            (&n_choice, 0, &n_evt_quest, 0),     // choice[0] → evt_quest
            (&n_choice, 1, &n_mgreet, 0),        // choice[1] → merchant
            (&n_choice, 2, &n_evt_know, 0),      // choice[2] → evt_know
            (&n_choice, 3, &n_decline, 0),       // choice[3] → decline
            (&n_evt_quest, 0, &n_happy, 0),      // evt_quest → happy
            (&n_happy, 0, &n_cond_sword, 0),     // happy → cond_sword
            (&n_cond_sword, 0, &n_sub, 0),       // sword[T] → subgraph
            (&n_cond_sword, 1, &n_suggest, 0),   // sword[F] → suggest
            (&n_sub, 0, &end_nodes[0], 0),       // subgraph → end_quest
            (&n_suggest, 0, &end_nodes[1], 0),   // suggest → end_need
            (&n_mgreet, 0, &n_cond_gold, 0),     // merchant → cond_gold
            (&n_cond_gold, 0, &n_evt_buy, 0),    // gold[T] → evt_buy
            (&n_cond_gold, 1, &n_msorry, 0),     // gold[F] → sorry
            (&n_evt_buy, 0, &n_mthanks, 0),      // evt_buy → thanks
            (&n_mthanks, 0, &end_nodes[2], 0),   // thanks → end_bought
            (&n_msorry, 0, &end_nodes[3], 0),    // sorry → end_nogold
            (&n_evt_know, 0, &n_random, 0),      // evt_know → random
            (&n_random, 0, &n_fire, 0),          // random[0] → fire
            (&n_random, 1, &n_treasure, 0),      // random[1] → treasure
            (&n_random, 2, &n_water, 0),         // random[2] → water
            (&n_fire, 0, &end_nodes[4], 0),      // fire → end_fire
            (&n_treasure, 0, &end_nodes[5], 0),  // treasure → end_treasure
            (&n_water, 0, &end_nodes[6], 0),     // water → end_water
            (&n_decline, 0, &end_nodes[7], 0),   // decline → end_declined
        ];

        // Collect connection data before moving
        let conn_data: Vec<_> = conns.iter().map(|(from, oi, to, ii)| {
            (from.id, p(from, "o", *oi), to.id, p(to, "i", *ii))
        }).collect();

        // Store node IDs for groups/reviews/tags before moving
        let (id_start, id_greet, id_warn, id_choice, id_evt_quest) =
            (start.id, n_greet.id, n_warn.id, n_choice.id, n_evt_quest.id);
        let (id_happy, id_sub, id_mgreet, id_cond_gold, id_evt_buy) =
            (n_happy.id, n_sub.id, n_mgreet.id, n_cond_gold.id, n_evt_buy.id);
        let (id_mthanks, id_msorry, id_random) = (n_mthanks.id, n_msorry.id, n_random.id);
        let (id_fire, id_treasure, id_water, id_decline) =
            (n_fire.id, n_treasure.id, n_water.id, n_decline.id);
        let end_ids: Vec<_> = end_nodes.iter().map(|n| n.id).collect();

        // Add all nodes
        for n in [start, n_greet, n_warn, n_choice, n_evt_quest, n_happy,
            n_cond_sword, n_sub, n_suggest, n_mgreet, n_cond_gold, n_evt_buy,
            n_mthanks, n_msorry, n_evt_know, n_random, n_fire, n_treasure,
            n_water, n_decline] {
            g.add_node(n);
        }
        for n in end_nodes.drain(..) { g.add_node(n); }

        // Add all connections
        for (fn_id, fp, tn_id, tp) in conn_data {
            g.add_connection(fn_id, fp, tn_id, tp);
        }

        // --- Groups (2) ---
        let mut grp_intro = NodeGroup::new("Introduction");
        grp_intro.node_ids = vec![id_greet, id_warn, id_choice];
        grp_intro.color = [74, 144, 217, 30];
        let mut grp_shop = NodeGroup::new("Merchant Shop");
        grp_shop.node_ids = vec![id_mgreet, id_cond_gold, id_evt_buy, id_mthanks, id_msorry];
        grp_shop.color = [217, 165, 74, 30];
        g.groups.push(grp_intro);
        g.groups.push(grp_shop);

        // --- Review statuses ---
        g.review_statuses.insert(id_start, ReviewStatus::Approved);
        g.review_statuses.insert(id_greet, ReviewStatus::Approved);
        g.review_statuses.insert(id_warn, ReviewStatus::NeedsReview);
        g.review_statuses.insert(id_happy, ReviewStatus::Approved);
        g.review_statuses.insert(id_mthanks, ReviewStatus::Approved);
        g.review_statuses.insert(id_msorry, ReviewStatus::NeedsReview);
        g.review_statuses.insert(id_decline, ReviewStatus::Approved);

        // --- Comments (3) ---
        for (nid, text) in [(id_warn, "Should the Elder sound more desperate here?"),
            (id_choice, "Consider adding a stealth option for rogue characters."),
            (id_evt_buy, "Verify gold cost with game designer \u{2014} 30 might be too cheap.")] {
            g.comments.push(NodeComment::new(nid, text.to_string()));
        }

        // --- Tags ---
        let mut tags: HashMap<uuid::Uuid, Vec<String>> = HashMap::new();
        for &id in &[id_start, id_greet, id_warn, id_choice] {
            tags.insert(id, vec!["main_path".to_string(), "intro".to_string()]);
        }
        for &id in &[id_mgreet, id_cond_gold, id_evt_buy, id_mthanks, id_msorry] {
            tags.insert(id, vec!["shop".to_string()]);
        }
        for &id in &[id_random, id_fire, id_treasure, id_water] {
            tags.insert(id, vec!["lore".to_string()]);
        }
        tags.insert(id_sub, vec!["quest".to_string(), "forest".to_string()]);
        tags.insert(id_evt_quest, vec!["quest".to_string()]);
        for &id in &end_ids { tags.insert(id, vec!["ending".to_string()]); }
        g.node_tags = tags;

        // --- Version snapshot (earlier draft of intro only) ---
        let mut v_graph = DialogueGraph::new();
        let v_start = Node::new_start([100.0, 300.0]);
        let mut v_dlg = Node::new_dialogue([400.0, 300.0]);
        dlg(&mut v_dlg, &elder, "Welcome, traveler!", "neutral");
        let mut v_end = Node::new_end([700.0, 300.0]);
        if let NodeType::End(ref mut e) = v_end.node_type { e.tag = "placeholder".into(); }
        let (vso, vdi) = (v_start.outputs[0].id, v_dlg.inputs[0].id);
        let (vdo, vei) = (v_dlg.outputs[0].id, v_end.inputs[0].id);
        let (vs_id, vd_id, ve_id) = (v_start.id, v_dlg.id, v_end.id);
        v_graph.characters.push(elder.clone());
        for n in [v_start, v_dlg, v_end] { v_graph.add_node(n); }
        v_graph.add_connection(vs_id, vso, vd_id, vdi);
        v_graph.add_connection(vd_id, vdo, ve_id, vei);
        let version = VersionSnapshot { id: 1, timestamp: "2026-02-20 14:30:00 UTC".into(),
            description: "Initial draft \u{2014} intro only".into(), graph: v_graph };

        // === Build project ===
        let project = Project {
            version: "1.0".to_string(),
            name: "Dragon's Lair".to_string(),
            graph: g,
            versions: vec![version],
        };

        let json = project.save_to_string().unwrap();
        std::fs::write("examples/dragon_quest.talenode", &json).unwrap();
        println!("Generated dragon_quest.talenode ({} bytes, {} nodes, {} connections)",
            json.len(),
            project.graph.nodes.len(),
            project.graph.connections.len(),
        );
    }
}
