# FAQ

## General

### What is TaleNode?

TaleNode is a desktop application for creating branching dialogues, storylines, and quest logic using a visual node graph. It exports clean JSON files that can be loaded by any game engine.

### What game engines does TaleNode support?

TaleNode is engine-agnostic. It exports a standard JSON format that works with Unity, Godot, Unreal, or any engine that can parse JSON. See the [Integration Guide](../export/integration-guide.md) for examples.

### Is TaleNode free?

TaleNode is open-source. Check the repository for license details.

### What platforms does TaleNode run on?

Windows 10+, macOS 11+, and Linux (X11 or Wayland). See [Installation](../getting-started/installation.md).

## Editor

### How do I pan the canvas?

Middle mouse drag, Space + left mouse drag, or scroll wheel. See [Canvas & Navigation](../user-guide/canvas.md).

### How do I zoom?

++ctrl++ + scroll wheel, or trackpad pinch on macOS. Zoom range is 25% to 400%.

### Can I undo mistakes?

Yes. Press ++ctrl+z++ to undo and ++ctrl+shift+z++ to redo. TaleNode keeps up to 100 undo steps.

### How do I delete a connection/wire?

There's no direct "delete wire" action. You can:

- Delete one of the connected nodes (the wire is removed automatically)
- Create a new connection to the same port (replaces the old wire)
- Undo the connection (++ctrl+z++)

### Can I copy/paste nodes between projects?

Not directly. You can duplicate nodes within the same project with ++ctrl+d++.

### How do I search for a node?

Press ++ctrl+f++ to open the search bar. It searches across all node types, dialogue text, speaker names, and more.

### Can I find and replace text across nodes?

Yes. Press ++ctrl+h++ (++cmd+shift+h++ on macOS) to open Search & Replace. You can replace in the current match or all matches at once. Replacements are case-insensitive and support undo.

## Nodes

### Can I create custom node types?

No. TaleNode supports 8 fixed node types: Start, Dialogue, Choice, Condition, Event, Random, End, and SubGraph. These cover the standard patterns for game dialogue. Use SubGraph nodes to organize complex conversations into reusable nested dialogues.

### What's the difference between .talenode and exported .json?

| Aspect | .talenode | Export .json |
|---|---|---|
| Purpose | Editor project file | Game engine data |
| Positions | Included | Not included |
| IDs | UUIDs | Human-readable (`dlg_1`) |
| Connections | Separate list | Baked into `next` fields |
| Groups | Included | Not included |

### Can a node connect to itself?

No. Self-loops are not allowed.

### Can one output connect to multiple inputs?

No. Each output port can have exactly one outgoing connection. Use Choice or Random nodes to create branches.

## Variables

### What variable types are supported?

Bool, Int, Float, and Text. See [Variables](../user-guide/variables.md).

### How do conditions work at runtime?

Your game engine evaluates Condition nodes by comparing the named variable against the specified value using the given operator. TaleNode's playtest mode evaluates conditions against runtime variables, taking the correct True or False branch.

### Can I use variables in dialogue text?

Yes. Use `{variable_name}` syntax in dialogue or choice text for inline substitution. You can also use math expressions (`{100 - gold}`), comparisons (`{gold >= 50}`), and inline conditionals (`{if has_key}...{else}...{/if}`). See [Variables — Text Interpolation](../user-guide/variables.md#text-interpolation).

### Are expressions evaluated in exported JSON?

No. The `{...}` syntax is preserved as-is in the exported JSON. Your game engine is responsible for evaluating expressions at runtime. TaleNode only evaluates them during playtest preview.

## Export

### Are node positions included in the export?

No. The exported JSON only contains dialogue data — no editor layout information.

### What does the exported JSON look like?

See [JSON Export Format](../export/json-format.md) for the full specification with examples.

### Can I export to formats other than JSON?

Yes. TaleNode supports multiple export formats:

- **JSON** — Standard game engine data format. See [JSON Export Format](../export/json-format.md).
- **XML** — Alternative structured format.
- **Voice Script (CSV)** — CSV with speaker, text, emotion, and audio clip fields for voice actors.
- **Runtime Plugins** — Drop-in plugins for Godot, Unity, and Unreal Engine. See [Plugin Export](../export/plugin-export.md).

## Troubleshooting

### The app is slow with many nodes

Use `cargo run --release` (or the release build) for better performance. The debug build has significantly lower rendering performance.

### My nodes disappeared

You may have panned far from your nodes. Press ++f++ to zoom-to-fit and show all nodes, or use the minimap (bottom-right corner) to navigate. You can also press ++ctrl+a++ to select all — the status bar will show the node count.

### Auto-save isn't working

Auto-save only activates after you've saved the project at least once (so it has a file path). Use ++ctrl+s++ to do an initial save.
