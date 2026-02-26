use super::TaleNodeApp;

impl TaleNodeApp {
    pub(super) fn render_collab_tab(&mut self, ui: &mut egui::Ui) {
        let action = crate::ui::collab_panel::show_collab_panel(
            ui,
            &self.collab_state,
            &mut self.collab_host_input,
            &mut self.collab_port_input,
            &self.settings.collab_username,
        );
        match action {
            crate::ui::collab_panel::CollabPanelAction::StartHosting { port } => {
                self.start_collab_hosting(port);
            }
            crate::ui::collab_panel::CollabPanelAction::JoinSession { host, port } => {
                self.start_collab_join(&host, port);
            }
            crate::ui::collab_panel::CollabPanelAction::Disconnect => {
                self.collab_disconnect();
            }
            crate::ui::collab_panel::CollabPanelAction::None => {}
        }
    }

    fn start_collab_hosting(&mut self, port: u16) {
        if self.collab_state.is_some() {
            self.status_message = Some((
                "Already in a collaboration session".to_string(),
                std::time::Instant::now(),
                true,
            ));
            return;
        }

        let graph_json = match serde_json::to_value(&self.graph) {
            Ok(v) => v,
            Err(e) => {
                self.status_message = Some((
                    format!("Failed to serialize graph: {e}"),
                    std::time::Instant::now(),
                    true,
                ));
                return;
            }
        };

        let username = self.settings.collab_username.clone();
        let result_tx = self.async_tx.clone();
        let (outgoing_tx, _outgoing_rx) = std::sync::mpsc::channel();

        self.tokio_runtime.spawn(async move {
            if let Err(e) = crate::collab::server::run_server(
                port,
                graph_json,
                username.clone(),
                result_tx.clone(),
            )
            .await
            {
                let _ = result_tx.send(
                    crate::app::async_runtime::AsyncResult::CollabError(e),
                );
            }
        });

        self.collab_state = Some(crate::collab::CollabState {
            mode: crate::collab::CollabMode::Hosting,
            peers: Vec::new(),
            local_username: self.settings.collab_username.clone(),
            host_addr: format!("0.0.0.0:{port}"),
            outgoing_tx,
        });

        self.status_message = Some((
            format!("Hosting collaboration on port {port}"),
            std::time::Instant::now(),
            false,
        ));
    }

    fn start_collab_join(&mut self, host: &str, port: u16) {
        if self.collab_state.is_some() {
            self.status_message = Some((
                "Already in a collaboration session".to_string(),
                std::time::Instant::now(),
                true,
            ));
            return;
        }

        let username = self.settings.collab_username.clone();
        let result_tx = self.async_tx.clone();
        let host_str = host.to_string();

        let (outgoing_tx_std, _outgoing_rx_std) = std::sync::mpsc::channel();
        let (outgoing_tx_tokio, outgoing_rx_tokio) =
            tokio::sync::mpsc::unbounded_channel();

        // Bridge from std mpsc to tokio mpsc not needed here — we store
        // tokio sender directly is cleaner but CollabState uses std::sync.
        // For now, the outgoing channel in CollabState is a placeholder;
        // the actual outgoing is the tokio channel passed to run_client.
        let _ = outgoing_tx_tokio; // Will be used when sending ops

        self.tokio_runtime.spawn(async move {
            if let Err(e) = crate::collab::client::run_client(
                &host_str,
                port,
                username,
                result_tx.clone(),
                outgoing_rx_tokio,
            )
            .await
            {
                let _ = result_tx.send(
                    crate::app::async_runtime::AsyncResult::CollabError(e),
                );
            }
        });

        let addr = format!("{host}:{port}");
        self.collab_state = Some(crate::collab::CollabState {
            mode: crate::collab::CollabMode::Joined,
            peers: Vec::new(),
            local_username: self.settings.collab_username.clone(),
            host_addr: addr.clone(),
            outgoing_tx: outgoing_tx_std,
        });

        self.status_message = Some((
            format!("Joining collaboration at {addr}"),
            std::time::Instant::now(),
            false,
        ));
    }

    fn collab_disconnect(&mut self) {
        self.collab_state = None;
        self.status_message = Some((
            "Disconnected from collaboration".to_string(),
            std::time::Instant::now(),
            false,
        ));
    }
}
