use crate::model::graph::DialogueGraph;

use super::json_export_types::{
    ExportedEntityProperty, ExportedTimeline, ExportedTimelineStep, ExportedWorldEntity,
};

/// Build exported timelines.
pub fn build_timelines(graph: &DialogueGraph) -> Vec<ExportedTimeline> {
    graph
        .timelines
        .iter()
        .map(|t| ExportedTimeline {
            name: t.name.clone(),
            description: t.description.clone(),
            steps: t
                .steps
                .iter()
                .map(|s| ExportedTimelineStep {
                    action: timeline_action_to_json(&s.action),
                    delay: s.delay,
                })
                .collect(),
            loop_playback: t.loop_playback,
        })
        .collect()
}

fn timeline_action_to_json(action: &crate::model::timeline::TimelineAction) -> serde_json::Value {
    use crate::model::timeline::TimelineAction;
    match action {
        TimelineAction::Dialogue { node_id } => serde_json::json!({
            "type": "dialogue", "node_id": node_id.map(|id| id.to_string()),
        }),
        TimelineAction::Camera { target, duration } => serde_json::json!({
            "type": "camera", "target": target, "duration": duration,
        }),
        TimelineAction::Animation { target, clip } => serde_json::json!({
            "type": "animation", "target": target, "clip": clip,
        }),
        TimelineAction::Audio { clip, volume } => serde_json::json!({
            "type": "audio", "clip": clip, "volume": volume,
        }),
        TimelineAction::Wait { seconds } => serde_json::json!({
            "type": "wait", "seconds": seconds,
        }),
        TimelineAction::SetVariable { key, value } => serde_json::json!({
            "type": "set_variable", "key": key, "value": value,
        }),
        TimelineAction::Custom { action_type, data } => serde_json::json!({
            "type": "custom", "action_type": action_type, "data": data,
        }),
    }
}

/// Build exported world entities.
pub fn build_world_entities(graph: &DialogueGraph) -> Vec<ExportedWorldEntity> {
    graph
        .world_entities
        .iter()
        .map(|e| ExportedWorldEntity {
            name: e.name.clone(),
            category: e.category.label().to_string(),
            description: e.description.clone(),
            tags: e.tags.clone(),
            properties: e
                .properties
                .iter()
                .map(|p| ExportedEntityProperty {
                    key: p.key.clone(),
                    value: p.value.clone(),
                })
                .collect(),
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::graph::DialogueGraph;
    use crate::model::timeline::{Timeline, TimelineStep, TimelineAction};
    use crate::model::world::{WorldEntity, EntityCategory, EntityProperty};

    #[test]
    fn build_timelines_empty() {
        let graph = DialogueGraph::new();
        let result = build_timelines(&graph);
        assert!(result.is_empty());
    }

    #[test]
    fn build_timelines_with_steps() {
        let mut graph = DialogueGraph::new();
        let mut tl = Timeline::new("Cutscene");
        tl.description = "Opening".to_string();
        tl.steps.push(TimelineStep::new(TimelineAction::Wait { seconds: 2.0 }));
        tl.steps.push(TimelineStep::new(TimelineAction::Camera {
            target: "player".into(),
            duration: 1.5,
        }));
        tl.loop_playback = true;
        graph.timelines.push(tl);

        let result = build_timelines(&graph);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].name, "Cutscene");
        assert_eq!(result[0].description, "Opening");
        assert_eq!(result[0].steps.len(), 2);
        assert!(result[0].loop_playback);
        assert_eq!(result[0].steps[0].action["type"], "wait");
        assert_eq!(result[0].steps[1].action["type"], "camera");
    }

    #[test]
    fn build_world_entities_empty() {
        let graph = DialogueGraph::new();
        let result = build_world_entities(&graph);
        assert!(result.is_empty());
    }

    #[test]
    fn build_world_entities_with_properties() {
        let mut graph = DialogueGraph::new();
        let mut entity = WorldEntity::new("Sword", EntityCategory::Item);
        entity.description = "A sharp blade".to_string();
        entity.tags.push("weapon".to_string());
        entity.properties.push(EntityProperty {
            key: "damage".to_string(),
            value: "50".to_string(),
        });
        graph.world_entities.push(entity);

        let result = build_world_entities(&graph);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].name, "Sword");
        assert_eq!(result[0].category, "Item");
        assert_eq!(result[0].description, "A sharp blade");
        assert_eq!(result[0].tags, vec!["weapon"]);
        assert_eq!(result[0].properties.len(), 1);
        assert_eq!(result[0].properties[0].key, "damage");
        assert_eq!(result[0].properties[0].value, "50");
    }

    #[test]
    fn timeline_action_dialogue_json() {
        let action = TimelineAction::Dialogue { node_id: None };
        let json = timeline_action_to_json(&action);
        assert_eq!(json["type"], "dialogue");
        assert!(json["node_id"].is_null());
    }

    #[test]
    fn timeline_action_camera_json() {
        let action = TimelineAction::Camera {
            target: "npc".into(),
            duration: 3.0,
        };
        let json = timeline_action_to_json(&action);
        assert_eq!(json["type"], "camera");
        assert_eq!(json["target"], "npc");
        assert_eq!(json["duration"], 3.0);
    }

    #[test]
    fn timeline_action_wait_json() {
        let action = TimelineAction::Wait { seconds: 1.5 };
        let json = timeline_action_to_json(&action);
        assert_eq!(json["type"], "wait");
        assert_eq!(json["seconds"], 1.5);
    }

    #[test]
    fn timeline_action_custom_json() {
        let action = TimelineAction::Custom {
            action_type: "shake".into(),
            data: "intensity=5".into(),
        };
        let json = timeline_action_to_json(&action);
        assert_eq!(json["type"], "custom");
        assert_eq!(json["action_type"], "shake");
        assert_eq!(json["data"], "intensity=5");
    }
}
