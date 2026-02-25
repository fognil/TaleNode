use std::time::Instant;

use super::TaleNodeApp;

impl TaleNodeApp {
    pub(super) fn do_export_locale_csv(&mut self) {
        let path = rfd::FileDialog::new()
            .add_filter("CSV", &["csv"])
            .set_file_name(format!("{}_localization.csv", self.project_name))
            .save_file();

        if let Some(path) = path {
            let csv = crate::export::locale_export::export_locale_csv(&self.graph);
            if let Err(e) = std::fs::write(&path, csv) {
                self.status_message = Some((
                    format!("Failed to write locale CSV: {e}"),
                    Instant::now(),
                    true,
                ));
            } else {
                self.status_message =
                    Some(("Locale CSV exported".to_string(), Instant::now(), false));
            }
        }
    }

    pub(super) fn do_import_locale_csv(&mut self) {
        let path = rfd::FileDialog::new()
            .add_filter("CSV", &["csv"])
            .pick_file();

        let Some(path) = path else { return };
        let contents = match std::fs::read_to_string(&path) {
            Ok(c) => c,
            Err(e) => {
                self.status_message =
                    Some((format!("Failed to read file: {e}"), Instant::now(), true));
                return;
            }
        };

        self.snapshot();
        match crate::export::locale_export::import_locale_csv(
            &contents,
            &mut self.graph.locale,
        ) {
            Ok(count) => {
                self.status_message = Some((
                    format!("Imported {count} translations"),
                    Instant::now(),
                    false,
                ));
            }
            Err(e) => {
                self.status_message = Some((
                    format!("Failed to import locale CSV: {e}"),
                    Instant::now(),
                    true,
                ));
            }
        }
    }
}
