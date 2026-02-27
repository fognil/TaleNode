#[cfg(test)]
use uuid::Uuid;

#[cfg(test)]
use crate::model::character::Character;
#[cfg(test)]
use crate::model::graph::DialogueGraph;
#[cfg(test)]
use crate::model::node::*;

#[cfg(test)]
pub struct Act2Ids {
    pub all_node_ids: Vec<Uuid>,
    pub kael_nodes: Vec<Uuid>,
    pub thrix_nodes: Vec<Uuid>,
    pub elara_secret: Uuid,
    pub saya_node: Uuid,
    pub end_saya: Uuid,
    pub hub: Uuid,
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
pub fn build_act2(
    graph: &mut DialogueGraph,
    chars: &super::characters::CharacterIds,
) -> Act2Ids {
    // --- Hub ---
    let mut hub = Node::new_choice([3100.0, 500.0]);
    hub.add_choice(); hub.add_choice(); hub.add_choice();
    if let NodeType::Choice(ref mut c) = hub.node_type {
        c.prompt = "The station buzzes with activity. Where do you go?".into();
        c.choices[0].text = "Meet Ambassador Kael (Solari)".into();
        c.choices[1].text = "Speak with Warden Thrix (Umbral)".into();
        c.choices[2].text = "Visit Dr. Elara's lab".into();
        c.choices[3].text = "Help Zeph with a delivery".into();
        c.choices[3].condition = Some(ConditionExpr {
            variable_name: "encounters".into(),
            operator: CompareOp::Gte,
            value: VariableValue::Int(1),
        });
        c.choices[4].text = "Seek Elder Saya's counsel".into();
    }
    for (i, lbl) in ["Kael (Solari)", "Thrix (Umbral)", "Elara's Lab",
        "Help Zeph", "Elder Saya"].iter().enumerate()
    {
        hub.outputs[i].label = lbl.to_string();
    }

    // --- Path A: Kael ---
    let mut kael1 = Node::new_dialogue([3500.0, 100.0]);
    dlg(&mut kael1, &chars.kael,
        "Ah, the diplomat arrives. Ambassador Kael of the Solari \
         Dominion. I trust you appreciate the finer points of \
         civilized negotiation.", "neutral");

    let mut cond_sol = Node::new_condition([3800.0, 100.0]);
    if let NodeType::Condition(ref mut c) = cond_sol.node_type {
        c.variable_name = "faction_reputation_solari".into();
        c.operator = CompareOp::Gt;
        c.value = VariableValue::Int(0);
    }

    let mut kael_warm = Node::new_dialogue([4100.0, 0.0]);
    dlg(&mut kael_warm, &chars.kael,
        "I see you understand the Solari vision. Progress through \
         unity, order through strength. We could use an ally \
         like you at the negotiating table.", "happy");

    let mut kael_cold = Node::new_dialogue([4100.0, 200.0]);
    dlg(&mut kael_cold, &chars.kael,
        "You have yet to prove yourself to the Dominion. Words are \
         wind, Agent. Show me where your loyalties lie.", "neutral");

    let mut end_kael_cold = Node::new_end([4400.0, 200.0]);
    if let NodeType::End(ref mut e) = end_kael_cold.node_type {
        e.tag = "kael_cold_reception".into();
    }

    let mut evt_kael = Node::new_event([4400.0, 100.0]);
    if let NodeType::Event(ref mut e) = evt_kael.node_type {
        e.actions = vec![
            EventAction { action_type: EventActionType::SetVariable,
                key: "trust_kael".into(), value: VariableValue::Int(1) },
            EventAction { action_type: EventActionType::SetVariable,
                key: "encounters".into(), value: VariableValue::Int(2) },
        ];
    }

    let mut end_kael = Node::new_end([4700.0, 100.0]);
    if let NodeType::End(ref mut e) = end_kael.node_type {
        e.tag = "kael_met".into();
    }

    // --- Path B: Thrix + Random mood ---
    let mut thrix1 = Node::new_dialogue([3500.0, 400.0]);
    dlg(&mut thrix1, &chars.thrix,
        "You. Diplomat. I am Warden Thrix of the Umbral Collective. \
         Speak quickly — I have little patience for politics.", "neutral");

    let mut rng_mood = Node::new_random([3800.0, 400.0]);
    rng_mood.add_random_branch();
    if let NodeType::Random(ref mut r) = rng_mood.node_type {
        r.branches[0].weight = 0.4;
        r.branches[1].weight = 0.35;
        r.branches[2].weight = 0.25;
    }
    rng_mood.outputs[0].label = "Aggressive (40%)".into();
    rng_mood.outputs[1].label = "Neutral (35%)".into();
    rng_mood.outputs[2].label = "Respectful (25%)".into();

    let mut thrix_agg = Node::new_dialogue([4100.0, 300.0]);
    dlg(&mut thrix_agg, &chars.thrix,
        "The Umbral Collective does NOT need your interference! \
         We survived centuries in the void alone. We do not bend.",
        "angry");

    let mut thrix_neu = Node::new_dialogue([4100.0, 400.0]);
    dlg(&mut thrix_neu, &chars.thrix,
        "State your business. The Collective's position is non-negotiable, \
         but I will hear you out. Once.", "neutral");

    let mut thrix_resp = Node::new_dialogue([4100.0, 500.0]);
    dlg(&mut thrix_resp, &chars.thrix,
        "Perhaps you are different from the usual diplomats. There is \
         steel in your eyes. The Collective respects strength.", "neutral");

    let mut end_thrix_neu = Node::new_end([4400.0, 400.0]);
    if let NodeType::End(ref mut e) = end_thrix_neu.node_type {
        e.tag = "thrix_neutral".into();
    }

    let mut end_thrix_resp = Node::new_end([4400.0, 500.0]);
    if let NodeType::End(ref mut e) = end_thrix_resp.node_type {
        e.tag = "thrix_respectful".into();
    }

    let mut evt_thrix = Node::new_event([4400.0, 300.0]);
    if let NodeType::Event(ref mut e) = evt_thrix.node_type {
        e.actions = vec![EventAction {
            action_type: EventActionType::SetVariable,
            key: "encounters".into(), value: VariableValue::Int(2),
        }];
    }

    let mut end_thrix = Node::new_end([4700.0, 400.0]);
    if let NodeType::End(ref mut e) = end_thrix.node_type {
        e.tag = "thrix_met".into();
    }

    // --- Path C: Elara ---
    let mut elara1 = Node::new_dialogue([3500.0, 700.0]);
    dlg(&mut elara1, &chars.elara,
        "Oh! You must be the new envoy. I'm Dr. Elara Chen, \
         chief science officer. Forgive the mess — I've been \
         analyzing something extraordinary.", "happy");

    let mut elara2 = Node::new_dialogue([3800.0, 700.0]);
    dlg(&mut elara2, &chars.elara,
        "I've been studying what we call the Ancient Signal — a \
         repeating pattern from deep space. Its structure is unlike \
         anything natural. It's... deliberate.", "surprised");

    let mut cond_trust = Node::new_condition([4100.0, 700.0]);
    if let NodeType::Condition(ref mut c) = cond_trust.node_type {
        c.variable_name = "trust_elara".into();
        c.operator = CompareOp::Gte;
        c.value = VariableValue::Int(2);
    }

    let mut elara_secret = Node::new_dialogue([4400.0, 650.0]);
    dlg(&mut elara_secret, &chars.elara,
        "I haven't told anyone this, but... the signal is artificial. \
         And it's coming from inside the station. Someone on Meridian \
         is broadcasting. The peace talks may be a cover.", "scared");
    if let NodeType::Dialogue(ref mut d) = elara_secret.node_type {
        d.audio_clip = Some("audio/elara_whisper.wav".into());
    }

    let mut evt_conspiracy = Node::new_event([4700.0, 650.0]);
    if let NodeType::Event(ref mut e) = evt_conspiracy.node_type {
        e.actions = vec![
            EventAction { action_type: EventActionType::SetVariable,
                key: "discovered_conspiracy".into(), value: VariableValue::Bool(true) },
            EventAction { action_type: EventActionType::SetVariable,
                key: "peace_progress".into(), value: VariableValue::Int(1) },
            EventAction { action_type: EventActionType::SetVariable,
                key: "has_evidence".into(), value: VariableValue::Bool(true) },
        ];
    }

    let mut end_elara_yes = Node::new_end([5000.0, 650.0]);
    if let NodeType::End(ref mut e) = end_elara_yes.node_type {
        e.tag = "conspiracy_discovered".into();
    }

    let mut elara_reject = Node::new_dialogue([4400.0, 800.0]);
    dlg(&mut elara_reject, &chars.elara,
        "Come back when we know each other better. Some secrets \
         need trust to surface.", "sad");

    let mut end_elara_no = Node::new_end([4700.0, 800.0]);
    if let NodeType::End(ref mut e) = end_elara_no.node_type {
        e.tag = "elara_not_ready".into();
    }

    // --- Path D: Zeph SubGraph ---
    let mut n_zeph_sub = Node::new_subgraph([3500.0, 1000.0]);
    if let NodeType::SubGraph(ref mut s) = n_zeph_sub.node_type {
        s.name = "Zeph's Favor".into();
        let mut sg = DialogueGraph::new();
        let ss = Node::new_start([100.0, 200.0]);
        let mut z1 = Node::new_dialogue([400.0, 200.0]);
        dlg(&mut z1, &chars.zeph,
            "Simple job: deliver this package to bay 7. Don't open \
             it. Don't ask what's inside. Don't... okay, it's \
             definitely not explosives.", "happy");
        let mut z2 = Node::new_dialogue([700.0, 200.0]);
        dlg(&mut z2, &chars.zeph,
            "You're back! And in one piece! That's more than the \
             last courier. Here — your cut.", "happy");
        let se = Node::new_end([1000.0, 200.0]);
        let sp: Vec<_> = [&ss, &z1, &z2, &se].iter().map(|n| {
            (n.id, n.inputs.first().map(|p| p.id), n.outputs.first().map(|p| p.id))
        }).collect();
        sg.add_node(ss); sg.add_node(z1); sg.add_node(z2); sg.add_node(se);
        sg.add_connection(sp[0].0, sp[0].2.unwrap(), sp[1].0, sp[1].1.unwrap());
        sg.add_connection(sp[1].0, sp[1].2.unwrap(), sp[2].0, sp[2].1.unwrap());
        sg.add_connection(sp[2].0, sp[2].2.unwrap(), sp[3].0, sp[3].1.unwrap());
        s.child_graph = sg;
    }

    let mut evt_zeph_quest = Node::new_event([3800.0, 1000.0]);
    if let NodeType::Event(ref mut e) = evt_zeph_quest.node_type {
        e.actions = vec![EventAction {
            action_type: EventActionType::StartQuest,
            key: "smugglers_deal".into(), value: VariableValue::Bool(true),
        }];
    }

    let mut end_zeph = Node::new_end([4100.0, 1000.0]);
    if let NodeType::End(ref mut e) = end_zeph.node_type {
        e.tag = "zeph_quest_started".into();
    }

    // --- Path E: Saya ---
    let mut saya1 = Node::new_dialogue([3500.0, 1250.0]);
    dlg(&mut saya1, &chars.saya,
        "The stars speak of convergence, young diplomat. Two rivers \
         that have flowed apart for centuries now rush toward the \
         same sea. Whether they merge or collide depends on you.",
        "neutral");

    let mut evt_saya = Node::new_event([3800.0, 1250.0]);
    if let NodeType::Event(ref mut e) = evt_saya.node_type {
        e.actions = vec![EventAction {
            action_type: EventActionType::SetVariable,
            key: "peace_progress".into(), value: VariableValue::Int(1),
        }];
    }

    let mut end_saya = Node::new_end([4100.0, 1250.0]);
    if let NodeType::End(ref mut e) = end_saya.node_type {
        e.tag = "saya_counsel".into();
    }

    // --- Capture ports and IDs ---
    type P = crate::model::port::PortId;
    let p = |n: &Node, d: &str, i: usize| -> P {
        match d { "i" => n.inputs[i].id, _ => n.outputs[i].id }
    };

    let conn_data: Vec<(Uuid, P, Uuid, P)> = vec![
        // Hub branches
        (hub.id, p(&hub,"o",0), kael1.id, p(&kael1,"i",0)),
        (hub.id, p(&hub,"o",1), thrix1.id, p(&thrix1,"i",0)),
        (hub.id, p(&hub,"o",2), elara1.id, p(&elara1,"i",0)),
        (hub.id, p(&hub,"o",3), n_zeph_sub.id, p(&n_zeph_sub,"i",0)),
        (hub.id, p(&hub,"o",4), saya1.id, p(&saya1,"i",0)),
        // Kael chain
        (kael1.id, p(&kael1,"o",0), cond_sol.id, p(&cond_sol,"i",0)),
        (cond_sol.id, p(&cond_sol,"o",0), kael_warm.id, p(&kael_warm,"i",0)),
        (cond_sol.id, p(&cond_sol,"o",1), kael_cold.id, p(&kael_cold,"i",0)),
        (kael_warm.id, p(&kael_warm,"o",0), evt_kael.id, p(&evt_kael,"i",0)),
        (evt_kael.id, p(&evt_kael,"o",0), end_kael.id, p(&end_kael,"i",0)),
        (kael_cold.id, p(&kael_cold,"o",0), end_kael_cold.id, p(&end_kael_cold,"i",0)),
        // Thrix chain
        (thrix1.id, p(&thrix1,"o",0), rng_mood.id, p(&rng_mood,"i",0)),
        (rng_mood.id, p(&rng_mood,"o",0), thrix_agg.id, p(&thrix_agg,"i",0)),
        (rng_mood.id, p(&rng_mood,"o",1), thrix_neu.id, p(&thrix_neu,"i",0)),
        (rng_mood.id, p(&rng_mood,"o",2), thrix_resp.id, p(&thrix_resp,"i",0)),
        (thrix_agg.id, p(&thrix_agg,"o",0), evt_thrix.id, p(&evt_thrix,"i",0)),
        (evt_thrix.id, p(&evt_thrix,"o",0), end_thrix.id, p(&end_thrix,"i",0)),
        (thrix_neu.id, p(&thrix_neu,"o",0), end_thrix_neu.id, p(&end_thrix_neu,"i",0)),
        (thrix_resp.id, p(&thrix_resp,"o",0), end_thrix_resp.id, p(&end_thrix_resp,"i",0)),
        // Elara chain
        (elara1.id, p(&elara1,"o",0), elara2.id, p(&elara2,"i",0)),
        (elara2.id, p(&elara2,"o",0), cond_trust.id, p(&cond_trust,"i",0)),
        (cond_trust.id, p(&cond_trust,"o",0), elara_secret.id, p(&elara_secret,"i",0)),
        (cond_trust.id, p(&cond_trust,"o",1), elara_reject.id, p(&elara_reject,"i",0)),
        (elara_secret.id, p(&elara_secret,"o",0), evt_conspiracy.id, p(&evt_conspiracy,"i",0)),
        (evt_conspiracy.id, p(&evt_conspiracy,"o",0), end_elara_yes.id, p(&end_elara_yes,"i",0)),
        (elara_reject.id, p(&elara_reject,"o",0), end_elara_no.id, p(&end_elara_no,"i",0)),
        // Zeph chain
        (n_zeph_sub.id, p(&n_zeph_sub,"o",0), evt_zeph_quest.id, p(&evt_zeph_quest,"i",0)),
        (evt_zeph_quest.id, p(&evt_zeph_quest,"o",0), end_zeph.id, p(&end_zeph,"i",0)),
        // Saya chain
        (saya1.id, p(&saya1,"o",0), evt_saya.id, p(&evt_saya,"i",0)),
        (evt_saya.id, p(&evt_saya,"o",0), end_saya.id, p(&end_saya,"i",0)),
    ];

    let kael_ids = vec![kael1.id, cond_sol.id, kael_warm.id, kael_cold.id,
        end_kael_cold.id, evt_kael.id, end_kael.id];
    let thrix_ids = vec![thrix1.id, rng_mood.id, thrix_agg.id, thrix_neu.id, thrix_resp.id,
        evt_thrix.id, end_thrix.id, end_thrix_neu.id, end_thrix_resp.id];

    let ids = Act2Ids {
        hub: hub.id,
        kael_nodes: kael_ids.clone(),
        thrix_nodes: thrix_ids.clone(),
        elara_secret: elara_secret.id,
        saya_node: saya1.id,
        end_saya: end_saya.id,
        all_node_ids: vec![
            hub.id,
            kael1.id, cond_sol.id, kael_warm.id, kael_cold.id,
            end_kael_cold.id, evt_kael.id, end_kael.id,
            thrix1.id, rng_mood.id, thrix_agg.id, thrix_neu.id, thrix_resp.id,
            evt_thrix.id, end_thrix.id, end_thrix_neu.id, end_thrix_resp.id,
            elara1.id, elara2.id, cond_trust.id, elara_secret.id, evt_conspiracy.id, end_elara_yes.id,
            elara_reject.id, end_elara_no.id,
            n_zeph_sub.id, evt_zeph_quest.id, end_zeph.id,
            saya1.id, evt_saya.id, end_saya.id,
        ],
    };

    // Add all nodes
    for n in [hub, kael1, cond_sol, kael_warm, kael_cold, end_kael_cold,
        evt_kael, end_kael,
        thrix1, rng_mood, thrix_agg, thrix_neu, thrix_resp,
        evt_thrix, end_thrix, end_thrix_neu, end_thrix_resp,
        elara1, elara2, cond_trust, elara_secret, evt_conspiracy, end_elara_yes,
        elara_reject, end_elara_no,
        n_zeph_sub, evt_zeph_quest, end_zeph,
        saya1, evt_saya, end_saya]
    {
        graph.add_node(n);
    }

    for (fn_id, fp, tn_id, tp) in conn_data {
        graph.add_connection(fn_id, fp, tn_id, tp);
    }

    ids
}
