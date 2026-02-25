# Engine Integration Guide

This guide shows how to load and play TaleNode dialogue JSON in your game engine.

## General Approach

1. **Load** the JSON file and parse it into your engine's data structures
2. **Initialize** variables from the `variables` array
3. **Find** the `start` node — this is the entry point
4. **Follow** `next` fields to traverse the dialogue graph
5. **Handle** each node type according to its `type` field

## Traversal Algorithm

```
1. current_node = find node where type == "start"
2. current_node = find node by current_node.next
3. loop:
   a. switch on current_node.type:
      - "dialogue": display text, wait for player input, follow next
      - "choice": display options, wait for selection, follow selected option's next
      - "condition": evaluate variable, follow true_next or false_next
      - "event": execute actions, follow next
      - "random": pick weighted random branch, follow branch's next
      - "end": dialogue is over
   b. if next is null, dialogue is over
```

## Unity (C#)

The TaleNode Unity package includes a ready-to-use `TaleNodeRunner` class. You don't need to write your own data classes or traversal logic.

### Quick Start

```csharp
using TaleNode;
using UnityEngine;

public class DialogueUI : MonoBehaviour
{
    private TaleNodeRunner runner;

    void Start()
    {
        runner = new TaleNodeRunner();

        runner.OnDialogueLine += (s, e) =>
        {
            Debug.Log($"{e.Speaker}: {e.Text}");
            // Show in your UI, then call runner.Advance() when player clicks
        };

        runner.OnChoicePresented += (s, e) =>
        {
            Debug.Log($"Prompt: {e.Prompt}");
            for (int i = 0; i < e.Options.Count; i++)
                Debug.Log($"  {i}: {e.Options[i]}");
            // Show choices in UI, then call runner.Choose(index)
        };

        runner.OnDialogueEnded += (s, e) =>
        {
            Debug.Log($"Ended with tag: {e.Tag}");
        };

        runner.OnEventTriggered += (s, e) =>
        {
            Debug.Log($"Event: {e.Action} {e.Key}={e.Value}");
        };

        runner.OnVariableChanged += (s, e) =>
        {
            Debug.Log($"Variable {e.Key} = {e.Value}");
        };

        runner.LoadDialogue("Assets/Dialogues/intro.talenode.json");
        runner.Start();
    }
}
```

### TaleNodeRunner API

| Method | Description |
|---|---|
| `LoadDialogue(string path)` | Load dialogue from a JSON file path |
| `LoadFromString(string json)` | Load dialogue from a JSON string |
| `Start(string startNodeId = null)` | Start the dialogue (optionally from a specific node) |
| `Advance()` | Continue to the next node after a dialogue line |
| `Choose(int index)` | Select a choice option by index |
| `GetVariable(string name)` | Get a runtime variable value |
| `SetVariable(string name, TaleValue value)` | Set a runtime variable value |
| `Stop()` | Stop the dialogue immediately |
| `IsRunning` | Whether a dialogue is currently active |

### Events

| Event | Args | When |
|---|---|---|
| `OnDialogueStarted` | `EventArgs` | Dialogue begins |
| `OnDialogueLine` | `Speaker`, `Text`, `Emotion`, `Portrait`, `Audio`, `NodeId` | A dialogue line is reached |
| `OnChoicePresented` | `Prompt`, `Options` (List&lt;string&gt;) | A choice node is reached |
| `OnDialogueEnded` | `Tag` | Dialogue ends (at an End node or dead end) |
| `OnEventTriggered` | `Action`, `Key`, `Value` | A non-variable event action fires |
| `OnVariableChanged` | `Key`, `Value` | A variable is set or modified |

### Expression Interpolation

The runner automatically interpolates `{variable}` expressions in dialogue text and choice prompts. You can also use the expression engine directly:

```csharp
using TaleNode;

// Evaluate an expression
TaleValue result = TaleNodeExpression.Evaluate("gold >= 100", variables);

// Evaluate as boolean
bool check = TaleNodeExpression.EvaluateBool("has_key && level > 5", variables);

// Interpolate text
string text = TaleNodeExpression.InterpolateText("You have {gold} gold.", variables);
```

### Editor Tools

The Unity package also includes editor tools for viewing and testing dialogues inside Unity. See the [Unity Editor Tools](unity-editor.md) guide.

## Godot (GDScript)

### Loading

```gdscript
func load_dialogue(path: String) -> Dictionary:
    var file = FileAccess.open(path, FileAccess.READ)
    var json = JSON.parse_string(file.get_as_text())
    return json

func build_node_map(dialogue: Dictionary) -> Dictionary:
    var map = {}
    for node in dialogue["nodes"]:
        map[node["id"]] = node
    return map
```

### Running

```gdscript
var dialogue: Dictionary
var node_map: Dictionary
var variables: Dictionary

func start_dialogue(path: String):
    dialogue = load_dialogue(path)
    node_map = build_node_map(dialogue)

    # Initialize variables
    variables = {}
    for v in dialogue["variables"]:
        variables[v["name"]] = v["default"]

    # Find start node
    var start = dialogue["nodes"].filter(func(n): return n["type"] == "start")[0]
    process_node(node_map[start["next"]])

func process_node(node: Dictionary):
    match node["type"]:
        "dialogue":
            show_dialogue(node)
            # Call process_node(node_map[node["next"]]) on continue

        "choice":
            show_choices(node)
            # Call process_node(node_map[option["next"]]) on selection

        "condition":
            var result = evaluate(node["variable"], node["operator"], node["value"])
            var next_id = node["true_next"] if result else node["false_next"]
            process_node(node_map[next_id])

        "event":
            for action in node["actions"]:
                execute_action(action)
            process_node(node_map[node["next"]])

        "random":
            var branch = pick_weighted(node["branches"])
            process_node(node_map[branch["next"]])

        "end":
            end_dialogue(node["tag"])
```

## Tips

!!! tip "Null next fields"
    Always check if `next` (or `true_next`/`false_next`) is `null` before following it. A null next means the output port was not connected in the editor.

!!! tip "Character lookup"
    Build a character map (`id → character`) at load time. Dialogue nodes reference characters by ID (e.g., `"char_1"`), not by name.

!!! tip "Choice conditions"
    If a choice option has a `condition`, evaluate it against your variables to determine visibility. Only show options whose conditions are met (or have no condition).

!!! tip "Custom events"
    Event actions with type `Custom` have a freeform key — map these to your game's event system as needed.
