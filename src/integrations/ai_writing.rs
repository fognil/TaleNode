use serde::{Deserialize, Serialize};

/// Supported AI providers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum AIProvider {
    #[default]
    OpenAI,
    Anthropic,
    Gemini,
}

impl AIProvider {
    pub fn label(self) -> &'static str {
        match self {
            Self::OpenAI => "OpenAI-compatible",
            Self::Anthropic => "Anthropic Claude",
            Self::Gemini => "Google Gemini",
        }
    }

    pub const ALL: [AIProvider; 3] = [AIProvider::OpenAI, AIProvider::Anthropic, AIProvider::Gemini];
}

pub fn default_ai_base_url() -> String {
    "https://api.openai.com/v1".to_string()
}

pub fn default_ai_model() -> String {
    "gpt-4o".to_string()
}

pub fn default_gemini_base_url() -> String {
    "https://generativelanguage.googleapis.com/v1beta".to_string()
}

pub fn default_gemini_model() -> String {
    "gemini-2.0-flash".to_string()
}

/// AI writing assistant HTTP client.
pub struct AIWritingClient {
    provider: AIProvider,
    api_key: String,
    base_url: String,
    model: String,
    client: reqwest::Client,
}

impl AIWritingClient {
    pub fn new(
        provider: AIProvider,
        api_key: String,
        base_url: String,
        model: String,
    ) -> Self {
        Self {
            provider,
            api_key,
            base_url,
            model,
            client: reqwest::Client::new(),
        }
    }

    /// Suggest alternative dialogue lines for a given speaker/emotion/text.
    pub async fn suggest_dialogue(
        &self,
        speaker: &str,
        emotion: &str,
        text: &str,
        instruction: &str,
    ) -> Result<Vec<String>, String> {
        let extra = if instruction.is_empty() {
            String::new()
        } else {
            format!("\nAdditional instruction: {instruction}")
        };
        let prompt = format!(
            "You are a dialogue writer for a video game. \
             A character named \"{speaker}\" with emotion \"{emotion}\" says: \"{text}\"\n\
             Suggest 3 alternative dialogue lines that keep the same meaning \
             but vary in tone or phrasing.{extra}\n\
             Return ONLY a JSON array of 3 strings, no other text."
        );
        let response = self.call_api(&prompt).await?;
        parse_json_string_array(&response, 3)
    }

    /// Generate choice option texts for a choice node.
    pub async fn generate_choices(
        &self,
        prompt: &str,
        context: &str,
        count: usize,
    ) -> Result<Vec<String>, String> {
        let system = format!(
            "You are a dialogue writer for a video game. \
             Given a choice prompt and context, generate exactly {count} \
             distinct player response options.\n\
             Choice prompt: \"{prompt}\"\n\
             Context: {context}\n\
             Return ONLY a JSON array of {count} strings, no other text."
        );
        let response = self.call_api(&system).await?;
        parse_json_string_array(&response, count)
    }

    /// Analyze tone consistency for a dialogue line.
    pub async fn check_tone(
        &self,
        speaker: &str,
        emotion: &str,
        text: &str,
    ) -> Result<String, String> {
        let prompt = format!(
            "You are a dialogue editor for a video game. \
             Analyze this line for tone consistency:\n\
             Speaker: \"{speaker}\"\nEmotion tag: \"{emotion}\"\n\
             Line: \"{text}\"\n\n\
             Briefly assess: does the line match the emotion? \
             Any grammar issues? Suggestions for improvement? \
             Keep the response under 100 words."
        );
        self.call_api(&prompt).await
    }

    async fn call_api(&self, prompt: &str) -> Result<String, String> {
        match self.provider {
            AIProvider::OpenAI => self.call_openai(prompt).await,
            AIProvider::Anthropic => self.call_anthropic(prompt).await,
            AIProvider::Gemini => self.call_gemini(prompt).await,
        }
    }

    async fn call_openai(&self, prompt: &str) -> Result<String, String> {
        let url = format!("{}/chat/completions", self.base_url);
        let body = serde_json::json!({
            "model": self.model,
            "messages": [
                { "role": "user", "content": prompt }
            ],
            "temperature": 0.8
        });

        let resp = self
            .client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await
            .map_err(|e| format!("AI request failed: {e}"))?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body_text = resp.text().await.unwrap_or_default();
            return Err(format!("AI API error {status}: {body_text}"));
        }

