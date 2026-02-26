# File Operations

TaleNode uses two file formats: `.talenode` for project files (full editor state) and `.json` for game engine export.

## Project Files (.talenode)

### Save

- **Shortcut**: ++ctrl+s++
- **Menu**: File > Save

If the project hasn't been saved before, this opens a "Save As" dialog. Otherwise, it overwrites the existing file.

### Save As

- **Menu**: File > Save As...

Opens a file dialog to choose a new save location.

### Open

- **Shortcut**: ++ctrl+o++
- **Menu**: File > Open...

Opens a file dialog to load a `.talenode` project file.

### New Project

- **Shortcut**: ++ctrl+n++
- **Menu**: File > New

Clears the current graph and starts fresh. A **confirmation dialog** will appear to prevent accidental data loss — click **Yes** to proceed or **No** to cancel.

### Auto-Save

TaleNode automatically saves your project every **60 seconds** if:

1. The project has been saved at least once (a file path exists)
2. Changes have been made since the last save

A brief "Auto-saved" message appears in the status bar when auto-save triggers.

## What's in a .talenode File?

The `.talenode` format is pretty-printed JSON containing:

```json
{
  "version": "1.0.0",
  "name": "My Dialogue",
  "graph": {
    "nodes": [...],
    "connections": [...],
    "variables": [...],
    "characters": [...],
    "groups": [...]
  }
}
```

This includes everything: node positions, port data, all connections, variables, characters, groups, and metadata. The file is fully self-contained.

!!! note
    `.talenode` files are backward compatible — new fields use `#[serde(default)]` so older files open in newer versions without issues.

## Export

### Export JSON

- **Menu**: File > Export JSON...

Opens a file dialog to save a `.json` file for your game engine. See [JSON Export Format](../export/json-format.md) for the full specification.

Key differences from the project file:

| Aspect | .talenode | Export .json |
|---|---|---|
| Node positions | Included | Not included |
| Connections | Separate list | Baked into `next` fields |
| Node IDs | UUIDs | Human-readable (`dlg_1`, `choice_2`) |
| Groups | Included | Not included |
| Port data | Included | Not included |

### Export XML

- **Menu**: File > Export XML...

Exports the dialogue graph in XML format.

### Runtime Plugin Export

- **Menu**: File > Export Godot Plugin...
- **Menu**: File > Export Unity Plugin...
- **Menu**: File > Export Unreal Plugin...

Exports a drop-in runtime plugin to the selected game engine project folder. See [Plugin Export](../export/plugin-export.md) for details.

### Export Voice Script (CSV)

- **Menu**: File > Export Voice Script (CSV)...

Exports a CSV file listing all dialogue lines with speaker, text, emotion, and audio clip fields — ready for voice actors.

### Export Markdown

- **Menu**: File > Export Markdown (.md)...

Exports the dialogue as a readable Markdown document with a characters table, variables table, quest checklists, and the full dialogue flow rendered via BFS traversal. Opens natively in any Markdown viewer, VS Code, or GitHub.

### Export Word (RTF)

- **Menu**: File > Export Word (.rtf)...

Exports the dialogue as an RTF file that opens natively in Microsoft Word, Google Docs, LibreOffice, and other word processors. Contains the same content as the Markdown export in formatted rich text.

### Batch Assign Audio

- **Menu**: File > Batch Assign Audio...

Opens the Batch Audio Assignment window for matching audio files to dialogue nodes in bulk.

1. Click **Select Folder** and choose a folder containing audio files (`.wav`, `.ogg`, `.mp3`)
2. TaleNode scans and matches files to dialogue nodes using two strategies:
    - **By readable ID**: e.g., `dlg_1.wav` matches the first dialogue node
    - **By speaker + index**: e.g., `elder_1.wav` matches the first dialogue line by "Elder"
3. Review the match table — matched files show in green, unmatched show "(none)"
4. Click **Apply Matches** to assign all matched audio paths to the dialogue nodes

Click **Re-scan** after adding files to the folder to refresh the match list.

### Export Locale CSV

- **Menu**: File > Export Locale CSV... (also available from the Localization panel)

Exports all translatable strings and their translations to a CSV file. The CSV includes columns for each defined locale, ready for professional translators. See [Localization](localization.md) for details.

### Import Locale CSV

- **Menu**: File > Import Locale CSV... (also available from the Localization panel)

Imports translations from a CSV file. Matches rows by string key and updates translations for all locales found in the CSV headers.

## Import

### Import from Yarn

- **Menu**: File > Import from Yarn...

Imports a Yarn Spinner `.yarn` file and converts it into a TaleNode dialogue graph.

### Import from Chat Mapper

- **Menu**: File > Import from Chat Mapper...

Imports a Chat Mapper JSON file.

### Import from articy

- **Menu**: File > Import from articy...

Imports an articy:draft XML export file.

### Import from Ink

- **Menu**: File > Import from Ink...

Imports an Inkle Ink `.ink` file. Knots and stitches are converted to nodes, choices become Choice nodes, diverts become connections, and `VAR` declarations become project variables.

!!! note
    Importing replaces the current graph. The operation supports undo — press ++ctrl+z++ to revert.

For detailed information on each import format, see [Import Formats](import.md).

## Tips

!!! tip
    Save your project (++ctrl+s++) regularly. Auto-save is a safety net, not a replacement for manual saves.

!!! tip
    Keep your `.talenode` project files in version control (Git). They're plain JSON and diff well.

!!! tip
    Export JSON only when you're ready to integrate with your game. The `.talenode` file is your working format.
