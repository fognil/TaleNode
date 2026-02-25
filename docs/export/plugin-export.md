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

The Unity plugin exports to a `TaleNode/` folder and includes:

- `TaleNodeRunner.cs` — MonoBehaviour dialogue runner
- `TaleNodeExpression.cs` — Expression parser and evaluator

Add the scripts to your Unity project's `Assets/` folder.

## Usage

After exporting the plugin, also export your dialogue as JSON (File > Export JSON). Place the JSON file in your game project, then use the runtime plugin to load and play it.

Refer to the [Engine Integration](integration-guide.md) guide for detailed setup instructions per engine.
