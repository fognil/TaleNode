# Plugin Export

TaleNode can export drop-in runtime plugins for popular game engines. These plugins include a dialogue runner that loads and plays your exported JSON files.

## Supported Engines

| Engine | Menu Item | Output |
|---|---|---|
| **Godot** | File > Export Godot Plugin... | GDScript addon in `addons/talenode/` |
| **Unity** | File > Export Unity Plugin... | C# scripts in `TaleNode/` |
| **Unreal** | File > Export Unreal Plugin... | Plugin folder for UE |

## How It Works

1. Choose the export menu item for your engine
2. Select the target folder (your game project's root or plugin directory)
3. TaleNode writes the runtime plugin files to that location

The exported plugin includes:

- **Dialogue runner** — Loads and plays through dialogue JSON files
- **Expression evaluator** — Handles `{variable}` interpolation, math expressions, and inline conditionals
- **Signal/event system** — Emits events for your game to respond to (dialogue started, choice presented, event triggered, etc.)

## Godot Plugin

The Godot plugin exports to `addons/talenode/` and includes:

- `talenode_runner.gd` — Main dialogue runner script
- `talenode_expression.gd` — Expression parser and evaluator
- `plugin.cfg` — Godot plugin configuration

Enable the plugin in **Project > Project Settings > Plugins**.

## Unity Plugin

The Unity plugin is a full UPM (Unity Package Manager) package for Unity 6+. It includes both a runtime dialogue runner and an editor toolkit.

### Installation

=== "Git URL (recommended)"

    In Unity: **Window > Package Manager > + > Add package from git URL**:

    ```
    https://github.com/fognil/TaleNode.git?path=plugins/unity
    ```

=== "Export from TaleNode"

    Use **File > Export Unity Plugin...** and select your Unity project root. TaleNode writes the full package to the target directory.

=== "Local disk"

    **Window > Package Manager > + > Add package from disk...** and select `plugins/unity/package.json` from the TaleNode repository.

### What's Included

**Runtime** (works in builds):

- `TaleNodeRunner.cs` — Dialogue execution engine with events
- `TaleNodeExpression.cs` — Expression parser, evaluator, and text interpolation
- `TaleNodeDialogueData.cs` — Serializable data classes for dialogue JSON

**Editor** (Unity Editor only):

- **ScriptedImporter** — Auto-imports `.talenode.json` files as assets
- **Custom Inspector** — Node stats, characters, variables, locale coverage
- **Graph View** — Visual read-only dialogue graph with auto-layout
- **Playtest Panel** — Run dialogues inside the editor with variable watch

See the [Unity Editor Tools](unity-editor.md) guide for detailed usage.

### File Extension

Name your exported JSON files with the `.talenode.json` compound extension (e.g. `intro.talenode.json`). The ScriptedImporter only matches this extension — regular `.json` files are not affected.

## Usage

After installing the plugin, export your dialogue as JSON (File > Export JSON), rename it to `*.talenode.json`, and drop it into your Unity project's `Assets/` folder. The file is auto-imported as a `TaleNodeDialogue` asset.

Refer to the [Engine Integration](integration-guide.md) guide for runtime API details, or the [Unity Editor Tools](unity-editor.md) guide for the editor workflow.
