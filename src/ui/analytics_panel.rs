use egui::Ui;

use crate::validation::analytics::GraphAnalytics;

/// Draw the analytics panel showing graph statistics.
pub fn show_analytics_panel(ui: &mut Ui, stats: &GraphAnalytics) {
    ui.heading("Flow Analytics");
    ui.separator();

    // Node counts
    ui.label(format!("Total nodes: {}", stats.total_nodes));
    ui.label(format!("Total connections: {}", stats.total_connections));

    if !stats.nodes_by_type.is_empty() {
        ui.add_space(4.0);
        ui.label("Nodes by type:");
        let mut types: Vec<_> = stats.nodes_by_type.iter().collect();
        types.sort_by_key(|(k, _)| *k);
        for (name, count) in types {
            ui.indent(name, |ui| {
                ui.label(format!("{name}: {count}"));
            });
        }
    }

    // Path analysis
    ui.add_space(8.0);
    ui.separator();
    ui.label("Paths (Start to End):");
    ui.indent("paths", |ui| {
        ui.label(format!("Total paths: {}", stats.total_paths));
        if stats.total_paths > 0 {
            ui.label(format!("Longest path: {} steps", stats.longest_path));
            ui.label(format!("Shortest path: {} steps", stats.shortest_path));
        }
    });

    // Branching
    ui.add_space(8.0);
    ui.separator();
    ui.label("Branching:");
    ui.indent("branching", |ui| {
        ui.label(format!("Max fan-out: {}", stats.max_fan_out));
        if stats.avg_choices > 0.0 {
            ui.label(format!("Avg choices per Choice node: {:.1}", stats.avg_choices));
        }
    });

    // Connectivity
    ui.add_space(8.0);
    ui.separator();
    ui.label("Connectivity:");
    ui.indent("connectivity", |ui| {
        ui.label(format!("Unreachable nodes: {}", stats.unreachable_count));
        ui.label(format!("Dead ends: {}", stats.dead_end_count));
    });
}
