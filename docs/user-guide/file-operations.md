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

Clears the current graph and starts fresh. If you have unsaved changes, you'll be working from a blank slate — save first if needed.

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

## Export JSON

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

## Tips

!!! tip
    Save your project (++ctrl+s++) regularly. Auto-save is a safety net, not a replacement for manual saves.

!!! tip
    Keep your `.talenode` project files in version control (Git). They're plain JSON and diff well.

!!! tip
    Export JSON only when you're ready to integrate with your game. The `.talenode` file is your working format.
