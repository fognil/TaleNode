use egui::Ui;

use crate::collab::{CollabMode, CollabState};

pub enum CollabPanelAction {
    None,
    StartHosting { port: u16 },
    JoinSession { host: String, port: u16 },
    Disconnect,
}

pub fn show_collab_panel(
    ui: &mut Ui,
    collab_state: &Option<CollabState>,
    host_input: &mut String,
    port_input: &mut u16,
    username: &str,
) -> CollabPanelAction {
    let mut action = CollabPanelAction::None;

    let mode = collab_state
        .as_ref()
        .map(|s| s.mode)
        .unwrap_or(CollabMode::Offline);

    // Status header
    ui.horizontal(|ui| {
        let status_text = match mode {
            CollabMode::Offline => "Offline",
            CollabMode::Hosting => "Hosting",
            CollabMode::Joined => "Connected",
        };
        ui.label(format!("Status: {status_text}"));
        if let Some(ref state) = collab_state {
            ui.label(format!("| {} peers", state.peer_count()));
        }
    });

    ui.separator();

    match mode {
        CollabMode::Offline => {
            show_offline_ui(ui, host_input, port_input, username, &mut action);
        }
        CollabMode::Hosting | CollabMode::Joined => {
            show_connected_ui(ui, collab_state, &mut action);
        }
    }

    action
}

fn show_offline_ui(
    ui: &mut Ui,
    host_input: &mut String,
    port_input: &mut u16,
    username: &str,
    action: &mut CollabPanelAction,
) {
    ui.label(format!("Username: {username}"));
    ui.add_space(4.0);

    // Host controls
    ui.group(|ui| {
        ui.strong("Host a Session");
        ui.horizontal(|ui| {
            ui.label("Port:");
            ui.add(egui::DragValue::new(port_input).range(1024..=65535));
        });
        if ui.button("Start Hosting").clicked() {
            *action = CollabPanelAction::StartHosting { port: *port_input };
        }
    });

    ui.add_space(8.0);

    // Join controls
    ui.group(|ui| {
        ui.strong("Join a Session");
        ui.horizontal(|ui| {
            ui.label("Host:");
            ui.text_edit_singleline(host_input);
        });
        ui.horizontal(|ui| {
            ui.label("Port:");
            ui.add(egui::DragValue::new(port_input).range(1024..=65535));
        });
        if ui.button("Join").clicked() {
            *action = CollabPanelAction::JoinSession {
                host: host_input.clone(),
                port: *port_input,
            };
        }
    });
}

fn show_connected_ui(
    ui: &mut Ui,
    collab_state: &Option<CollabState>,
    action: &mut CollabPanelAction,
) {
    if let Some(ref state) = collab_state {
        ui.label(format!("Connected as: {}", state.local_username));
        ui.label(format!("Address: {}", state.host_addr));

        ui.add_space(4.0);

        // Peer list
        if !state.peers.is_empty() {
            ui.strong("Connected Peers:");
            for peer in &state.peers {
                let color = egui::Color32::from_rgb(
                    peer.color[0],
                    peer.color[1],
                    peer.color[2],
                );
                ui.horizontal(|ui| {
                    let (rect, _) = ui.allocate_exact_size(
                        egui::Vec2::new(10.0, 10.0),
                        egui::Sense::hover(),
                    );
                    ui.painter().rect_filled(rect, 2.0, color);
                    ui.label(&peer.username);
                    if !peer.selected_nodes.is_empty() {
                        ui.label(format!(
                            "({} selected)",
                            peer.selected_nodes.len()
                        ));
                    }
                });
            }
        }
    }

    ui.add_space(8.0);
    if ui.button("Disconnect").clicked() {
        *action = CollabPanelAction::Disconnect;
    }
}