        let data: serde_json::Value = resp
            .json()
            .await
            .map_err(|e| format!("AI parse error: {e}"))?;

        data["choices"][0]["message"]["content"]
            .as_str()
            .map(|s| s.to_string())
            .ok_or_else(|| "Unexpected API response format".to_string())
    }

    async fn call_anthropic(&self, prompt: &str) -> Result<String, String> {
        let url = "https://api.anthropic.com/v1/messages";
        let body = serde_json::json!({
            "model": self.model,
            "max_tokens": 1024,
            "messages": [
                { "role": "user", "content": prompt }
            ]
        });

        let resp = self
            .client
            .post(url)
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", "2023-06-01")
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await
            .map_err(|e| format!("Anthropic request failed: {e}"))?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body_text = resp.text().await.unwrap_or_default();
            return Err(format!("Anthropic API error {status}: {body_text}"));
        }

        let data: serde_json::Value = resp
            .json()
            .await
            .map_err(|e| format!("Anthropic parse error: {e}"))?;

        data["content"][0]["text"]
            .as_str()
            .map(|s| s.to_string())
            .ok_or_else(|| "Unexpected Anthropic response format".to_string())
    }

    async fn call_gemini(&self, prompt: &str) -> Result<String, String> {
        let url = format!(
            "{}/models/{}:generateContent?key={}",
            self.base_url, self.model, self.api_key
        );
        let body = serde_json::json!({
            "contents": [{"parts": [{"text": prompt}]}],
            "generationConfig": {"temperature": 0.8}
        });

        let resp = self
            .client
            .post(&url)
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await
            .map_err(|e| format!("Gemini request failed: {e}"))?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body_text = resp.text().await.unwrap_or_default();
            return Err(format!("Gemini API error {status}: {body_text}"));
        }

        let data: serde_json::Value = resp
            .json()
            .await
            .map_err(|e| format!("Gemini parse error: {e}"))?;

        data["candidates"][0]["content"]["parts"][0]["text"]
            .as_str()
            .map(|s| s.to_string())
            .ok_or_else(|| "Unexpected Gemini response format".to_string())
    }
}

