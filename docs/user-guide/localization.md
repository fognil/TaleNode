# Localization

TaleNode has built-in support for managing translations of your dialogue across multiple languages. Define locales, edit translations per-node or in bulk, and export CSV files for professional translators.

## Concepts

### Default Locale

Your dialogue text (in Dialogue nodes, Choice prompts, Choice options) is written in the **default locale** (e.g., English). This text lives directly in the node fields and is always editable on the canvas and in the Inspector.

### Extra Locales

Additional languages (e.g., `fr`, `ja`, `de`) are stored separately in a translation table. Each translatable string is identified by a **string key** derived from the node's UUID.

### String Keys

| Source | Key Pattern | Example |
|---|---|---|
| Dialogue text | `dlg_{uuid8}` | `dlg_7de3cb62` |
| Choice prompt | `choice_{uuid8}` | `choice_4968a99b` |
| Choice option | `opt_{uuid8}_{index}` | `opt_4968a99b_0` |

Keys are stable — they never change once a node is created, even if you edit the text.

## Locale Panel

Open via **View > Localization** (or drag the tab from the dock).

### Toolbar

- **Add Locale**: Type a locale code (e.g., `fr`, `ja`, `de`, `pt-BR`) and click **Add** to create a new locale
- **Remove Locale**: Click the **X** button next to a locale name to remove it and all its translations
- **Export CSV**: Save all translatable strings and translations to a CSV file for translators
- **Import CSV**: Load translations from a CSV file back into the project
- **Filter**: Toggle between **All** strings and **Untranslated only** to focus on missing translations

### Progress Bars

Each locale shows a progress bar with the count and percentage of translated strings:

```
fr: 45/120 (37%)  [████████░░░░░░░░░░░░]
ja: 120/120 (100%) [████████████████████]
```

### Translation Table

The table shows all translatable strings in your graph:

| Column | Description |
|---|---|
| **Key** | The string key (clickable — navigates to the node on canvas) |
| **Type** | `dialogue`, `prompt`, or `option` |
| **Default** | The default locale text (read-only in this view) |
| **[locale]** | One editable column per extra locale |

Edit translations directly in the table cells. Empty cells show as "(untranslated)".

## Inspector Locale Switcher

When extra locales exist, the Inspector shows a **Locale** dropdown below the node ID:

- **Default (en)**: Normal editing mode — edit the node's text fields directly
- **fr / ja / etc.**: Translation mode — each translatable text field shows an additional translation input below the default text

In translation mode, the default text is still editable. Below each translatable field, a labeled input (e.g., `[fr]`) lets you type the translation. Untranslated fields show a dim "(untranslated)" placeholder.

## CSV Workflow

The CSV format is designed for professional translation workflows:

### Export

1. Open the Locale panel (**View > Localization**)
2. Click **Export CSV**
3. Choose a save location

The CSV file contains:

```csv
key,type,en,fr,ja
dlg_7de3cb62,dialogue,"Hello, traveler!","Bonjour, voyageur!",""
choice_4968a99b,prompt,"What will you do?","Que voulez-vous faire?",""
opt_4968a99b_0,option,"Fight","Combattre",""
```

- First row: headers with locale codes
- `key` and `type` columns identify each string
- One column per locale (default + all extras)
- Empty strings indicate untranslated entries

### Import

1. Open the Locale panel
2. Click **Import CSV**
3. Select the CSV file

TaleNode matches rows by `key` and updates translations for all locales found in the CSV headers. New keys in the CSV that don't match existing nodes are ignored.

!!! tip
    Send the exported CSV to your translators. They fill in the empty cells and return the file. Import it to apply all translations at once.

## JSON Export

When your project has extra locales, the exported JSON includes a string table:

```json
{
  "version": "1.0",
  "default_locale": "en",
  "locales": ["en", "fr", "ja"],
  "strings": {
    "dlg_1": { "en": "Hello!", "fr": "Bonjour!", "ja": "こんにちは！" },
    "choice_1": { "en": "What next?", "fr": "Et maintenant?", "ja": "次は？" }
  },
  "nodes": [...]
}
```

- `default_locale`, `locales`, and `strings` are only present when extra locales exist
- String keys in export use readable IDs (`dlg_1`) instead of UUID-based keys
- The `text` fields in nodes still contain the default locale text

See [JSON Export Format](../export/json-format.md) for the full specification.

## Tips

!!! tip
    Use the **Untranslated only** filter in the Locale panel to quickly find strings that still need translation.

!!! tip
    The Inspector locale switcher and the Locale panel both edit the same translation data. Use whichever workflow suits you — per-node in the Inspector or in bulk in the panel.

!!! tip
    Translations are saved in your `.talenode` project file and persist across sessions. No separate translation files to manage.
