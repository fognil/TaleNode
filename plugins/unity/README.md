# TaleNode Unity Plugin

Node-based dialogue system runtime and editor tools for Unity 6+.

## Installation

1. Open your Unity project (Unity 6 or later)
2. Go to **Window > Package Manager**
3. Click **+** > **Add package from disk...**
4. Navigate to `plugins/unity/package.json` and select it

The package requires `com.unity.nuget.newtonsoft-json` (installed automatically).

## Setup

### Importing Dialogue Files

Export your dialogue from the TaleNode desktop app as JSON, then rename the file with a `.talenode.json` extension (e.g. `intro.talenode.json`). Drop it into your Unity `Assets/` folder. The ScriptedImporter will automatically create a `TaleNodeDialogue` asset.

### Custom Inspector

Select any `.talenode.json` asset in the Project window to see:

- Dialogue name, version, and node count
- Node statistics by type (color-coded)
- Characters and variables lists
- Locale coverage percentages
- Buttons to open Graph View and Playtest

### Graph View

Open via **Window > TaleNode > Dialogue Graph** or the inspector button.

- Pick a dialogue asset from the toolbar ObjectField
- Nodes are auto-laid out left-to-right (BFS from start)
- Node colors match the TaleNode desktop app
- Drag to pan, scroll to zoom, click nodes to select
- Use the search field to filter nodes by name/ID
- Switch locale from the toolbar dropdown to preview translations
- Read-only: editing is done in the TaleNode desktop app

### Playtest

Open via **Window > TaleNode > Playtest** or the inspector button.

- Select a dialogue asset and click **Play**
- Dialogue lines appear in a scrollable log
- Choices appear as clickable buttons
- Variable watch panel shows current values (highlights changes)
- Syncs with Graph View to highlight the active node
- Switch locale to test translations

## Runtime API

The runtime classes (`TaleNodeRunner`, `TaleNodeExpression`) work independently of the editor. Use them in your game scripts:

```csharp
using TaleNode;

var runner = new TaleNodeRunner();
runner.OnDialogueLine += (s, e) => Debug.Log($"{e.Speaker}: {e.Text}");
runner.OnChoicePresented += (s, e) => {
    for (int i = 0; i < e.Options.Count; i++)
        Debug.Log($"  {i}: {e.Options[i]}");
};
runner.OnDialogueEnded += (s, e) => Debug.Log("Done");

runner.LoadDialogue("Assets/Dialogues/intro.talenode.json");
runner.Start();

// After OnDialogueLine fires:
runner.Advance();

// After OnChoicePresented fires:
runner.Choose(0);
```

## File Structure

```
Runtime/
  TaleNodeRunner.cs          — Dialogue execution engine
  TaleNodeExpression.cs      — Expression parser and evaluator
  TaleNodeDialogueData.cs    — Serializable data classes
Editor/
  Asset/
    TaleNodeDialogue.cs      — ScriptableObject wrapper
    TaleNodeDialogueImporter.cs — ScriptedImporter for .talenode.json
    TaleNodeDialogueEditor.cs   — Custom inspector
  GraphView/
    TaleNodeGraphWindow.cs   — EditorWindow host
    TaleNodeGraphView.cs     — GraphView visualization
    TaleNodeGraphNode.cs     — Node rendering per type
    TaleNodeGraphLayout.cs   — BFS auto-layout
    TaleNodeGraphStyles.uss  — USS stylesheet
  Playtest/
    TaleNodePlaytestPanel.cs — In-editor dialogue tester
```

## Requirements

- Unity 6+ (6000.0)
- Newtonsoft.Json (com.unity.nuget.newtonsoft-json)
