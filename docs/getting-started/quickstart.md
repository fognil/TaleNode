# Quick Start

Create your first branching dialogue in 5 minutes.

## Step 1: Create a New Project

Launch TaleNode. You start with an empty canvas. Use **File > New** (or ++ctrl+n++) to ensure a clean slate.

## Step 2: Add a Start Node

Right-click on the canvas and select **Add Node > Start**. A green Start node appears. Every dialogue needs exactly one Start node — it's the entry point.

## Step 3: Add a Dialogue Node

Right-click again to the right of the Start node and select **Add Node > Dialogue**. Click the Dialogue node to select it — the Inspector panel opens on the right.

In the Inspector, fill in:

- **Speaker**: `Guard`
- **Text**: `Halt! Who goes there?`
- **Emotion**: `angry`

## Step 4: Connect the Nodes

Hover over the **output port** (right side) of the Start node. Click and drag a wire to the **input port** (left side) of the Dialogue node. Release to create the connection.

You should see a bezier curve connecting the two nodes.

## Step 5: Add a Choice

Right-click and add a **Choice** node. Connect the Dialogue node's output to the Choice node's input.

Select the Choice node. In the Inspector, set:

- **Prompt**: `What do you say?`
- **Option 1**: `I'm a friend.`
- **Option 2**: `None of your business.`

## Step 6: Add Responses

Add two more **Dialogue** nodes — one for each choice outcome.

- Connect Choice output "I'm a friend." to the first Dialogue node.
    - Speaker: `Guard`, Text: `Very well, you may pass.`, Emotion: `neutral`
- Connect Choice output "None of your business." to the second Dialogue node.
    - Speaker: `Guard`, Text: `Then you shall not pass!`, Emotion: `angry`

## Step 7: Add End Nodes

Add two **End** nodes. Connect each response Dialogue node to its own End node.

- First End: Tag = `friendly`
- Second End: Tag = `hostile`

## Step 8: Test It

Open **View > Playtest Panel**. Click **Start** to walk through your dialogue. Try both choices.

## Step 9: Save and Export

1. **Save**: ++ctrl+s++ — saves as a `.talenode` project file (preserves positions, all data).
2. **Export**: **File > Export JSON...** — outputs a clean JSON file for your game engine.

## Your First Graph

Your finished graph should look like this:

```
[Start] → [Dialogue: "Halt!"] → [Choice: "What do you say?"]
                                        ├─ "Friend" → [Dialogue: "You may pass."] → [End: friendly]
                                        └─ "Business" → [Dialogue: "Shall not pass!"] → [End: hostile]
```

## What's Next?

- [Canvas & Navigation](../user-guide/canvas.md) — Learn pan, zoom, and selection
- [All Node Types](../user-guide/nodes.md) — Explore Condition, Event, and Random nodes
- [Variables](../user-guide/variables.md) — Add game state to your dialogues
- [JSON Export Format](../export/json-format.md) — Understand the output format
