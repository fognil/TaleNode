# Unity Editor Tools

The TaleNode Unity package includes editor tools that let you view dialogue graphs, inspect assets, and playtest dialogues directly inside Unity. All editing is done in the TaleNode desktop app — the Unity tools are read-only.

## Asset Import

Name your exported JSON files with the `.talenode.json` compound extension (e.g. `intro.talenode.json`). When you drop the file into your Unity `Assets/` folder, the ScriptedImporter automatically creates a `TaleNodeDialogue` asset.

!!! tip "Regular .json files are not affected"
    The importer only matches the `.talenode.json` compound extension. Your other JSON files will continue to work as before.

## Custom Inspector

Select any `.talenode.json` asset in the Project window to see the custom inspector:

- **Header** — Dialogue name, version, total node count
- **Node Statistics** — Count per node type with color-coded dots matching the desktop app
- **Characters** — List of all characters in the dialogue
- **Variables** — Name, type, and default value for each variable
- **Locales** — Available locales with translation coverage percentage
- **Action buttons** — "Open in Graph View" and "Open in Playtest"

## Graph View

Open via **Window > TaleNode > Dialogue Graph** or the inspector's "Open in Graph View" button.

### Toolbar

| Control | Function |
|---|---|
| **Dialogue** field | Pick which `.talenode.json` asset to display |
| **Locale** dropdown | Switch displayed text to a different locale |
| **Frame All** button | Zoom to fit all nodes in view |
| **Search** field | Filter nodes by name or ID (non-matching nodes fade out) |

### Node Display

Each node type has a distinct color:

| Type | Color |
|---|---|
| Start | Green |
| Dialogue | Blue |
| Choice | Yellow |
| Condition | Orange |
| Event | Purple |
| Random | Gray |
| End | Red |

Nodes show a preview of their content:

- **Dialogue** — Speaker name, emotion tag, and text preview
- **Choice** — Prompt text; each option is a separate output port
- **Condition** — Variable, operator, and comparison value
- **Event** — List of actions (up to 3, then "...")
- **End** — Tag name

### Auto-Layout

Since exported JSON does not include node positions, the graph uses BFS auto-layout:

- **Start nodes** are placed at the far left (column 0)
- **Each hop** from start adds a column to the right
- **Condition nodes** fan true/false branches vertically
- **Choice/Random nodes** stack their options vertically
- **Orphaned nodes** (not reachable from start) are placed at the far right

Nodes are draggable for viewing convenience, but positions are not saved.

### Read-Only

The graph is view-only:

- You cannot create new connections or nodes
- You cannot delete existing connections or nodes
- Positions are not persisted

To edit the dialogue, use the TaleNode desktop app and re-export.

## Playtest Panel

Open via **Window > TaleNode > Playtest** or the inspector's "Open in Playtest" button.

### Toolbar

| Control | Function |
|---|---|
| **Dialogue** field | Pick which asset to playtest |
| **Locale** dropdown | Switch dialogue text to a different locale |
| **Play** | Start the dialogue from the beginning |
| **Stop** | Stop the current dialogue |
| **Restart** | Stop and restart from the beginning |

### Main Area

The left pane shows a scrollable dialogue log:

- **Dialogue lines** appear as "Speaker: Text" with the speaker name in blue
- **Choice prompts** appear in yellow
- **Your selections** appear in green as "You: chosen option"
- **Events** appear in purple with action details
- **End markers** appear in red

When a dialogue line is reached, click **Continue >>>** to advance. When choices appear, click the option buttons to select.

### Variable Watch

The right pane shows all variables with their current values. When a variable changes, its value highlights in yellow so you can track state changes in real time.

### Graph Sync

If the Graph View window is open, the playtest panel highlights the currently active node with a yellow border and scrolls to it automatically. This lets you follow the dialogue flow visually as you playtest.

## Localization Preview

Both the Graph View and Playtest Panel support locale switching:

1. Your dialogue must have locales defined (exported with `locales` and `strings` fields)
2. Select a locale from the toolbar dropdown
3. Node text updates to show the translated version
4. Untranslated strings fall back to the default text
