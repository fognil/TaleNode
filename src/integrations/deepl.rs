use serde::Deserialize;

const FREE_API_URL: &str = "https://api-free.deepl.com/v2/translate";
const PRO_API_URL: &str = "https://api.deepl.com/v2/translate";

/// Max texts per single DeepL API call.
const BATCH_SIZE: usize = 50;

pub struct DeepLClient {
    api_key: String,
    use_pro: bool,
    client: reqwest::Client,
}

#[derive(Deserialize)]
struct DeepLResponse {
    translations: Vec<DeepLTranslation>,
}

#[derive(Deserialize)]
struct DeepLTranslation {
    text: String,
}

impl DeepLClient {
    pub fn new(api_key: String, use_pro: bool) -> Self {
        Self {
            api_key,
            use_pro,
            client: reqwest::Client::new(),
        }
    }

    fn api_url(&self) -> &str {
        if self.use_pro {
            PRO_API_URL
        } else {
            FREE_API_URL
        }
    }

    /// Translate a batch of texts (max 50) to the target language.
    async fn translate_batch(
        &self,
        texts: &[String],
        target_lang: &str,
    ) -> Result<Vec<String>, String> {
        if texts.is_empty() {
            return Ok(Vec::new());
        }

        let mut form: Vec<(&str, String)> = Vec::new();
        for t in texts {
            form.push(("text", t.clone()));
        }
        form.push(("target_lang", target_lang.to_uppercase()));

        let resp = self
            .client
            .post(self.api_url())
            .header("Authorization", format!("DeepL-Auth-Key {}", self.api_key))
            .form(&form)
            .send()
            .await
            .map_err(|e| format!("DeepL request failed: {e}"))?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            return Err(format!("DeepL API error {status}: {body}"));
        }

        let data: DeepLResponse = resp
            .json()
            .await
            .map_err(|e| format!("DeepL parse error: {e}"))?;

        Ok(data.translations.into_iter().map(|t| t.text).collect())
    }

    /// Translate all (key, source_text) pairs to the target language.
    /// Returns (key, translated_text) pairs and an optional error if a batch failed.
    /// On partial failure, successfully translated pairs are still returned.
    pub async fn translate_all(
        &self,
        pairs: Vec<(String, String)>,
        target_lang: &str,
    ) -> (Vec<(String, String)>, Option<String>) {
        let total = pairs.len();
        let mut result = Vec::with_capacity(total);

        for chunk in pairs.chunks(BATCH_SIZE) {
            let texts: Vec<String> = chunk.iter().map(|(_, t)| t.clone()).collect();
            match self.translate_batch(&texts, target_lang).await {
                Ok(translated) => {
                    for (i, (key, _)) in chunk.iter().enumerate() {
                        if let Some(text) = translated.get(i) {
                            result.push((key.clone(), text.clone()));
                        }
                    }
                }
                Err(e) => {
                    let msg = format!(
                        "{} (translated {}/{} before failure)",
                        e,
                        result.len(),
                        total
                    );
                    return (result, Some(msg));
                }
            }
        }

        (result, None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn client_uses_correct_url() {
        let free = DeepLClient::new("key".to_string(), false);
        assert_eq!(free.api_url(), FREE_API_URL);

        let pro = DeepLClient::new("key".to_string(), true);
        assert_eq!(pro.api_url(), PRO_API_URL);
    }

    #[test]
    fn batch_size_constant() {
        assert_eq!(BATCH_SIZE, 50);
    }

    #[test]
    fn deserialize_response() {
        let json = r#"{"translations":[{"text":"Bonjour"},{"text":"Monde"}]}"#;
        let resp: DeepLResponse = serde_json::from_str(json).unwrap();
        assert_eq!(resp.translations.len(), 2);
        assert_eq!(resp.translations[0].text, "Bonjour");
        assert_eq!(resp.translations[1].text, "Monde");
    }

    #[test]
    fn deserialize_single_translation() {
        let json = r#"{"translations":[{"text":"Hola"}]}"#;
        let resp: DeepLResponse = serde_json::from_str(json).unwrap();
        assert_eq!(resp.translations.len(), 1);
        assert_eq!(resp.translations[0].text, "Hola");
    }

    #[test]
    fn deserialize_empty_translations() {
        let json = r#"{"translations":[]}"#;
        let resp: DeepLResponse = serde_json::from_str(json).unwrap();
        assert!(resp.translations.is_empty());
    }

    #[test]
    fn client_new_creates_instance() {
        let client = DeepLClient::new("my-key".to_string(), true);
        assert_eq!(client.api_key, "my-key");
        assert!(client.use_pro);
    }
}
