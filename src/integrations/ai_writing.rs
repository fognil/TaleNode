use serde::{Deserialize, Serialize};

/// Supported AI providers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum AIProvider {
    #[default]
    OpenAI,
    Anthropic,
}

impl AIProvider {
    pub fn label(self) -> &'static str {
        match self {
            Self::OpenAI => "OpenAI-compatible",
            Self::Anthropic => "Anthropic Claude",
        }
    }

    pub const ALL: [AIProvider; 2] = [AIProvider::OpenAI, AIProvider::Anthropic];
}

pub fn default_ai_base_url() -> String {
    "https://api.openai.com/v1".to_string()
}

pub fn default_ai_model() -> String {
    "gpt-4o".to_string()
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
    }

    #[test]
    fn default_urls_and_model() {
        assert_eq!(default_ai_base_url(), "https://api.openai.com/v1");
        assert_eq!(default_ai_model(), "gpt-4o");
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
    }
}
