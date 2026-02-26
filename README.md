<p align="center">
  <h1 align="center">TaleNode</h1>
  <p align="center">
    <strong>Node-based dialogue & quest editor for game developers</strong><br>
    Build branching conversations visually. Export to Unity, Godot, Unreal, or any engine.
  </p>
  <p align="center">
    <a href="#installation">Installation</a> &middot;
    <a href="docs/getting-started/quickstart.md">Quick Start</a> &middot;
    <a href="https://talenode.readthedocs.io">Documentation</a> &middot;
    <a href="#engine-plugins">Engine Plugins</a>
  </p>
</p>

---

## Why TaleNode?

Most dialogue tools are either too simple (linear text editors) or locked inside a specific engine. TaleNode is a **standalone desktop app** that gives you a full visual node graph with branching logic, variables, conditions, localization, AI-assisted writing, voice synthesis, and real-time collaboration -- then exports a clean JSON file that works with **any** game engine.

---

## Features

### Core Editor

- **Visual Node Graph** -- Infinite canvas with pan, zoom, grid snapping, minimap, and box selection
- **8 Node Types** -- Start, Dialogue, Choice, Condition, Event, Random, End, SubGraph
- **Inspector Panel** -- Edit every node property: speaker, text, emotion, conditions, events, weights
- **Undo/Redo** -- Snapshot-based full graph undo with unlimited history
- **Find & Replace** -- Search across all nodes, tags, and variables with bulk replace
- **Live Validation** -- Real-time error checking: missing connections, dead ends, empty dialogues, unreachable nodes
- **Dockable Panels** -- 20 draggable/closable panels with persistent layout

### Branching & Logic

- **Variables** -- Bool, Int, Float, Text variables for game state tracking
- **Condition Nodes** -- Branch on variable comparisons (==, !=, >, <, >=, <=, contains)
- **Event Nodes** -- Trigger actions: SetVariable, AddItem, RemoveItem, PlaySound, ModifyRelationship, StartQuest, CompleteObjective, FailQuest, Custom
- **Random Nodes** -- Weighted random branch selection
- **SubGraphs** -- Nested dialogue graphs with breadcrumb navigation
- **Scripting Engine** -- Expression parser with `{variable}` interpolation, `{if cond}...{else}...{/if}` conditionals, and `{100 - gold}` math

### Characters & World

- **Character Database** -- Names, colors, portraits, voice assignments
- **NPC Relationships** -- Named relationship tracks (Friendship, Fear, Trust) with min/max ranges, modifiable via Event nodes
- **World Database** -- Catalog items, locations, lore entries with categories, tags, and key-value properties
- **Quest System** -- Quests with objectives, status tracking (NotStarted/InProgress/Completed/Failed), triggered from Event nodes
- **Bark Dialogue** -- Per-character ambient lines with weights and conditions, exportable as CSV

### Playtest & Review

- **Playtest Mode** -- Walk through your dialogue with live variable tracking, string interpolation, and automatic SubGraph traversal
- **Checkpoints** -- Save/load playtest state (up to 20 checkpoints)
- **Comments & Review** -- Per-node comments with Draft/Needs Review/Approved workflow
- **Bookmarks & Tags** -- Tag nodes for quick navigation and filtering
- **Version History** -- Snapshot versions with side-by-side diff comparison
- **Analytics** -- Path count, max depth, fan-out, dead ends, word counts

### Localization

- **Multi-Locale Support** -- Define locales, edit translations per-node or in bulk
- **DeepL Auto-Translation** -- One-click machine translation with batch processing
- **CSV Export/Import** -- Exchange translation tables with professional translators
- **String Table in Export** -- Optional localized string table included in JSON output

### AI & Voice

