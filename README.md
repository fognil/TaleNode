# TaleNode

Node-based dialogue and quest editor for game developers.

Build branching conversations visually. Export JSON for Unity, Godot, Unreal, or any engine.

## Features

- **Visual Node Editor** — infinite canvas with pan, zoom, grid snapping, minimap
- **8 Node Types** — Start, Dialogue, Choice, Condition, Event, Random, End, SubGraph
- **Variables & Conditions** — Bool, Int, Float, Text variables for branching logic
- **Character Database** — names, colors, portrait paths referenced in Dialogue nodes
- **Localization** — multiple locales, per-node translation editing, CSV export/import for translators
- **Multi-Format Export** — JSON, XML, Voice CSV; runtime plugins for Godot, Unity, Unreal
- **Multi-Format Import** — Yarn Spinner, Chat Mapper, articy:draft
- **Script Editor** — visual node graph + Yarn script side by side with bidirectional sync
- **Playtest Mode** — walk through dialogue in a built-in preview
- **Live Validation** — real-time checks for disconnected nodes, missing Start, dead ends
- **Review & Collaboration** — comments per node, review status tracking, version history with diff
- **Templates & Analytics** — reusable node patterns, path stats, word counts, dead-path detection
- **Undo/Redo** — snapshot-based full graph undo

## Node Types

| Node | Purpose |
|---|---|
| **Start** | Entry point of a conversation |
| **Dialogue** | NPC speaks a line (speaker, text, emotion, audio) |
| **Choice** | Player picks from N options |
| **Condition** | Branch based on variable comparison |
| **Event** | Trigger game actions (set variable, play sound, etc.) |
| **Random** | Random branch selection with weights |
| **End** | Terminates conversation with optional tag |
| **SubGraph** | Contains a nested dialogue graph |

## Quick Start

### Requirements

| | Minimum |
|---|---|
| OS | Windows 10+, macOS 11+, Linux (X11/Wayland) |
| GPU | OpenGL 3.3 or Vulkan |
| Rust | Stable toolchain |

### Build from Source

```bash
git clone https://github.com/fognil/TaleNode.git
cd TaleNode
cargo build --release
./target/release/talenode
```

Linux extra packages (Debian/Ubuntu):

```bash
sudo apt install libxcb-render0-dev libxcb-shape0-dev libxcb-xfixes0-dev libxkbcommon-dev
```

### Run in Debug Mode

```bash
cargo run
```

## File Formats

### .talenode (Project)

Full editor state: nodes, connections, positions, variables, characters, groups, translations. JSON format, backward compatible.

### Export JSON (Game Engine)

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
      { "text": "Friend", "next": "end_1" },
      { "text": "Enemy", "next": "end_2" }
    ]},
    { "id": "end_1", "type": "end", "tag": "friendly" },
    { "id": "end_2", "type": "end", "tag": "hostile" }
  ]
}
```

- Flat node array with baked `next` fields — no separate connections array
- Human-readable IDs (`dlg_1`, `choice_2`) instead of UUIDs
- No editor positions — clean data for game runtime
- Optional `strings` table when localization is configured

See [JSON Export Format](docs/export/json-format.md) for the full specification.

## Engine Plugins

Drop-in runtime plugins are included for loading TaleNode JSON in your game:

| Engine | Language | Path |
|---|---|---|
| Godot | GDScript | `plugins/godot/` |
| Unity | C# | `plugins/unity/` |
| Unreal | C++ | `plugins/unreal/` |

Export via **File > Export [Engine] Plugin...** to copy the plugin directly into your project.

## Tech Stack

| Component | Crate |
|---|---|
| UI framework | eframe + egui 0.31 |
| Serialization | serde + serde_json |
| IDs | uuid v4 |
| File dialogs | rfd |
| XML parsing | roxmltree |
| Dock panels | egui_dock |

## Documentation

Full documentation: [docs/](docs/)

| Page | Description |
|---|---|
| [Installation](docs/getting-started/installation.md) | Download or build from source |
| [Quick Start](docs/getting-started/quickstart.md) | Create your first dialogue in 5 minutes |
| [Canvas & Navigation](docs/user-guide/canvas.md) | Pan, zoom, select, connect nodes |
| [Inspector Panel](docs/user-guide/inspector.md) | Edit node properties |
| [Localization](docs/user-guide/localization.md) | Multi-language translation workflow |
| [JSON Export Format](docs/export/json-format.md) | Full export schema |
| [Keyboard Shortcuts](docs/reference/keyboard-shortcuts.md) | Complete shortcut table |

## Development

```bash
cargo check          # Compile check
cargo clippy         # Lint
cargo test           # Run tests (232 tests)
cargo run --release  # Run with optimizations
```

## License

MIT
