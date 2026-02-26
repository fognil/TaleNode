# Auto-Translation

TaleNode integrates with the DeepL API to automatically translate your dialogue into any supported language. Auto-translation works alongside the existing [Localization](localization.md) system — it fills in untranslated strings for a locale in one click.

## Setup

1. Get a DeepL API key from [deepl.com/pro-api](https://www.deepl.com/pro-api)
2. Open **Settings > Settings...**
3. Paste your API key in the **DeepL Translation** section
4. If you have a Pro plan, check **Use DeepL Pro API**
5. Click **Save Settings**

## Usage

### Translating a Locale

1. Open the Locale panel via **View > Localization**
2. Add at least one extra locale (e.g., `fr`, `ja`, `de`)
3. Select the locale you want to translate
4. Click **Auto-Translate**

TaleNode collects all untranslated strings for the selected locale, sends them to DeepL in batches, and fills in the translations automatically.

### What Gets Translated

Auto-translation targets all translatable strings that are currently empty for the selected locale:

| String Type | Source |
|---|---|
| Dialogue text | `DialogueData.text` field |
| Choice prompts | `ChoiceData.prompt` field |
| Choice options | Each option's `text` field |

Strings that already have a translation are skipped — auto-translate only fills in blanks.

### Batch Processing

DeepL has a limit of 50 texts per API call. TaleNode automatically batches larger projects:

- 120 untranslated strings = 3 API calls (50 + 50 + 20)
- Progress is applied all at once when complete

## Supported Languages

DeepL supports 30+ target languages including:

`BG`, `CS`, `DA`, `DE`, `EL`, `EN`, `ES`, `ET`, `FI`, `FR`, `HU`, `ID`, `IT`, `JA`, `KO`, `LT`, `LV`, `NB`, `NL`, `PL`, `PT`, `RO`, `RU`, `SK`, `SL`, `SV`, `TR`, `UK`, `ZH`

Use the standard locale codes when adding locales in TaleNode — they are automatically mapped to DeepL's target language codes.

## Workflow

A typical translation workflow with auto-translate:

1. Write all dialogue in your default locale (e.g., English)
2. Add target locales in the Locale panel (`fr`, `ja`, etc.)
3. Click **Auto-Translate** for each locale to get machine translations
4. Review and edit translations in the Locale panel or Inspector
5. Export CSV for professional translators to polish the machine translations
6. Import the polished CSV back into TaleNode

!!! tip
    Use auto-translate as a first pass, then send the CSV export to professional translators for review. Machine translation is a great starting point but may miss nuance, idioms, or game-specific terminology.

!!! tip
    Auto-translate creates an undo snapshot before applying translations. If the results aren't satisfactory, use **Edit > Undo** to revert all translations at once.

## Error Handling

| Error | Cause |
|---|---|
| "No DeepL API key configured" | API key not set in Settings |
| "No locale selected" | No active locale chosen in the Locale panel |
| "DeepL API error 403" | Invalid API key or quota exceeded |
| "DeepL API error 456" | Free plan character limit reached for the month |

Check the status bar at the bottom of the window for error messages after an auto-translate attempt.
