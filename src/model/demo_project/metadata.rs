#[cfg(test)]
use std::collections::BTreeMap;
#[cfg(test)]
use uuid::Uuid;

#[cfg(test)]
use crate::model::bark::BarkLine;
#[cfg(test)]
use crate::model::graph::DialogueGraph;
#[cfg(test)]
use crate::model::group::NodeGroup;
#[cfg(test)]
use crate::model::locale::LocaleSettings;
#[cfg(test)]
use crate::model::node_types::VariableValue;
#[cfg(test)]
use crate::model::quest::{Objective, Quest, QuestStatus};
#[cfg(test)]
use crate::model::review::{NodeComment, ReviewStatus};
#[cfg(test)]
use crate::model::timeline::{Timeline, TimelineAction, TimelineStep};
#[cfg(test)]
use crate::model::world::{EntityCategory, EntityProperty, WorldEntity};

#[cfg(test)]
use super::act1::Act1Ids;
#[cfg(test)]
use super::act2::Act2Ids;
#[cfg(test)]
use super::act3::Act3Ids;
#[cfg(test)]
use super::characters::CharacterIds;

#[cfg(test)]
pub fn build_quests() -> Vec<Quest> {
    let mut q1 = Quest::new("Diplomatic Mission");
    q1.description = "Negotiate peace between the Solari Dominion \
        and Umbral Collective at Station Meridian.".into();
    q1.objectives = vec![
        Objective::new("Meet the faction leaders"),
        Objective::new("Gather evidence of the conspiracy"),
        Objective::new("Propose a resolution to both factions"),
    ];
    q1.status = QuestStatus::InProgress;

    let mut q2 = Quest::new("Shadows on Meridian");
    q2.description = "Strange signals and sabotage threaten the \
        station. Investigate the source.".into();
    q2.objectives = vec![
        Objective::new("Investigate the anomalous signal in Dr. Chen's lab"),
        Objective::new("Find evidence of sabotage"),
        Objective::new("Confront or report the saboteur"),
    ];

    let mut q3 = Quest::new("The Smuggler's Deal");
    q3.description = "Zeph needs a favor. A simple delivery job... \
        or is it?".into();
    let mut obj_deliver = Objective::new("Deliver Zeph's package to the contact");
    obj_deliver.completed = false;
    let mut obj_open = Objective::new("Open the package to see what's inside");
    obj_open.optional = true;
    q3.objectives = vec![obj_deliver, obj_open];

    vec![q1, q2, q3]
}

#[cfg(test)]
pub fn build_world_entities() -> Vec<WorldEntity> {
    let mut e1 = WorldEntity::new("Station Meridian", EntityCategory::Location);
    e1.description = "A neutral orbital station at the border between \
        Solari and Umbral space. Houses 3,000 permanent residents.".into();
    e1.properties = vec![
        EntityProperty { key: "capacity".into(), value: "3000".into() },
        EntityProperty { key: "allegiance".into(), value: "Neutral".into() },
    ];
    e1.tags = vec!["setting".into(), "neutral_zone".into()];

    let mut e2 = WorldEntity::new("Solari Dominion", EntityCategory::Lore);
    e2.description = "An expansionist faction that believes in unity \
        through technology and order.".into();
    e2.tags = vec!["faction".into(), "politics".into()];

    let mut e3 = WorldEntity::new("Umbral Collective", EntityCategory::Lore);
    e3.description = "An isolationist faction that values independence \
        and tradition above all.".into();
    e3.tags = vec!["faction".into(), "politics".into()];

    let mut e4 = WorldEntity::new("Neural Disruptor", EntityCategory::Item);
    e4.description = "A banned weapon capable of disrupting neural \
        implants. Possession is a capital offense on Meridian.".into();
    e4.properties = vec![
        EntityProperty { key: "damage".into(), value: "high".into() },
        EntityProperty { key: "legality".into(), value: "banned".into() },
    ];
    e4.icon = Some("icons/weapon_disruptor.png".into());

    let mut e5 = WorldEntity::new("Meridian Core", EntityCategory::Location);
    e5.description = "The station's central reactor and AI housing. \
        Access restricted to Level 5 personnel.".into();
    e5.properties = vec![
        EntityProperty { key: "access_level".into(), value: "5".into() },
    ];

    let mut e6 = WorldEntity::new("Ancient Signal", EntityCategory::Lore);
    e6.description = "A repeating signal of unknown origin detected \
        by Dr. Chen. Its pattern matches no known natural phenomenon. \
        It appears to be... deliberate.".into();
    e6.tags = vec!["mystery".into(), "plot_critical".into()];

    vec![e1, e2, e3, e4, e5, e6]
}

