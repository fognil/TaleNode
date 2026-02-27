#[cfg(test)]
use uuid::Uuid;

#[cfg(test)]
use crate::model::character::Character;
#[cfg(test)]
use crate::model::graph::DialogueGraph;
#[cfg(test)]
use crate::model::node::*;

#[cfg(test)]
#[allow(dead_code)]
pub struct Act1Ids {
    pub all_node_ids: Vec<Uuid>,
    pub start: Uuid,
    pub aria_greeting: Uuid,
    pub hub_choice: Uuid,
    pub voss_meet: Uuid,
    pub zeph_meet: Uuid,
    pub farewell: Uuid,
    pub act1_end: Uuid,
}

#[cfg(test)]
fn dlg(n: &mut Node, ch: &Character, text: &str, emotion: &str) {
    if let NodeType::Dialogue(ref mut d) = n.node_type {
        d.speaker_id = Some(ch.id);
        d.speaker_name = ch.name.clone();
        d.text = text.to_string();
        d.emotion = emotion.to_string();
    }
}

#[cfg(test)]
pub fn build_act1(
    graph: &mut DialogueGraph,
    chars: &super::characters::CharacterIds,
) -> Act1Ids {
    // --- Create nodes ---
    let start = Node::new_start([100.0, 500.0]);

    let mut n_aria1 = Node::new_dialogue([400.0, 500.0]);
    dlg(&mut n_aria1, &chars.aria,
        "Welcome to Station Meridian, {player_name}. I am ARIA, \
         the station's administrative intelligence. Your diplomatic \
         credentials have been verified.", "neutral");
    if let NodeType::Dialogue(ref mut d) = n_aria1.node_type {
        d.audio_clip = Some("audio/aria_welcome.wav".to_string());
    }

    let mut n_evt_init = Node::new_event([700.0, 500.0]);
    if let NodeType::Event(ref mut e) = n_evt_init.node_type {
        e.actions = vec![
            EventAction {
                action_type: EventActionType::SetVariable,
                key: "chapter".into(),
                value: VariableValue::Int(1),
            },
            EventAction {
                action_type: EventActionType::PlaySound,
                key: "station_ambient.wav".into(),
                value: VariableValue::Bool(true),
            },
        ];
    }

    let mut n_aria2 = Node::new_dialogue([1000.0, 500.0]);
    dlg(&mut n_aria2, &chars.aria,
        "Commander Voss has requested your presence in the command \
         center. However, you are free to explore the station first. \
         Tensions between the Solari and Umbral delegations are... elevated.",
        "neutral");

    // Hub choice — 4 options
    let mut n_hub = Node::new_choice([1300.0, 500.0]);
    n_hub.add_choice();
    n_hub.add_choice();
    if let NodeType::Choice(ref mut c) = n_hub.node_type {
        c.prompt = "What would you like to do?".to_string();
        c.choices[0].text = "Report to Commander Voss".to_string();
        c.choices[1].text = "Explore the station".to_string();
        c.choices[2].text = "Visit the bar".to_string();
        c.choices[3].text = "Review mission briefing".to_string();
    }
    for (i, lbl) in ["Report to Commander Voss", "Explore the station",
        "Visit the bar", "Review mission briefing"].iter().enumerate()
    {
        n_hub.outputs[i].label = lbl.to_string();
    }

    // Path A: Voss
    let mut n_voss1 = Node::new_dialogue([1700.0, 200.0]);
    dlg(&mut n_voss1, &chars.voss,
        "Agent. I'm Commander Voss. This station is a powder keg \
         and you're the only diplomat the Council could spare. \
         No pressure.", "neutral");
    if let NodeType::Dialogue(ref mut d) = n_voss1.node_type {
        d.portrait_override = Some("portraits/voss_serious.png".to_string());
    }

    let mut n_evt_quest = Node::new_event([2000.0, 200.0]);
    if let NodeType::Event(ref mut e) = n_evt_quest.node_type {
        e.actions = vec![
            EventAction {
                action_type: EventActionType::StartQuest,
                key: "diplomatic_mission".into(),
                value: VariableValue::Bool(true),
            },
        ];
    }

    let mut n_voss2 = Node::new_dialogue([2300.0, 200.0]);
    dlg(&mut n_voss2, &chars.voss,
        "The Solari Dominion wants expansion rights. The Umbral \
         Collective demands isolation. Find common ground before \
         they tear this station apart.", "neutral");
    if let NodeType::Dialogue(ref mut d) = n_voss2.node_type {
        d.metadata.insert("quest_hook".into(), "true".into());
    }

    // Path B: Station Tour SubGraph
    let mut n_sub = Node::new_subgraph([1700.0, 500.0]);
    if let NodeType::SubGraph(ref mut s) = n_sub.node_type {
        s.name = "Station Tour".to_string();
        let mut sg = DialogueGraph::new();
        let ss = Node::new_start([100.0, 200.0]);
        let mut sg1 = Node::new_dialogue([400.0, 200.0]);
        dlg(&mut sg1, &chars.aria,
            "The docking bay processes all arrivals. Security \
             scanners ensure no contraband enters the station.",
            "neutral");
        let mut sg2 = Node::new_dialogue([700.0, 200.0]);
        dlg(&mut sg2, &chars.aria,
            "The commons area serves all three factions. Neutral \
             ground — though 'neutral' is a generous term lately.",
            "neutral");
        let mut sg3 = Node::new_dialogue([1000.0, 200.0]);
        dlg(&mut sg3, &chars.aria,
            "The observation deck offers a view of the Veil Nebula. \
             Many find it calming. I find it... data-rich.", "neutral");
        let se = Node::new_end([1300.0, 200.0]);
        let ports: Vec<_> = [&ss, &sg1, &sg2, &sg3, &se].iter().map(|n| {
            (n.id, n.inputs.first().map(|p| p.id), n.outputs.first().map(|p| p.id))
        }).collect();
        sg.add_node(ss); sg.add_node(sg1); sg.add_node(sg2);
        sg.add_node(sg3); sg.add_node(se);
        sg.add_connection(ports[0].0, ports[0].2.unwrap(), ports[1].0, ports[1].1.unwrap());
        sg.add_connection(ports[1].0, ports[1].2.unwrap(), ports[2].0, ports[2].1.unwrap());
        sg.add_connection(ports[2].0, ports[2].2.unwrap(), ports[3].0, ports[3].1.unwrap());
        sg.add_connection(ports[3].0, ports[3].2.unwrap(), ports[4].0, ports[4].1.unwrap());
        s.child_graph = sg;
    }

    // Path C: Bar / Zeph
    let mut n_zeph1 = Node::new_dialogue([1700.0, 700.0]);
    dlg(&mut n_zeph1, &chars.zeph,
        "Psst! Hey, you look official. Name's Zeph. I deal in \
         information — and occasionally other things. Interested?",
        "happy");

    let mut n_zeph_choice = Node::new_choice([2000.0, 700.0]);
    if let NodeType::Choice(ref mut c) = n_zeph_choice.node_type {
        c.prompt = "Zeph offers you a compact neural disruptor.".to_string();
        c.choices[0].text = "No thanks. I work by the book.".to_string();
        c.choices[1].text = "How much?".to_string();
    }
    n_zeph_choice.outputs[0].label = "Refuse".to_string();
    n_zeph_choice.outputs[1].label = "Buy weapon".to_string();

    let mut n_evt_refuse = Node::new_event([2300.0, 650.0]);
    if let NodeType::Event(ref mut e) = n_evt_refuse.node_type {
        e.actions = vec![EventAction {
            action_type: EventActionType::SetVariable,
            key: "encounters".into(),
            value: VariableValue::Int(1),
        }];
    }

    let mut n_evt_buy = Node::new_event([2300.0, 800.0]);
    if let NodeType::Event(ref mut e) = n_evt_buy.node_type {
        e.actions = vec![
            EventAction {
                action_type: EventActionType::SetVariable,
                key: "has_weapon".into(),
                value: VariableValue::Bool(true),
            },
            EventAction {
                action_type: EventActionType::RemoveItem,
                key: "credits".into(),
                value: VariableValue::Int(200),
            },
            EventAction {
                action_type: EventActionType::AddItem,
                key: "neural_disruptor".into(),
                value: VariableValue::Int(1),
            },
        ];
    }

    // Path D: Briefing
    let mut n_brief = Node::new_dialogue([1700.0, 950.0]);
    dlg(&mut n_brief, &chars.aria,
        "Briefing summary: Station Meridian hosts 3,000 residents. \
         The Solari delegation arrived 72 hours ago. The Umbral \
         contingent arrived 48 hours ago. Three altercations \
         have already been reported.", "neutral");

    // End nodes for side paths (single-input-port: each path needs its own End)
    let mut end_tour = Node::new_end([2000.0, 500.0]);
    if let NodeType::End(ref mut e) = end_tour.node_type {
        e.tag = "tour_complete".to_string();
    }

    let mut end_refuse = Node::new_end([2600.0, 650.0]);
    if let NodeType::End(ref mut e) = end_refuse.node_type {
        e.tag = "weapon_refused".to_string();
    }

    let mut end_buy = Node::new_end([2600.0, 800.0]);
    if let NodeType::End(ref mut e) = end_buy.node_type {
        e.tag = "weapon_bought".to_string();
    }

    let mut end_brief = Node::new_end([2000.0, 950.0]);
    if let NodeType::End(ref mut e) = end_brief.node_type {
        e.tag = "briefing_read".to_string();
    }

    // Main path convergence: farewell (only Voss path leads here → Act 2)
    let mut n_farewell = Node::new_dialogue([2600.0, 500.0]);
    dlg(&mut n_farewell, &chars.aria,
        "You are now oriented. The diplomatic sessions can begin \
         whenever you are ready. Good luck, {player_name}.",
        "neutral");

    let mut n_end = Node::new_end([2900.0, 500.0]);
    if let NodeType::End(ref mut e) = n_end.node_type {
        e.tag = "act1_complete".to_string();
    }

    // --- Capture IDs and ports ---
    type P = crate::model::port::PortId;
    let p = |n: &Node, d: &str, i: usize| -> P {
        match d { "i" => n.inputs[i].id, _ => n.outputs[i].id }
    };

    let conn_data: Vec<(Uuid, P, Uuid, P)> = vec![
        (start.id, p(&start,"o",0), n_aria1.id, p(&n_aria1,"i",0)),
        (n_aria1.id, p(&n_aria1,"o",0), n_evt_init.id, p(&n_evt_init,"i",0)),
        (n_evt_init.id, p(&n_evt_init,"o",0), n_aria2.id, p(&n_aria2,"i",0)),
        (n_aria2.id, p(&n_aria2,"o",0), n_hub.id, p(&n_hub,"i",0)),
        // Path A: Voss → main path (continues to Act 2)
        (n_hub.id, p(&n_hub,"o",0), n_voss1.id, p(&n_voss1,"i",0)),
        (n_voss1.id, p(&n_voss1,"o",0), n_evt_quest.id, p(&n_evt_quest,"i",0)),
        (n_evt_quest.id, p(&n_evt_quest,"o",0), n_voss2.id, p(&n_voss2,"i",0)),
        (n_voss2.id, p(&n_voss2,"o",0), n_farewell.id, p(&n_farewell,"i",0)),
        (n_farewell.id, p(&n_farewell,"o",0), n_end.id, p(&n_end,"i",0)),
        // Path B: Station Tour → end
        (n_hub.id, p(&n_hub,"o",1), n_sub.id, p(&n_sub,"i",0)),
        (n_sub.id, p(&n_sub,"o",0), end_tour.id, p(&end_tour,"i",0)),
        // Path C: Bar → Zeph choice → refuse/buy endings
        (n_hub.id, p(&n_hub,"o",2), n_zeph1.id, p(&n_zeph1,"i",0)),
        (n_zeph1.id, p(&n_zeph1,"o",0), n_zeph_choice.id, p(&n_zeph_choice,"i",0)),
        (n_zeph_choice.id, p(&n_zeph_choice,"o",0), n_evt_refuse.id, p(&n_evt_refuse,"i",0)),
        (n_zeph_choice.id, p(&n_zeph_choice,"o",1), n_evt_buy.id, p(&n_evt_buy,"i",0)),
        (n_evt_refuse.id, p(&n_evt_refuse,"o",0), end_refuse.id, p(&end_refuse,"i",0)),
        (n_evt_buy.id, p(&n_evt_buy,"o",0), end_buy.id, p(&end_buy,"i",0)),
        // Path D: Briefing → end
        (n_hub.id, p(&n_hub,"o",3), n_brief.id, p(&n_brief,"i",0)),
        (n_brief.id, p(&n_brief,"o",0), end_brief.id, p(&end_brief,"i",0)),
    ];

    let ids = Act1Ids {
        start: start.id, aria_greeting: n_aria1.id,
        hub_choice: n_hub.id, voss_meet: n_voss1.id,
        zeph_meet: n_zeph1.id, farewell: n_farewell.id,
        act1_end: n_end.id,
        all_node_ids: vec![
            start.id, n_aria1.id, n_evt_init.id, n_aria2.id, n_hub.id,
            n_voss1.id, n_evt_quest.id, n_voss2.id, n_sub.id, end_tour.id,
            n_zeph1.id, n_zeph_choice.id, n_evt_refuse.id, end_refuse.id,
            n_evt_buy.id, end_buy.id,
            n_brief.id, end_brief.id, n_farewell.id, n_end.id,
        ],
    };

    // Add nodes
    for n in [start, n_aria1, n_evt_init, n_aria2, n_hub, n_voss1,
        n_evt_quest, n_voss2, n_sub, end_tour, n_zeph1, n_zeph_choice,
        n_evt_refuse, end_refuse, n_evt_buy, end_buy,
        n_brief, end_brief, n_farewell, n_end]
    {
        graph.add_node(n);
    }

    for (fn_id, fp, tn_id, tp) in conn_data {
        graph.add_connection(fn_id, fp, tn_id, tp);
    }

    ids
}
