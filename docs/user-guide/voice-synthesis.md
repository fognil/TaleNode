# Voice Synthesis

TaleNode integrates with ElevenLabs to generate AI voice audio for your dialogue nodes. Assign voices to characters, then generate speech audio files that your game engine can use at runtime.

## Setup

1. Get an ElevenLabs API key from [elevenlabs.io](https://elevenlabs.io)
2. Open **Settings > Settings...**
3. Paste your API key in the **ElevenLabs Voice** section
4. Click **Save Settings**

## Assigning Voices to Characters

Before generating audio, assign a voice to each character:

1. Open the **Characters** section in the left panel
2. Select a character to expand its editor
3. Use the **Voice** dropdown to pick from your ElevenLabs voice library
4. If the dropdown is empty, click **Fetch Voices** in the Voice Generation panel first

The dropdown shows all voices available in your ElevenLabs account, including default and custom cloned voices.

## Voice Generation Panel

Open via **View > Voice Generation** (or drag the tab from the dock).

### Panel Layout

The panel shows a table of all Dialogue nodes in your graph:

| Column | Description |
|---|---|
| **Speaker** | Character name assigned to the dialogue node |
| **Text** | Preview of the dialogue text (truncated to 60 characters) |
| **Voice** | Assigned voice name, or "(no voice)" if the speaker's character has no voice set |
| **Audio** | Status indicator — shows the audio file path or "(none)" |
| **Action** | **Generate** button for individual node generation |

### Toolbar Buttons

| Button | Description |
|---|---|
| **Fetch Voices** | Loads the list of available voices from your ElevenLabs account |
| **Generate All** | Generates audio for all Dialogue nodes that have an assigned voice but no audio file yet |

## Generating Audio

### Single Node

Click the **Generate** button next to any dialogue node in the Voice Generation panel. The audio file is saved and the node's `audio_clip` field is set automatically.

### Batch Generation

Click **Generate All** to process all dialogue nodes that:

- Have a speaker with an assigned voice
- Don't already have an audio file

Nodes are processed sequentially. Progress is shown in the status bar.

## Audio Files

Generated audio files are saved to:

```
{project_directory}/voices/{readable_id}.mp3
```

For example, if your project is at `/documents/my_game.talenode`, audio files go to `/documents/voices/dlg_1.mp3`, `/documents/voices/dlg_2.mp3`, etc.

The `audio_clip` field on each Dialogue node is set to the relative path (e.g., `voices/dlg_1.mp3`) so your game engine can locate the files relative to the export.

## Voice Model

TaleNode uses the `eleven_multilingual_v2` model for generation, which supports multiple languages. This means voice generation works with translated dialogue text as well.

## Tips

!!! tip
    Fetch voices once after entering your API key. The voice list is cached for the current session — you don't need to fetch again unless you add new voices to your ElevenLabs account.

!!! tip
    Generate audio after your dialogue text is finalized. If you change the text, you'll need to regenerate the audio for those nodes.

!!! tip
    ElevenLabs charges by character count. Use **Generate All** for batch processing, but be aware of your plan's character limits. Check your usage at [elevenlabs.io](https://elevenlabs.io).

## Error Handling

| Error | Cause |
|---|---|
| "No ElevenLabs API key configured" | API key not set in Settings |
| "Character has no voice_id" | The speaker's character doesn't have a voice assigned |
| "ElevenLabs API error 401" | Invalid API key |
| "ElevenLabs API error 429" | Rate limit or quota exceeded |

Check the status bar at the bottom of the window for error messages after generation attempts.