#[cfg(test)]
pub fn build_timelines(aria_greeting_id: Uuid) -> Vec<Timeline> {
    let mut t1 = Timeline::new("Arrival Cutscene");
    t1.description = "Opening sequence as the player docks at Meridian.".into();
    t1.steps = vec![
        TimelineStep::new(TimelineAction::Camera {
            target: "station_exterior".into(), duration: 3.0 }),
        { let mut s = TimelineStep::new(TimelineAction::Wait { seconds: 2.0 });
          s.delay = 0.5; s },
        TimelineStep::new(TimelineAction::Camera {
            target: "docking_bay".into(), duration: 2.0 }),
        TimelineStep::new(TimelineAction::Animation {
            target: "player_ship".into(), clip: "docking_sequence".into() }),
        { let mut s = TimelineStep::new(TimelineAction::Audio {
            clip: "station_ambient.wav".into(), volume: 0.6 });
          s.delay = 1.0; s },
        { let mut s = TimelineStep::new(TimelineAction::Dialogue {
            node_id: Some(aria_greeting_id) });
          s.delay = 0.5; s },
    ];

    let mut t2 = Timeline::new("Confrontation");
    t2.description = "The dramatic final confrontation sequence.".into();
    t2.steps = vec![
        TimelineStep::new(TimelineAction::Camera {
            target: "council_chamber".into(), duration: 1.5 }),
        TimelineStep::new(TimelineAction::Audio {
            clip: "tension_music.wav".into(), volume: 0.8 }),
        { let mut s = TimelineStep::new(TimelineAction::Animation {
            target: "kael".into(), clip: "stand_aggressive".into() });
          s.delay = 0.5; s },
        { let mut s = TimelineStep::new(TimelineAction::Animation {
            target: "thrix".into(), clip: "draw_weapon".into() });
          s.delay = 0.3; s },
        TimelineStep::new(TimelineAction::Wait { seconds: 1.0 }),
        TimelineStep::new(TimelineAction::SetVariable {
            key: "confrontation_started".into(), value: "true".into() }),
    ];

    vec![t1, t2]
}

#[cfg(test)]
pub fn build_barks(chars: &CharacterIds) -> BTreeMap<Uuid, Vec<BarkLine>> {
    let mut barks = BTreeMap::new();

    // Voss barks
    let mut v1 = BarkLine::new("The station's on edge. Keep your head down.");
    v1.condition_variable = Some("chapter".into());
    v1.condition_value = Some(VariableValue::Int(1));
    let mut v2 = BarkLine::new("Progress. Good. Don't let up now.");
    v2.condition_variable = Some("peace_progress".into());
    v2.condition_value = Some(VariableValue::Int(2));
    let mut v3 = BarkLine::new("If this goes south, I'm holding you responsible.");
    v3.weight = 0.5;
    barks.insert(chars.voss.id, vec![v1, v2, v3]);

    // ARIA barks
    let a1 = BarkLine::new("Reminder: Station curfew begins at 2200 hours.");
    let mut a2 = BarkLine::new("Security alert levels remain elevated.");
    a2.condition_variable = Some("chapter".into());
    a2.condition_value = Some(VariableValue::Int(2));
    barks.insert(chars.aria.id, vec![a1, a2]);

    // Ambient (attached to Zeph as bar NPC)
    let z1 = BarkLine::new("Did you hear about the signal? Creepy stuff...");
    let mut z2 = BarkLine::new("Another diplomat? This station's getting crowded.");
    z2.weight = 0.8;
    barks.insert(chars.zeph.id, vec![z1, z2]);

    barks
}

#[cfg(test)]
pub fn build_locale(graph: &DialogueGraph) -> LocaleSettings {
    let mut settings = LocaleSettings {
        default_locale: "en".into(),
        extra_locales: vec!["fr".into(), "ja".into()],
        ..Default::default()
    };

    // Translate a subset of dialogue nodes
    let translations: &[(&str, &str, &str)] = &[
        ("Welcome to Station Meridian", "Bienvenue sur la Station Meridian",
         "\u{30b9}\u{30c6}\u{30fc}\u{30b7}\u{30e7}\u{30f3}\u{30fb}\u{30e1}\u{30ea}\u{30c7}\u{30a3}\u{30a2}\u{30f3}\u{3078}\u{3088}\u{3046}\u{3053}\u{305d}"),
        ("What would you like to do?", "Que souhaitez-vous faire ?",
         "\u{4f55}\u{3092}\u{3057}\u{307e}\u{3059}\u{304b}\u{ff1f}"),
        ("I'm Commander Voss", "Je suis le Commandant Voss",
         "\u{30f4}\u{30a9}\u{30b9}\u{53f8}\u{4ee4}\u{5b98}\u{3067}\u{3059}"),
    ];

    // Find nodes whose text starts with these phrases and build locale keys
    for node in graph.nodes.values() {
        if let crate::model::node::NodeType::Dialogue(ref d) = node.node_type {
            let uuid8 = &node.id.to_string()[..8];
            for &(en_prefix, fr, ja) in translations {
                if d.text.starts_with(en_prefix) {
                    let key = format!("dlg_{uuid8}");
                    let mut map = BTreeMap::new();
                    map.insert("fr".into(), fr.into());
                    map.insert("ja".into(), ja.into());
                    settings.translations.insert(key, map);
                    break;
                }
            }
        }
    }

    settings
}

