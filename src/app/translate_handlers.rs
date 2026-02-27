use super::TaleNodeApp;

impl TaleNodeApp {
    pub(super) fn start_auto_translate(&mut self, target_locale: String) {
        let api_key = match &self.settings.deepl_api_key {
            Some(k) if !k.is_empty() => k.clone(),
            _ => {
                self.status_message = Some((
                    "DeepL API key not set — open Settings".to_string(),
                    std::time::Instant::now(),
                    true,
                ));
                return;
            }
        };
        let use_pro = self.settings.deepl_use_pro;

        // Collect untranslated strings
        let strings =
            crate::model::locale::collect_translatable_strings(&self.graph);
        let pairs: Vec<(String, String)> = strings
            .into_iter()
            .filter(|s| {
                self.graph
                    .locale
                    .get_translation(&s.key, &target_locale)
                    .is_none_or(|t| t.is_empty())
            })
            .map(|s| (s.key, s.default_text))
            .collect();

        if pairs.is_empty() {
            self.status_message = Some((
                "All strings already translated".to_string(),
                std::time::Instant::now(),
                false,
            ));
            return;
        }

        self.translation_in_progress = true;
        let tx = self.async_tx.clone();
        let locale = target_locale.clone();

        self.tokio_runtime.spawn(async move {
            let client =
                crate::integrations::deepl::DeepLClient::new(api_key, use_pro);
            let (translations, err) =
                client.translate_all(pairs, &locale).await;
            if !translations.is_empty() {
                let _ = tx.send(
                    crate::app::async_runtime::AsyncResult::TranslationDone {
                        locale,
                        translations,
                    },
                );
            }
            if let Some(e) = err {
                let _ = tx.send(
                    crate::app::async_runtime::AsyncResult::TranslationError(e),
                );
            }
        });

        self.status_message = Some((
            format!("Translating to '{target_locale}'..."),
            std::time::Instant::now(),
            false,
        ));
    }
}