- **AI Writing Assistant** -- Dialogue suggestions, choice generation, and tone checking powered by OpenAI, Anthropic Claude, or Google Gemini
- **Model Fetcher** -- Browse available models from your provider's API directly in settings
- **Voice Synthesis** -- Generate speech audio for every dialogue node via ElevenLabs
- **Audio Manager** -- Batch-assign audio files to dialogue nodes by matching filenames

### Collaboration

- **Real-Time Co-Editing** -- Host a WebSocket session on your LAN and edit the same graph simultaneously
- **Live Sync** -- 11 operation types synced in real time: nodes, connections, variables, characters
- **Conflict Resolution** -- Last-Write-Wins strategy for concurrent edits

### Templates & Scripts

- **Template Library** -- Save and reuse node patterns across projects
- **Script Editor** -- Yarn Spinner text view side-by-side with the visual graph

---

## Node Types

| Node | Ports | Purpose |
|---|---|---|
| **Start** | 0 in, 1 out | Entry point of a conversation |
| **Dialogue** | 1 in, 1 out | NPC speaks a line (speaker, text, emotion, audio) |
| **Choice** | 1 in, N out | Player picks from N options, each with optional conditions |
| **Condition** | 1 in, 2 out | Branch on variable comparison (True/False) |
| **Event** | 1 in, 1 out | Trigger game actions (set variable, play sound, start quest, etc.) |
| **Random** | 1 in, N out | Weighted random branch selection |
| **End** | 1 in, 0 out | Terminates conversation with optional tag |
| **SubGraph** | 1 in, 1 out | Contains a nested dialogue graph |

---

## Installation

### Requirements

| | Minimum |
|---|---|
| **OS** | Windows 10+, macOS 11+, Linux (X11/Wayland) |
| **GPU** | OpenGL 3.3 or Vulkan |
| **Rust** | Stable toolchain (1.75+) |

### Build from Source

```bash
git clone https://github.com/fognil/TaleNode.git
cd TaleNode
cargo build --release
./target/release/talenode
```

Linux dependencies (Debian/Ubuntu):

```bash
sudo apt install libxcb-render0-dev libxcb-shape0-dev libxcb-xfixes0-dev libxkbcommon-dev
```

### Run in Development

```bash
cargo run
```

---

## Export Formats

TaleNode exports your dialogue in multiple formats:

| Format | Use Case |
|---|---|
| **JSON** | Primary game engine format -- flat nodes, baked connections, readable IDs |
| **XML** | Alternative structured format |
| **HTML Playable** | Self-contained playable dialogue in a browser |
| **Yarn** | Yarn Spinner compatible format |
| **Screenplay** | Formatted script for writers and voice actors |
| **Markdown / RTF** | Readable documents with character tables and quest checklists |
| **Voice CSV** | Voice recording spreadsheet (speaker, text, emotion, audio path) |
| **Bark CSV** | Ambient dialogue export per character |
| **Locale CSV** | Translation table for professional translators |
| **Analytics** | Graph analysis report |

### JSON Example

```json
{
  "version": "1.0",
  "name": "Guard Dialogue",
  "variables": [
    { "name": "reputation", "type": "Int", "default": 0 }
  ],
  "characters": [
    { "id": "char_1", "name": "Guard", "color": "#4A90D9" }
  ],
  "nodes": [
    { "id": "start_1", "type": "start", "next": "dlg_1" },
    { "id": "dlg_1", "type": "dialogue", "speaker": "char_1", "text": "Halt!", "emotion": "angry", "next": "choice_1" },
    { "id": "choice_1", "type": "choice", "prompt": "Respond:", "options": [
      { "text": "I'm a friend.", "next": "end_1" },
      { "text": "Out of my way.", "next": "end_2" }
    ]},
    { "id": "end_1", "type": "end", "tag": "friendly" },
    { "id": "end_2", "type": "end", "tag": "hostile" }
  ]
}
```

- Flat node array with baked `next` fields -- no separate connections array
- Human-readable IDs (`dlg_1`, `choice_2`) instead of raw UUIDs
- No editor positions -- clean runtime data only
- Optional `strings` table when localization is configured