/// Parse a JSON string array from the AI response text.
fn parse_json_string_array(text: &str, expected: usize) -> Result<Vec<String>, String> {
    // Try to find a JSON array in the response (AI may add surrounding text)
    let trimmed = text.trim();
    let start = trimmed.find('[').unwrap_or(0);
    let end = trimmed.rfind(']').map(|i| i + 1).unwrap_or(trimmed.len());
    let slice = &trimmed[start..end];

    let arr: Vec<String> =
        serde_json::from_str(slice).map_err(|e| format!("Failed to parse AI response: {e}"))?;

    if arr.is_empty() {
        return Err("AI returned empty suggestions".to_string());
    }

    // Truncate or pad to expected count
    let mut result = arr;
    result.truncate(expected);
    Ok(result)
}
/// Fetch chat-compatible models from an OpenAI-compatible `/models` endpoint.
pub async fn fetch_openai_models(base_url: &str, api_key: &str) -> Result<Vec<String>, String> {
    let resp = reqwest::Client::new()
        .get(format!("{base_url}/models"))
        .header("Authorization", format!("Bearer {api_key}"))
        .send().await.map_err(|e| format!("Failed to fetch models: {e}"))?;
    if !resp.status().is_success() {
        let s = resp.status();
        return Err(format!("Models API error {s}: {}", resp.text().await.unwrap_or_default()));
    }
    let data: serde_json::Value =
        resp.json().await.map_err(|e| format!("Models parse error: {e}"))?;
    let prefixes = ["gpt-", "o1", "o3", "o4", "chatgpt-"];
    let mut models: Vec<String> = data["data"]
        .as_array()
        .map(|arr| {
            arr.iter()
                .filter_map(|m| m["id"].as_str().map(String::from))
                .filter(|id| prefixes.iter().any(|p| id.starts_with(p)))
                .collect()
        })
        .unwrap_or_default();
    models.sort();
    Ok(models)
}
/// Fetch main Gemini models (filters out nano, embedding, dated variants).
pub async fn fetch_gemini_models(base_url: &str, api_key: &str) -> Result<Vec<String>, String> {
    let resp = reqwest::Client::new()
        .get(format!("{base_url}/models?key={api_key}&pageSize=200"))
        .send().await.map_err(|e| format!("Failed to fetch Gemini models: {e}"))?;
    if !resp.status().is_success() {
        let s = resp.status();
        return Err(format!("Gemini API error {s}: {}", resp.text().await.unwrap_or_default()));
    }
    let data: serde_json::Value =
        resp.json().await.map_err(|e| format!("Gemini models parse error: {e}"))?;
    fn is_main_model(name: &str) -> bool {
        name.starts_with("gemini-")
            && !name.contains("nano")
            && !name.contains("embedding")
            && !name.contains("aqa")
            && !name.chars().last().is_some_and(|c| c.is_ascii_digit())
    }
    let mut models: Vec<String> = data["models"]
        .as_array()
        .map(|arr| {
            arr.iter()
                .filter_map(|m| {
                    m["name"].as_str().map(|n| n.strip_prefix("models/").unwrap_or(n).to_string())
                })
                .filter(|n| is_main_model(n))
                .collect()
        })
        .unwrap_or_default();
    models.sort();
    Ok(models)
}
/// Hardcoded Anthropic model list (no public list endpoint).
pub fn hardcoded_anthropic_models() -> Vec<String> {
    vec![
        "claude-opus-4-6".to_string(),
        "claude-sonnet-4-6".to_string(),
        "claude-haiku-4-5-20251001".to_string(),
        "claude-sonnet-4-20250514".to_string(),
        "claude-opus-4-20250514".to_string(),
    ]
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn provider_default_is_openai() {
        assert_eq!(AIProvider::default(), AIProvider::OpenAI);
    }

    #[test]
    fn provider_labels() {
        assert_eq!(AIProvider::OpenAI.label(), "OpenAI-compatible");
        assert_eq!(AIProvider::Anthropic.label(), "Anthropic Claude");
        assert_eq!(AIProvider::Gemini.label(), "Google Gemini");
        assert_eq!(AIProvider::ALL.len(), 3);
    }

    #[test]
    fn default_urls_and_model() {
        assert_eq!(default_ai_base_url(), "https://api.openai.com/v1");
        assert_eq!(default_ai_model(), "gpt-4o");
        assert_eq!(
            default_gemini_base_url(),
            "https://generativelanguage.googleapis.com/v1beta"
        );
        assert_eq!(default_gemini_model(), "gemini-2.0-flash");
    }

    #[test]
    fn parse_json_array_simple() {
        let input = r#"["Hello there", "Hi friend", "Greetings"]"#;
        let result = parse_json_string_array(input, 3).unwrap();
        assert_eq!(result.len(), 3);
        assert_eq!(result[0], "Hello there");
    }

    #[test]
    fn parse_json_array_with_surrounding_text() {
        let input = "Here are suggestions:\n[\"A\", \"B\", \"C\"]\nHope that helps!";
        let result = parse_json_string_array(input, 3).unwrap();
        assert_eq!(result.len(), 3);
    }

    #[test]
    fn parse_json_array_truncates() {
        let input = r#"["A", "B", "C", "D"]"#;
        let result = parse_json_string_array(input, 2).unwrap();
        assert_eq!(result.len(), 2);
    }

    #[test]
    fn parse_json_array_error() {
        let input = "not json at all";
        assert!(parse_json_string_array(input, 3).is_err());
    }

    #[test]
    fn provider_serialization() {
        let json = serde_json::to_string(&AIProvider::Anthropic).unwrap();
        let loaded: AIProvider = serde_json::from_str(&json).unwrap();
        assert_eq!(loaded, AIProvider::Anthropic);

        let json = serde_json::to_string(&AIProvider::Gemini).unwrap();
        let loaded: AIProvider = serde_json::from_str(&json).unwrap();
        assert_eq!(loaded, AIProvider::Gemini);
    }
}
