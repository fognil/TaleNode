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

### Data Classes

```csharp
[System.Serializable]
public class TaleNodeDialogue
{
    public string version;
    public string name;
    public TaleNodeVariable[] variables;
    public TaleNodeCharacter[] characters;
    public TaleNodeNode[] nodes;
}

[System.Serializable]
public class TaleNodeNode
{
    public string id;
    public string type;

    // Dialogue fields
    public string speaker;
    public string text;
    public string emotion;
    public string portrait;
    public string audio;

    // Shared
    public string next;

    // Choice fields
    public string prompt;
    public TaleNodeOption[] options;

    // Condition fields
    public string variable;
    public string @operator;  // "operator" is a reserved word
    public object value;
    public string true_next;
    public string false_next;

    // Event fields
    public TaleNodeAction[] actions;

    // Random fields
    public TaleNodeBranch[] branches;

    // End fields
    public string tag;
}

[System.Serializable]
public class TaleNodeOption
{
    public string text;
    public string next;
    public TaleNodeCondition condition;
}

[System.Serializable]
public class TaleNodeCondition
{
    public string variable;
    public string @operator;
    public object value;
}

[System.Serializable]
public class TaleNodeAction
{
    public string action;
    public string key;
    public object value;
}

[System.Serializable]
public class TaleNodeBranch
{
    public float weight;
    public string next;
}

[System.Serializable]
public class TaleNodeVariable
{
    public string name;
    public string type;
    public object @default;
}

[System.Serializable]
public class TaleNodeCharacter
{
    public string id;
    public string name;
    public string color;
    public string portrait;
}
```

### Loading and Running

```csharp
public class DialogueRunner : MonoBehaviour
{
    private TaleNodeDialogue dialogue;
    private Dictionary<string, TaleNodeNode> nodeMap;
    private Dictionary<string, object> variables;

    public void LoadDialogue(string jsonPath)
    {
        string json = File.ReadAllText(jsonPath);
        dialogue = JsonUtility.FromJson<TaleNodeDialogue>(json);

        // Build lookup map
        nodeMap = new Dictionary<string, TaleNodeNode>();
        foreach (var node in dialogue.nodes)
            nodeMap[node.id] = node;

        // Initialize variables
        variables = new Dictionary<string, object>();
        foreach (var v in dialogue.variables)
            variables[v.name] = v.@default;
    }

    public void StartDialogue()
    {
        var startNode = dialogue.nodes.First(n => n.type == "start");
        ProcessNode(nodeMap[startNode.next]);
    }

    private void ProcessNode(TaleNodeNode node)
    {
        switch (node.type)
        {
            case "dialogue":
                ShowDialogue(node.speaker, node.text, node.emotion);
                // On continue: ProcessNode(nodeMap[node.next]);
                break;

            case "choice":
                ShowChoices(node.prompt, node.options);
                // On selection: ProcessNode(nodeMap[selectedOption.next]);
                break;

            case "condition":
                bool result = EvaluateCondition(node.variable, node.@operator, node.value);
                string nextId = result ? node.true_next : node.false_next;
                ProcessNode(nodeMap[nextId]);
                break;

            case "event":
                ExecuteActions(node.actions);
                ProcessNode(nodeMap[node.next]);
                break;

            case "random":
                var branch = PickWeightedRandom(node.branches);
                ProcessNode(nodeMap[branch.next]);
                break;

            case "end":
                EndDialogue(node.tag);
                break;
        }
    }
}
```

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