See [JSON Export Format](https://talenode.readthedocs.io/export/json-format/) for the full specification.

---

## Import Formats

Migrate from existing tools:

| Format | Source |
|---|---|
| **Yarn Spinner** | `.yarn` files |
| **Chat Mapper** | Chat Mapper JSON export |
| **articy:draft** | articy JSON export |
| **Ink** | Inkle Ink format |

---

## Engine Plugins

Drop-in runtime plugins for loading TaleNode JSON in your game:

| Engine | Language | Status |
|---|---|---|
| **Unity** | C# | Available -- [`plugins/unity/`](plugins/unity/) |
| **Godot** | GDScript | Coming soon |
| **Unreal** | C++ | Coming soon |

Export via **File > Export Unity Plugin...** to copy the plugin directly into your Unity project.

---

## Keyboard Shortcuts

| Shortcut | Action |
|---|---|
| `Cmd/Ctrl+N` | New project |
| `Cmd/Ctrl+O` | Open project |
| `Cmd/Ctrl+S` | Save project |
| `Cmd/Ctrl+Z` | Undo |
| `Cmd/Ctrl+Shift+Z` | Redo |
| `Cmd/Ctrl+A` | Select all nodes |
| `Cmd/Ctrl+D` | Duplicate selected |
| `Cmd/Ctrl+F` | Find |
| `Cmd/Ctrl+H` | Find & Replace |
| `Delete` | Delete selected nodes |
| `F` | Zoom to fit |
| `Escape` | Close search / Exit SubGraph |
| `Middle drag` / `Space+drag` | Pan canvas |
| `Ctrl+Scroll` | Zoom |
| `Right click` | Context menu |

---

## Tech Stack

| Component | Crate |
|---|---|
| UI framework | eframe + egui 0.31 |
| Dock panels | egui_dock |
| Serialization | serde + serde_json |
| IDs | uuid v4 |
| File dialogs | rfd |
| HTTP client | reqwest |
| Async runtime | tokio |
| WebSocket | tokio-tungstenite |

---

## Development

```bash
cargo check          # Compile check
cargo clippy         # Lint (0 warnings required)
cargo test           # Run tests (360+ tests)
cargo run --release  # Run with optimizations
```

### Project Structure

```
src/
  app/           App state, canvas interaction, file I/O, settings, async handlers
  model/         Pure data structs (no UI imports)
  ui/            Rendering and input handling
  export/        JSON, XML, Yarn, HTML, Screenplay, CSV, Markdown, RTF, plugins
  import/        Yarn, Chat Mapper, articy, Ink parsers
  integrations/  DeepL, ElevenLabs, AI writing (OpenAI/Anthropic/Gemini)
  collab/        WebSocket real-time collaboration
  scripting/     Expression parser, evaluator, text interpolation
  validation/    Graph validator and analytics
  actions/       Undo/redo history
plugins/
  godot/         GDScript runtime
  unity/         C# runtime
  unreal/        C++ runtime
docs/            MkDocs documentation site
```

---

## Documentation

Full documentation at [talenode.readthedocs.io](https://talenode.readthedocs.io).

| Section | Topics |
|---|---|
| [Getting Started](https://talenode.readthedocs.io/getting-started/) | Installation, Quick Start |
| [User Guide](https://talenode.readthedocs.io/user-guide/) | Canvas, nodes, inspector, variables, playtest, localization, AI writing, voice, collaboration, quests, world database, timeline, and more |
| [Export Reference](https://talenode.readthedocs.io/export/) | JSON format, plugin export, Unity editor integration |
| [Reference](https://talenode.readthedocs.io/reference/) | Node reference, keyboard shortcuts, FAQ |

---

## License

MIT -- see [LICENSE](LICENSE) for details.

---

<p align="center">
  &copy; fognil
</p>
