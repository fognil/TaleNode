# Script Editor

The Script Editor provides a text-based view of your dialogue graph using Yarn-style syntax. Edit dialogue as text and sync changes back to the visual node graph — or use both views side by side.

## Opening the Script Editor

- **Menu**: View > Script Editor (checkbox toggle)

The Script Editor opens as a right-side panel alongside the canvas. Both views show the same dialogue — edits in one are reflected in the other.

## Yarn-Style Syntax

The script view uses a syntax inspired by [Yarn Spinner](https://yarnspinner.dev/):

```yarn
title: Start
---
<<start>>
-> Elder
===

title: Elder
---
Elder: Welcome, traveler. What brings you here?
-> Choice: Ask about quest
    -> QuestInfo
-> Choice: Ask about village
    -> VillageInfo
===
```

Each node is represented as a Yarn-style passage with a title, body, and delimiters (`---` and `===`).

## Sync Status

The toolbar shows the current sync state:

| Status | Color | Meaning |
|---|---|---|
| **Synced** | Green | Script and graph are in sync |
| **Modified** | Yellow | You've edited the script text but haven't committed |
| **Graph changed** | Blue | The graph was changed on the canvas — click Refresh |
| **Modified + Graph changed** | Orange | Both sides have unsaved changes |

## Toolbar Buttons

| Button | Action |
|---|---|
| **Commit** | Parse the script text and apply changes to the graph (appears when modified) |
| **Discard** | Discard script edits and revert to the graph state (appears when modified) |
| **Refresh** | Regenerate the script text from the current graph |

## Workflow

### Graph-first editing

1. Build your dialogue visually on the canvas
2. Open the Script Editor to review the text flow
3. Make quick text edits (fix typos, adjust wording) directly in the script
4. Click **Commit** to apply changes back to the graph

### Script-first editing

1. Open the Script Editor
2. Write or paste dialogue text in Yarn syntax
3. Click **Commit** to generate nodes and connections from the script
4. Fine-tune positions and connections on the canvas

## Tips

!!! tip
    The Script Editor is great for proofreading — it shows all dialogue text in a linear format that's easier to read than scanning nodes on the canvas.

!!! tip
    When both views show changes (orange status), decide which version to keep: **Commit** to apply script edits, or **Refresh** to discard them and reload from the graph.