#[cfg(test)]
pub fn build_groups(a1: &Act1Ids, a2: &Act2Ids, a3: &Act3Ids) -> Vec<NodeGroup> {
    let mut g1 = NodeGroup::new("Act 1 \u{2014} Arrival");
    g1.color = [100, 150, 255, 30];
    g1.node_ids = a1.all_node_ids.clone();

    let mut g2 = NodeGroup::new("Act 2 \u{2014} Investigation");
    g2.color = [100, 200, 100, 30];
    g2.node_ids = a2.all_node_ids.clone();

    let mut g3 = NodeGroup::new("Act 3 \u{2014} Resolution");
    g3.color = [220, 80, 80, 30];
    g3.node_ids = a3.all_node_ids.clone();

    let mut g4 = NodeGroup::new("Solari Path");
    g4.color = [220, 190, 80, 30];
    g4.node_ids = a2.kael_nodes.clone();

    let mut g5 = NodeGroup::new("Umbral Path");
    g5.color = [160, 80, 180, 30];
    g5.node_ids = a2.thrix_nodes.clone();

    let mut g6 = NodeGroup::new("Side Quests");
    g6.color = [80, 200, 180, 30];
    g6.node_ids = vec![a2.saya_node];

    vec![g1, g2, g3, g4, g5, g6]
}

#[cfg(test)]
pub fn apply_reviews(graph: &mut DialogueGraph, a1: &Act1Ids, a2: &Act2Ids) {
    for &id in &a1.all_node_ids {
        graph.review_statuses.insert(id, ReviewStatus::Approved);
    }
    for (i, &id) in a2.all_node_ids.iter().enumerate() {
        let status = if i < a2.all_node_ids.len() / 2 {
            ReviewStatus::Approved
        } else {
            ReviewStatus::NeedsReview
        };
        graph.review_statuses.insert(id, status);
    }
    // Act 3 stays Draft (default)
}

#[cfg(test)]
pub fn apply_comments(graph: &mut DialogueGraph, a1: &Act1Ids, a2: &Act2Ids, a3: &Act3Ids) {
    let comments = [
        (a1.aria_greeting, "Great opening \u{2014} sets the sci-fi tone immediately."),
        (a1.hub_choice, "Consider adding a 'check inventory' option."),
        (a2.elara_secret, "This is the emotional core. Polish carefully."),
        (a2.hub, "Balance faction paths \u{2014} Thrix feels shorter than Kael."),
        (a3.peace_end, "Verify that peace is achievable with typical playthroughs."),
        (a3.sacrifice_end, "Playtest this ending \u{2014} is it satisfying?"),
    ];
    for (nid, text) in comments {
        graph.comments.push(NodeComment::new(nid, text.to_string()));
    }
}

#[cfg(test)]
pub fn apply_tags(
    graph: &mut DialogueGraph,
    a1: &Act1Ids, a2: &Act2Ids, a3: &Act3Ids,
) {
    let mut tags: BTreeMap<Uuid, Vec<String>> = BTreeMap::new();
    for &id in &a1.all_node_ids {
        tags.insert(id, vec!["act_1".into(), "main_path".into()]);
    }
    for &id in &a2.all_node_ids {
        tags.insert(id, vec!["act_2".into()]);
    }
    // Tag specific nodes
    if let Some(t) = tags.get_mut(&a2.elara_secret) {
        t.push("emotional".into());
        t.push("branching".into());
    }
    if let Some(t) = tags.get_mut(&a2.saya_node) {
        t.push("side_quest".into());
    }
    for &id in &a3.all_node_ids {
        tags.entry(id).or_default().push("act_3".into());
    }
    for &id in &[a3.peace_end, a3.war_end, a3.betrayal_end,
        a3.escape_end, a3.sacrifice_end]
    {
        tags.entry(id).or_default().push("ending".into());
    }
    tags.entry(a3.sacrifice_end).or_default().push("emotional".into());
    graph.node_tags = tags;
}
