# Settings

TaleNode stores persistent settings for API integrations and collaboration preferences. Settings are saved to a configuration file and persist across sessions.

## Opening Settings

Open settings via **Settings > Settings...** in the menu bar.

## Configuration File

Settings are stored at:

| Platform | Path |
|---|---|
| **macOS** | `~/Library/Application Support/talenode/settings.json` |
| **Linux** | `~/.config/talenode/settings.json` |
| **Windows** | `%APPDATA%\talenode\settings.json` |

The file is created automatically when you first save settings.

## Settings Sections

### Appearance

| Field | Description |
|---|---|
| **Theme Preset** | Choose from **Dark**, **Light**, or **Custom**. Dark is the default |
| **Font Size** | Slider to adjust UI font size (range: 10–24, default: 14) |
| **Accent Color** | RGB color picker for the UI accent color (used for buttons, highlights, selections) |

You can also cycle through theme presets quickly via **View > Theme: ... ->** in the menu bar.

The Custom preset uses the Dark theme as a base but applies your chosen accent color. Light and Dark presets use their respective default accent colors.

### DeepL Translation

| Field | Description |
|---|---|
| **API Key** | Your DeepL API key (masked input). Required for auto-translation |
| **Use DeepL Pro API** | Checkbox — toggle between free and pro API endpoints |

Get a DeepL API key at [deepl.com/pro-api](https://www.deepl.com/pro-api). The free plan includes 500,000 characters/month.

### ElevenLabs Voice

| Field | Description |
|---|---|
| **API Key** | Your ElevenLabs API key (masked input). Required for voice synthesis |

Get an ElevenLabs API key at [elevenlabs.io](https://elevenlabs.io). The free plan includes limited characters/month.

### Collaboration

| Field | Description |
|---|---|
| **Username** | Your display name shown to other collaborators (defaults to your OS username) |
| **Default Port** | TCP port used when hosting a collaboration session (default: `9847`, range: 1024–65535) |

## Saving

Click **Save Settings** at the bottom of the settings window to write changes to disk. Settings are loaded automatically when TaleNode starts.

!!! tip
    API keys are stored in plain text in the settings file. Do not share your settings file publicly.

!!! tip
    If you change the DeepL plan from free to pro (or vice versa), toggle the **Use DeepL Pro API** checkbox accordingly — the free and pro endpoints are different.
