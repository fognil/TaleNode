/// Example project generation — separated to keep project.rs under 400 lines.

#[cfg(test)]
mod tests {
    use crate::model::character::Character;
    use crate::model::node::*;
    use crate::model::project::Project;
    use crate::model::variable::Variable;

    #[test]
    #[ignore] // Run manually: cargo test generate_example -- --ignored
    fn generate_example_project() {
        let mut graph = crate::model::graph::DialogueGraph::new();

        let elder = Character {
            id: uuid::Uuid::new_v4(),
            name: "Village Elder".to_string(),
            color: [74, 144, 217, 255],
            portrait_path: String::new(),
        };
        let merchant = Character {
            id: uuid::Uuid::new_v4(),
            name: "Merchant".to_string(),
            color: [217, 165, 74, 255],
            portrait_path: String::new(),
        };
        graph.characters.push(elder.clone());
        graph.characters.push(merchant.clone());

        graph.variables.push(Variable::new_bool("quest_started", false));
        graph.variables.push(Variable::new_int("gold", 50));
        graph.variables.push(Variable::new_bool("has_sword", false));

        let start = Node::new_start([100.0, 300.0]);
        let mut dlg_greet = Node::new_dialogue([400.0, 300.0]);
        if let NodeType::Dialogue(ref mut d) = dlg_greet.node_type {
            d.speaker_name = "Village Elder".to_string();
            d.speaker_id = Some(elder.id);
            d.text = "Welcome, traveler! Our village is in danger.".to_string();
            d.emotion = "worried".to_string();
        }
        let mut choice = Node::new_choice([750.0, 300.0]);
        if let NodeType::Choice(ref mut c) = choice.node_type {
            c.prompt = "What will you do?".to_string();
            c.choices[0].text = "I'll help you!".to_string();
            c.choices[1].text = "I need supplies first.".to_string();
        }
        choice.outputs[0].label = "I'll help you!".to_string();
        choice.outputs[1].label = "I need supplies first.".to_string();
        choice.add_choice();
        if let NodeType::Choice(ref mut c) = choice.node_type {
            c.choices[2].text = "Not interested.".to_string();
        }
        choice.outputs[2].label = "Not interested.".to_string();

        let mut evt_quest = Node::new_event([1150.0, 150.0]);
        if let NodeType::Event(ref mut e) = evt_quest.node_type {
            e.actions.push(EventAction {
                action_type: EventActionType::SetVariable,
                key: "quest_started".to_string(),
                value: VariableValue::Bool(true),
            });
        }
        let mut dlg_brave = Node::new_dialogue([1500.0, 150.0]);
        if let NodeType::Dialogue(ref mut d) = dlg_brave.node_type {
            d.speaker_name = "Village Elder".to_string();
            d.speaker_id = Some(elder.id);
            d.text = "You are brave indeed! Be careful!".to_string();
            d.emotion = "happy".to_string();
        }
        let mut end_quest = Node::new_end([1850.0, 150.0]);
        if let NodeType::End(ref mut e) = end_quest.node_type {
            e.tag = "quest_accepted".to_string();
        }

        let mut dlg_merchant = Node::new_dialogue([1150.0, 400.0]);
        if let NodeType::Dialogue(ref mut d) = dlg_merchant.node_type {
            d.speaker_name = "Merchant".to_string();
            d.speaker_id = Some(merchant.id);
            d.text = "Welcome to my shop!".to_string();
            d.emotion = "happy".to_string();
        }
        let mut cond_gold = Node::new_condition([1500.0, 400.0]);
        if let NodeType::Condition(ref mut c) = cond_gold.node_type {
            c.variable_name = "gold".to_string();
            c.operator = CompareOp::Gte;
            c.value = VariableValue::Int(30);
        }
        let mut evt_buy = Node::new_event([1850.0, 300.0]);
        if let NodeType::Event(ref mut e) = evt_buy.node_type {
            e.actions.push(EventAction {
                action_type: EventActionType::SetVariable,
                key: "has_sword".to_string(),
                value: VariableValue::Bool(true),
            });
        }
        let mut dlg_bought = Node::new_dialogue([2200.0, 300.0]);
        if let NodeType::Dialogue(ref mut d) = dlg_bought.node_type {
            d.speaker_name = "Merchant".to_string();
            d.speaker_id = Some(merchant.id);
            d.text = "Here's your sword!".to_string();
            d.emotion = "neutral".to_string();
        }
        let mut end_shop = Node::new_end([2550.0, 300.0]);
        if let NodeType::End(ref mut e) = end_shop.node_type { e.tag = "bought_supplies".to_string(); }
        let mut dlg_poor = Node::new_dialogue([1850.0, 550.0]);
        if let NodeType::Dialogue(ref mut d) = dlg_poor.node_type {
            d.speaker_name = "Merchant".to_string();
            d.speaker_id = Some(merchant.id);
            d.text = "Sorry, not enough gold.".to_string();
            d.emotion = "sad".to_string();
        }
        let mut end_poor = Node::new_end([2200.0, 550.0]);
        if let NodeType::End(ref mut e) = end_poor.node_type { e.tag = "no_gold".to_string(); }
        let mut dlg_bye = Node::new_dialogue([1150.0, 650.0]);
        if let NodeType::Dialogue(ref mut d) = dlg_bye.node_type {
            d.speaker_name = "Village Elder".to_string();
            d.speaker_id = Some(elder.id);
            d.text = "Safe travels, stranger.".to_string();
            d.emotion = "sad".to_string();
        }
        let mut end_bye = Node::new_end([1500.0, 650.0]);
        if let NodeType::End(ref mut e) = end_bye.node_type { e.tag = "declined".to_string(); }

        // Port IDs
        let (s_o, dg_i, dg_o) = (start.outputs[0].id, dlg_greet.inputs[0].id, dlg_greet.outputs[0].id);
        let (ch_i, ch0, ch1, ch2) = (choice.inputs[0].id, choice.outputs[0].id, choice.outputs[1].id, choice.outputs[2].id);
        let (eq_i, eq_o) = (evt_quest.inputs[0].id, evt_quest.outputs[0].id);
        let (db_i, db_o, ekq_i) = (dlg_brave.inputs[0].id, dlg_brave.outputs[0].id, end_quest.inputs[0].id);
        let (dm_i, dm_o) = (dlg_merchant.inputs[0].id, dlg_merchant.outputs[0].id);
        let (cg_i, cg_t, cg_f) = (cond_gold.inputs[0].id, cond_gold.outputs[0].id, cond_gold.outputs[1].id);
        let (eb_i, eb_o) = (evt_buy.inputs[0].id, evt_buy.outputs[0].id);
        let (dbo_i, dbo_o, es_i) = (dlg_bought.inputs[0].id, dlg_bought.outputs[0].id, end_shop.inputs[0].id);
        let (dp_i, dp_o, ep_i) = (dlg_poor.inputs[0].id, dlg_poor.outputs[0].id, end_poor.inputs[0].id);
        let (dby_i, dby_o, eby_i) = (dlg_bye.inputs[0].id, dlg_bye.outputs[0].id, end_bye.inputs[0].id);

        // Node IDs
        let ids = [start.id, dlg_greet.id, choice.id, evt_quest.id, dlg_brave.id, end_quest.id,
            dlg_merchant.id, cond_gold.id, evt_buy.id, dlg_bought.id, end_shop.id,
            dlg_poor.id, end_poor.id, dlg_bye.id, end_bye.id];

        for n in [start, dlg_greet, choice, evt_quest, dlg_brave, end_quest, dlg_merchant,
            cond_gold, evt_buy, dlg_bought, end_shop, dlg_poor, end_poor, dlg_bye, end_bye] {
            graph.add_node(n);
        }

        graph.add_connection(ids[0], s_o, ids[1], dg_i);
        graph.add_connection(ids[1], dg_o, ids[2], ch_i);
        graph.add_connection(ids[2], ch0, ids[3], eq_i);
        graph.add_connection(ids[3], eq_o, ids[4], db_i);
        graph.add_connection(ids[4], db_o, ids[5], ekq_i);
        graph.add_connection(ids[2], ch1, ids[6], dm_i);
        graph.add_connection(ids[6], dm_o, ids[7], cg_i);
        graph.add_connection(ids[7], cg_t, ids[8], eb_i);
        graph.add_connection(ids[8], eb_o, ids[9], dbo_i);
        graph.add_connection(ids[9], dbo_o, ids[10], es_i);
        graph.add_connection(ids[7], cg_f, ids[11], dp_i);
        graph.add_connection(ids[11], dp_o, ids[12], ep_i);
        graph.add_connection(ids[2], ch2, ids[13], dby_i);
        graph.add_connection(ids[13], dby_o, ids[14], eby_i);

        let project = Project {
            version: "1.0".to_string(),
            name: "Dragon Quest Example".to_string(),
            graph,
            versions: Vec::new(),
        };

        let json = project.save_to_string().unwrap();
        std::fs::write("examples/dragon_quest.talenode", &json).unwrap();
        println!("Generated ({} bytes)", json.len());
    }
}
