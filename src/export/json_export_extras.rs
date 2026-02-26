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
