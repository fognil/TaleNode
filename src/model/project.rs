use serde::{Deserialize, Serialize};

use super::graph::DialogueGraph;

/// The full project file format (.talenode).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Project {
    pub version: String,
    pub name: String,
    pub graph: DialogueGraph,
}

impl Default for Project {
    fn default() -> Self {
        Self {
            version: "1.0".to_string(),
            name: "Untitled".to_string(),
            graph: DialogueGraph::new(),
        }
    }
}

impl Project {
    pub fn save_to_string(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }

    pub fn load_from_string(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::character::Character;
    use crate::model::node::*;
    use crate::model::variable::Variable;

    #[test]
    fn save_load_roundtrip() {
        let mut project = Project::default();
        project.name = "Test Project".to_string();
        project.graph.add_node(Node::new_start([100.0, 200.0]));
        project.graph.add_node(Node::new_dialogue([300.0, 200.0]));

        let json = project.save_to_string().unwrap();
        let loaded = Project::load_from_string(&json).unwrap();

        assert_eq!(loaded.name, "Test Project");
        assert_eq!(loaded.graph.nodes.len(), 2);
    }

    #[test]
    #[ignore] // Run manually: cargo test generate_example -- --ignored
    fn generate_example_project() {
        let mut graph = crate::model::graph::DialogueGraph::new();

        // --- Characters ---
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

        // --- Variables ---
        graph.variables.push(Variable::new_bool("quest_started", false));
        graph.variables.push(Variable::new_int("gold", 50));
        graph.variables.push(Variable::new_bool("has_sword", false));

        // === Nodes ===
        // Row 1: Start
        let start = Node::new_start([100.0, 300.0]);

        // Row 2: Elder greeting
        let mut dlg_greet = Node::new_dialogue([400.0, 300.0]);
        if let NodeType::Dialogue(ref mut d) = dlg_greet.node_type {
            d.speaker_name = "Village Elder".to_string();
            d.speaker_id = Some(elder.id);
            d.text = "Welcome, traveler! Our village is in danger. A dragon has been spotted near the northern caves.".to_string();
            d.emotion = "worried".to_string();
        }

        // Row 3: Player choice
        let mut choice = Node::new_choice([750.0, 300.0]);
        if let NodeType::Choice(ref mut c) = choice.node_type {
            c.prompt = "What will you do?".to_string();
            c.choices[0].text = "I'll help you!".to_string();
            c.choices[1].text = "I need supplies first.".to_string();
        }
        // Update output port labels to match
        choice.outputs[0].label = "I'll help you!".to_string();
        choice.outputs[1].label = "I need supplies first.".to_string();
        // Add third choice
        choice.add_choice();
        if let NodeType::Choice(ref mut c) = choice.node_type {
            c.choices[2].text = "Not interested.".to_string();
        }
        choice.outputs[2].label = "Not interested.".to_string();

        // Branch A: Accept quest
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
            d.text = "You are brave indeed! Head north through the forest. Be careful!".to_string();
            d.emotion = "happy".to_string();
        }

        let mut end_quest = Node::new_end([1850.0, 150.0]);
        if let NodeType::End(ref mut e) = end_quest.node_type {
            e.tag = "quest_accepted".to_string();
        }

        // Branch B: Need supplies → Merchant
        let mut dlg_merchant = Node::new_dialogue([1150.0, 400.0]);
        if let NodeType::Dialogue(ref mut d) = dlg_merchant.node_type {
            d.speaker_name = "Merchant".to_string();
            d.speaker_id = Some(merchant.id);
            d.text = "Welcome to my shop! I have swords and potions. What do you need?".to_string();
            d.emotion = "happy".to_string();
        }

        // Condition: check gold
        let mut cond_gold = Node::new_condition([1500.0, 400.0]);
        if let NodeType::Condition(ref mut c) = cond_gold.node_type {
            c.variable_name = "gold".to_string();
            c.operator = CompareOp::Gte;
            c.value = VariableValue::Int(30);
        }

        // True: can buy
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
            d.text = "Here's your sword! Good luck on your quest.".to_string();
            d.emotion = "neutral".to_string();
        }

        let mut end_shop = Node::new_end([2550.0, 300.0]);
        if let NodeType::End(ref mut e) = end_shop.node_type {
            e.tag = "bought_supplies".to_string();
        }

        // False: too poor
        let mut dlg_poor = Node::new_dialogue([1850.0, 550.0]);
        if let NodeType::Dialogue(ref mut d) = dlg_poor.node_type {
            d.speaker_name = "Merchant".to_string();
            d.speaker_id = Some(merchant.id);
            d.text = "Sorry, you don't have enough gold. Come back when you have 30 gold.".to_string();
            d.emotion = "sad".to_string();
        }

        let mut end_poor = Node::new_end([2200.0, 550.0]);
        if let NodeType::End(ref mut e) = end_poor.node_type {
            e.tag = "no_gold".to_string();
        }

        // Branch C: Not interested
        let mut dlg_bye = Node::new_dialogue([1150.0, 650.0]);
        if let NodeType::Dialogue(ref mut d) = dlg_bye.node_type {
            d.speaker_name = "Village Elder".to_string();
            d.speaker_id = Some(elder.id);
            d.text = "I understand. Safe travels, stranger.".to_string();
            d.emotion = "sad".to_string();
        }

