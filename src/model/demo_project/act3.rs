#[cfg(test)]
use uuid::Uuid;

#[cfg(test)]
use crate::model::character::Character;
#[cfg(test)]
use crate::model::graph::DialogueGraph;
#[cfg(test)]
use crate::model::node::*;

#[cfg(test)]
pub struct Act3Ids {
    pub all_node_ids: Vec<Uuid>,
    pub entry: Uuid,
    pub peace_end: Uuid,
    pub war_end: Uuid,
    pub betrayal_end: Uuid,
    pub escape_end: Uuid,
    pub sacrifice_end: Uuid,
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
pub fn build_act3(
    graph: &mut DialogueGraph,
    chars: &super::characters::CharacterIds,
) -> Act3Ids {
    // --- Entry condition: peace_progress ---
    let mut cond_ready = Node::new_condition([5400.0, 500.0]);
    if let NodeType::Condition(ref mut c) = cond_ready.node_type {
        c.variable_name = "peace_progress".into();
        c.operator = CompareOp::Gte;
        c.value = VariableValue::Int(2);
    }

    // --- Strong position path ---
    let mut voss_strong = Node::new_dialogue([5700.0, 300.0]);
    dlg(&mut voss_strong, &chars.voss,
        "You've done well, Agent. We have enough leverage to \
         broker a real agreement. How do you want to play this?",
        "happy");

    let mut choice_strong = Node::new_choice([6000.0, 300.0]);
    choice_strong.add_choice();
    if let NodeType::Choice(ref mut c) = choice_strong.node_type {
        c.prompt = "How will you resolve the conflict?".into();
        c.choices[0].text = "Propose a peace accord".into();
        c.choices[1].text = "Expose the conspiracy".into();
        c.choices[2].text = "Force both sides to comply".into();
    }
    choice_strong.outputs[0].label = "Diplomacy".into();
    choice_strong.outputs[1].label = "Truth".into();
    choice_strong.outputs[2].label = "Authority".into();

    // Diplomacy → Random outcome
    let mut rng_peace = Node::new_random([6300.0, 150.0]);
    if let NodeType::Random(ref mut r) = rng_peace.node_type {
        r.branches[0].weight = 0.6;
        r.branches[1].weight = 0.4;
    }
    rng_peace.outputs[0].label = "Success (60%)".into();
    rng_peace.outputs[1].label = "Complication (40%)".into();

    let mut dlg_peace_ok = Node::new_dialogue([6600.0, 100.0]);
    dlg(&mut dlg_peace_ok, &chars.voss,
        "Against all odds... they signed. Both factions agreed to \
         the Meridian Accord. The Veil Nebula will know peace.",
        "happy");

    let mut evt_peace = Node::new_event([6900.0, 100.0]);
    if let NodeType::Event(ref mut e) = evt_peace.node_type {
        e.actions = vec![
            EventAction { action_type: EventActionType::CompleteObjective,
                key: "propose_resolution".into(), value: VariableValue::Bool(true) },
            EventAction { action_type: EventActionType::PlaySound,
                key: "fanfare.wav".into(), value: VariableValue::Bool(true) },
        ];
    }

    let mut end_peace = Node::new_end([7200.0, 100.0]);
    if let NodeType::End(ref mut e) = end_peace.node_type {
        e.tag = "peace".into();
    }

    let mut dlg_war = Node::new_dialogue([6600.0, 200.0]);
    dlg(&mut dlg_war, &chars.thrix,
        "Negotiations have collapsed. The Umbral fleet is mobilizing. \
         The Solari have sealed their sector. This station is now \
         a battlefield.", "angry");

    let mut end_war = Node::new_end([6900.0, 200.0]);
    if let NodeType::End(ref mut e) = end_war.node_type {
        e.tag = "war".into();
    }

    // Truth path → Condition on conspiracy knowledge
    let mut cond_conspiracy = Node::new_condition([6300.0, 300.0]);
    if let NodeType::Condition(ref mut c) = cond_conspiracy.node_type {
        c.variable_name = "discovered_conspiracy".into();
        c.operator = CompareOp::Eq;
        c.value = VariableValue::Bool(true);
    }

    let mut elara_reveal = Node::new_dialogue([6600.0, 300.0]);
    dlg(&mut elara_reveal, &chars.elara,
        "The truth must come out. The Ancient Signal was manufactured \
         by a faction within the Solari — they wanted war to justify \
         military expansion. Here is the proof.", "scared");

    let mut end_betrayal = Node::new_end([6900.0, 300.0]);
    if let NodeType::End(ref mut e) = end_betrayal.node_type {
        e.tag = "betrayal".into();
    }

    let mut voss_no_proof = Node::new_dialogue([6600.0, 400.0]);
    dlg(&mut voss_no_proof, &chars.voss,
        "You want to expose a conspiracy without evidence? That's \
         not courage, Agent — that's recklessness. We proceed \
         with conventional diplomacy.", "angry");

    let mut end_no_proof = Node::new_end([6900.0, 400.0]);
    if let NodeType::End(ref mut e) = end_no_proof.node_type {
        e.tag = "war".into();
    }

    // Authority path → war
    let mut end_forced = Node::new_end([6600.0, 450.0]);
    if let NodeType::End(ref mut e) = end_forced.node_type {
        e.tag = "war".into();
    }

    let mut voss_force = Node::new_dialogue([6300.0, 450.0]);
    dlg(&mut voss_force, &chars.voss,
        "I'll deploy the station's security forces. Neither side \
         will have a choice. This is martial law, Agent — I hope \
         you know what you've started.", "neutral");

    // --- Weak position path ---
    let mut voss_weak = Node::new_dialogue([5700.0, 700.0]);
    dlg(&mut voss_weak, &chars.voss,
        "We don't have enough to work with. The factions are \
         entrenched and we're running out of time.", "sad");

    let mut choice_weak = Node::new_choice([6000.0, 700.0]);
    if let NodeType::Choice(ref mut c) = choice_weak.node_type {
        c.prompt = "The situation is dire. What do you do?".into();
        c.choices[0].text = "Flee on Zeph's ship".into();
        c.choices[1].text = "Sacrifice everything for peace".into();
    }
    choice_weak.outputs[0].label = "Escape".into();
    choice_weak.outputs[1].label = "Sacrifice".into();

    let mut zeph_escape = Node::new_dialogue([6300.0, 650.0]);
    dlg(&mut zeph_escape, &chars.zeph,
        "I've got a ship prepped in bay 7. We can be in neutral \
         space before anyone notices you're gone. Sometimes the \
         smart move is knowing when to fold.", "neutral");

    let mut end_escape = Node::new_end([6600.0, 650.0]);
    if let NodeType::End(ref mut e) = end_escape.node_type {
        e.tag = "escape".into();
    }

    let mut saya_sacrifice = Node::new_dialogue([6300.0, 800.0]);
    dlg(&mut saya_sacrifice, &chars.saya,
        "To give oneself entirely for harmony... that is the path \
         of the last constellation. A star that burns brightest \
         in its final moment.", "sad");

    let mut end_sacrifice = Node::new_end([6600.0, 800.0]);
    if let NodeType::End(ref mut e) = end_sacrifice.node_type {
        e.tag = "sacrifice".into();
    }

    // --- Capture ports ---
    type P = crate::model::port::PortId;
    let p = |n: &Node, d: &str, i: usize| -> P {
        match d { "i" => n.inputs[i].id, _ => n.outputs[i].id }
    };

    let conn_data: Vec<(Uuid, P, Uuid, P)> = vec![
        // Entry branch
        (cond_ready.id, p(&cond_ready,"o",0), voss_strong.id, p(&voss_strong,"i",0)),
        (cond_ready.id, p(&cond_ready,"o",1), voss_weak.id, p(&voss_weak,"i",0)),
        // Strong path
        (voss_strong.id, p(&voss_strong,"o",0), choice_strong.id, p(&choice_strong,"i",0)),
        (choice_strong.id, p(&choice_strong,"o",0), rng_peace.id, p(&rng_peace,"i",0)),
        (choice_strong.id, p(&choice_strong,"o",1), cond_conspiracy.id, p(&cond_conspiracy,"i",0)),
        (choice_strong.id, p(&choice_strong,"o",2), voss_force.id, p(&voss_force,"i",0)),
        (rng_peace.id, p(&rng_peace,"o",0), dlg_peace_ok.id, p(&dlg_peace_ok,"i",0)),
        (rng_peace.id, p(&rng_peace,"o",1), dlg_war.id, p(&dlg_war,"i",0)),
        (dlg_peace_ok.id, p(&dlg_peace_ok,"o",0), evt_peace.id, p(&evt_peace,"i",0)),
        (evt_peace.id, p(&evt_peace,"o",0), end_peace.id, p(&end_peace,"i",0)),
        (dlg_war.id, p(&dlg_war,"o",0), end_war.id, p(&end_war,"i",0)),
        // Truth path
        (cond_conspiracy.id, p(&cond_conspiracy,"o",0), elara_reveal.id, p(&elara_reveal,"i",0)),
        (cond_conspiracy.id, p(&cond_conspiracy,"o",1), voss_no_proof.id, p(&voss_no_proof,"i",0)),
        (elara_reveal.id, p(&elara_reveal,"o",0), end_betrayal.id, p(&end_betrayal,"i",0)),
        (voss_no_proof.id, p(&voss_no_proof,"o",0), end_no_proof.id, p(&end_no_proof,"i",0)),
        // Authority → war
        (voss_force.id, p(&voss_force,"o",0), end_forced.id, p(&end_forced,"i",0)),
        // Weak path
        (voss_weak.id, p(&voss_weak,"o",0), choice_weak.id, p(&choice_weak,"i",0)),
        (choice_weak.id, p(&choice_weak,"o",0), zeph_escape.id, p(&zeph_escape,"i",0)),
        (choice_weak.id, p(&choice_weak,"o",1), saya_sacrifice.id, p(&saya_sacrifice,"i",0)),
        (zeph_escape.id, p(&zeph_escape,"o",0), end_escape.id, p(&end_escape,"i",0)),
        (saya_sacrifice.id, p(&saya_sacrifice,"o",0), end_sacrifice.id, p(&end_sacrifice,"i",0)),
    ];

    let ids = Act3Ids {
        entry: cond_ready.id,
        peace_end: end_peace.id,
        war_end: end_war.id,
        betrayal_end: end_betrayal.id,
        escape_end: end_escape.id,
        sacrifice_end: end_sacrifice.id,
        all_node_ids: vec![
            cond_ready.id, voss_strong.id, choice_strong.id,
            rng_peace.id, dlg_peace_ok.id, evt_peace.id, end_peace.id,
            dlg_war.id, end_war.id,
            cond_conspiracy.id, elara_reveal.id, end_betrayal.id,
            voss_no_proof.id, end_no_proof.id,
            voss_force.id, end_forced.id,
            voss_weak.id, choice_weak.id,
            zeph_escape.id, end_escape.id,
            saya_sacrifice.id, end_sacrifice.id,
        ],
    };

    for n in [cond_ready, voss_strong, choice_strong,
        rng_peace, dlg_peace_ok, evt_peace, end_peace,
        dlg_war, end_war,
        cond_conspiracy, elara_reveal, end_betrayal,
        voss_no_proof, end_no_proof, voss_force, end_forced,
        voss_weak, choice_weak,
        zeph_escape, end_escape,
        saya_sacrifice, end_sacrifice]
    {
        graph.add_node(n);
    }

    for (fn_id, fp, tn_id, tp) in conn_data {
        graph.add_connection(fn_id, fp, tn_id, tp);
    }

    ids
}
