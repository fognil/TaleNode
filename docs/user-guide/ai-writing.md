# AI Writing Assistant

TaleNode integrates with OpenAI, Anthropic Claude, and Google Gemini to help you write and refine dialogue. Get alternative dialogue suggestions, generate choice options, and check tone consistency — all from within the editor.

## Setup

1. Get an API key from your chosen provider:
    - **OpenAI**: [platform.openai.com/api-keys](https://platform.openai.com/api-keys)
    - **Anthropic**: [console.anthropic.com](https://console.anthropic.com)
    - **Google Gemini**: [aistudio.google.com/apikey](https://aistudio.google.com/apikey)
2. Open **Settings > Settings...**
3. In the **AI Writing Assistant** section:
    - Select your **Provider** from the dropdown
    - Paste your **API Key**
    - (Optional) Click **Fetch Models** to load available models, then pick one from the dropdown
4. Click **Save Settings**

## Providers

| Provider | Base URL | Default Model | Notes |
|---|---|---|---|
| **OpenAI-compatible** | `https://api.openai.com/v1` | `gpt-4o` | Also works with local LLM servers (LM Studio, Ollama) by changing the base URL |
| **Anthropic Claude** | *(not configurable)* | `claude-sonnet-4-6` | Uses Anthropic's native API with `x-api-key` auth |
| **Google Gemini** | `https://generativelanguage.googleapis.com/v1beta` | `gemini-2.0-flash` | API key passed as query parameter |

When you switch providers, the base URL and model reset to that provider's defaults automatically.

## Fetching Models

Click **Fetch Models** in settings to load available models from your provider's API:

- **OpenAI**: Fetches from `/models` endpoint, filtered to chat models (GPT, o1, o3, o4 families)
- **Gemini**: Fetches from `/models` endpoint, filtered to text-generation models (excludes nano, embedding, and dated variants)
- **Anthropic**: Loads a built-in list (Anthropic has no public model listing endpoint)

The Fetch Models button is enabled only when an API key is set. After fetching, the model text field becomes a dropdown for easy selection.

## Writing Panel

Open via **View > AI Writing** (or drag the tab from the dock).

### Features

#### Dialogue Suggestions

Select a Dialogue node, then click **Suggest Alternatives**. The AI generates 3 alternative lines that keep the same meaning but vary in tone or phrasing.

You can add an optional **instruction** (e.g., "make it more formal", "shorter sentences") to guide the suggestions.

Click any suggestion to apply it to the node.

#### Choice Generation

Select a Choice node, then click **Generate Choices**. Set the desired number of options (default: 3). The AI generates distinct player response options based on the choice prompt and surrounding context.

Generated options are added to the Choice node automatically.

#### Tone Check

Select a Dialogue node, then click **Check Tone**. The AI analyzes the line against the speaker's emotion tag and provides a brief assessment:

- Does the line match the emotion?
- Any grammar issues?
- Suggestions for improvement

The result appears in the panel as a text report.

## Tips

!!! tip
    Use a fast, cheap model (like `gpt-4o-mini` or `gemini-2.0-flash`) for iterative dialogue writing. Switch to a larger model for final tone checks.

!!! tip
    The **OpenAI-compatible** provider works with any API that implements the OpenAI chat completions format. Point the base URL at a local LLM server to use AI writing offline.

!!! tip
    API providers charge per token. Dialogue suggestions and tone checks use relatively few tokens per request. Choice generation scales with the number of options requested.

## Error Handling

| Error | Cause |
|---|---|
| "Set an API key first" | No API key configured in Settings |
| "AI API error 401" | Invalid API key |
| "AI API error 429" | Rate limit or quota exceeded |
| "Unexpected API response format" | Provider returned an unexpected response structure |
| "AI request failed" | Network error or provider unreachable |

Check the status bar at the bottom of the window for error messages.
