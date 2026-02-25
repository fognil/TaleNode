use super::TaleNodeApp;

/// A destructive action that requires user confirmation before executing.
#[derive(Debug, Clone)]
pub(super) enum PendingAction {
    NewProject,
    RestoreVersion(usize),
}

impl PendingAction {
    fn message(&self) -> &str {
        match self {
            Self::NewProject => "Create a new project? Unsaved changes will be lost.",
            Self::RestoreVersion(_) => "Restore this version? Current graph will be replaced.",
        }
    }
}

impl TaleNodeApp {
    /// Show a modal confirmation dialog if there is a pending action.
    /// Must be called in `update()`.
    pub(super) fn show_confirmation_dialog(&mut self, ctx: &egui::Context) {
        let Some(action) = self.pending_confirmation.clone() else {
            return;
        };

        let mut open = true;
        egui::Window::new("Confirm")
            .open(&mut open)
            .collapsible(false)
            .resizable(false)
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .show(ctx, |ui| {
                ui.label(action.message());
                ui.add_space(8.0);
                ui.horizontal(|ui| {
                    if ui.button("Yes").clicked() {
                        self.execute_pending_action(&action);
                        self.pending_confirmation = None;
                    }
                    if ui.button("No").clicked() {
                        self.pending_confirmation = None;
                    }
                });
            });

        // Window close button (X) also cancels
        if !open {
            self.pending_confirmation = None;
        }
    }

    fn execute_pending_action(&mut self, action: &PendingAction) {
        match action {
            PendingAction::NewProject => {
                self.do_new_project();
            }
            PendingAction::RestoreVersion(id) => {
                let mut project = crate::model::project::Project {
                    version: "1.0".to_string(),
                    name: self.project_name.clone(),
                    graph: self.graph.clone(),
                    versions: self.versions.clone(),
                    dock_layout: None,
                };
                if let Some(_old_graph) = project.restore_version(*id) {
                    self.snapshot();
                    self.graph = project.graph;
                }
            }
        }
    }
}