        let mut end_bye = Node::new_end([1500.0, 650.0]);
        if let NodeType::End(ref mut e) = end_bye.node_type {
            e.tag = "declined".to_string();
        }

        // Collect port IDs before adding nodes
        let start_out = start.outputs[0].id;
        let dlg_greet_in = dlg_greet.inputs[0].id;
        let dlg_greet_out = dlg_greet.outputs[0].id;
        let choice_in = choice.inputs[0].id;
        let choice_out_0 = choice.outputs[0].id;
        let choice_out_1 = choice.outputs[1].id;
        let choice_out_2 = choice.outputs[2].id;

        let evt_quest_in = evt_quest.inputs[0].id;
        let evt_quest_out = evt_quest.outputs[0].id;
        let dlg_brave_in = dlg_brave.inputs[0].id;
        let dlg_brave_out = dlg_brave.outputs[0].id;
        let end_quest_in = end_quest.inputs[0].id;

        let dlg_merchant_in = dlg_merchant.inputs[0].id;
        let dlg_merchant_out = dlg_merchant.outputs[0].id;
        let cond_gold_in = cond_gold.inputs[0].id;
        let cond_gold_true = cond_gold.outputs[0].id;
        let cond_gold_false = cond_gold.outputs[1].id;

        let evt_buy_in = evt_buy.inputs[0].id;
        let evt_buy_out = evt_buy.outputs[0].id;
        let dlg_bought_in = dlg_bought.inputs[0].id;
        let dlg_bought_out = dlg_bought.outputs[0].id;
        let end_shop_in = end_shop.inputs[0].id;

        let dlg_poor_in = dlg_poor.inputs[0].id;
        let dlg_poor_out = dlg_poor.outputs[0].id;
        let end_poor_in = end_poor.inputs[0].id;

        let dlg_bye_in = dlg_bye.inputs[0].id;
        let dlg_bye_out = dlg_bye.outputs[0].id;
        let end_bye_in = end_bye.inputs[0].id;

        // Node IDs
        let start_id = start.id;
        let dlg_greet_id = dlg_greet.id;
        let choice_id = choice.id;
        let evt_quest_id = evt_quest.id;
        let dlg_brave_id = dlg_brave.id;
        let end_quest_id = end_quest.id;
        let dlg_merchant_id = dlg_merchant.id;
        let cond_gold_id = cond_gold.id;
        let evt_buy_id = evt_buy.id;
        let dlg_bought_id = dlg_bought.id;
        let end_shop_id = end_shop.id;
        let dlg_poor_id = dlg_poor.id;
        let end_poor_id = end_poor.id;
        let dlg_bye_id = dlg_bye.id;
        let end_bye_id = end_bye.id;

        // Add all nodes
        graph.add_node(start);
        graph.add_node(dlg_greet);
        graph.add_node(choice);
        graph.add_node(evt_quest);
        graph.add_node(dlg_brave);
        graph.add_node(end_quest);
        graph.add_node(dlg_merchant);
        graph.add_node(cond_gold);
        graph.add_node(evt_buy);
        graph.add_node(dlg_bought);
        graph.add_node(end_shop);
        graph.add_node(dlg_poor);
        graph.add_node(end_poor);
        graph.add_node(dlg_bye);
        graph.add_node(end_bye);

        // Add connections
        graph.add_connection(start_id, start_out, dlg_greet_id, dlg_greet_in);
        graph.add_connection(dlg_greet_id, dlg_greet_out, choice_id, choice_in);
        // Branch A
        graph.add_connection(choice_id, choice_out_0, evt_quest_id, evt_quest_in);
        graph.add_connection(evt_quest_id, evt_quest_out, dlg_brave_id, dlg_brave_in);
        graph.add_connection(dlg_brave_id, dlg_brave_out, end_quest_id, end_quest_in);
        // Branch B
        graph.add_connection(choice_id, choice_out_1, dlg_merchant_id, dlg_merchant_in);
        graph.add_connection(dlg_merchant_id, dlg_merchant_out, cond_gold_id, cond_gold_in);
        graph.add_connection(cond_gold_id, cond_gold_true, evt_buy_id, evt_buy_in);
        graph.add_connection(evt_buy_id, evt_buy_out, dlg_bought_id, dlg_bought_in);
        graph.add_connection(dlg_bought_id, dlg_bought_out, end_shop_id, end_shop_in);
        graph.add_connection(cond_gold_id, cond_gold_false, dlg_poor_id, dlg_poor_in);
        graph.add_connection(dlg_poor_id, dlg_poor_out, end_poor_id, end_poor_in);
        // Branch C
        graph.add_connection(choice_id, choice_out_2, dlg_bye_id, dlg_bye_in);
        graph.add_connection(dlg_bye_id, dlg_bye_out, end_bye_id, end_bye_in);

        let project = Project {
            version: "1.0".to_string(),
            name: "Dragon Quest Example".to_string(),
            graph,
        };

        let json = project.save_to_string().unwrap();
        std::fs::write("examples/dragon_quest.talenode", &json).unwrap();
        println!("Generated examples/dragon_quest.talenode ({} bytes)", json.len());
    }
}
