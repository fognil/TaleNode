# Nodes

Nodes are the building blocks of your dialogue graph. TaleNode provides 7 node types, each with a specific purpose.

## Node Types Overview

| Type | Color | Inputs | Outputs | Purpose |
|---|---|---|---|---|
| <span class="node-badge node-badge--start">Start</span> | Green | 0 | 1 | Entry point |
| <span class="node-badge node-badge--dialogue">Dialogue</span> | Blue | 1 | 1 | Character speech |
| <span class="node-badge node-badge--choice">Choice</span> | Yellow | 1 | N | Player choices |
| <span class="node-badge node-badge--condition">Condition</span> | Orange | 1 | 2 | Variable branching |
| <span class="node-badge node-badge--event">Event</span> | Purple | 1 | 1 | Trigger actions |
| <span class="node-badge node-badge--random">Random</span> | Gray | 1 | N | Weighted random |
| <span class="node-badge node-badge--end">End</span> | Red | 1 | 0 | Conversation end |

## Adding Nodes

Right-click on the canvas to open the context menu, then select **Add Node** and choose a type. The node appears at the cursor position.

## Start Node

The entry point of your dialogue. Every graph needs exactly one Start node.

- **Ports**: No inputs, 1 output
- **Properties**: None
- The validation system warns if you have zero or multiple Start nodes.

## Dialogue Node

Represents a line of dialogue spoken by a character.

- **Ports**: 1 input, 1 output
- **Properties**:
    - **Speaker**: Character name (or reference to a Character)
    - **Text**: The dialogue line (multi-line supported)
    - **Emotion**: One of `neutral`, `happy`, `sad`, `angry`, `surprised`, `scared`
    - **Audio Clip**: Optional path to a voice audio file
- **Canvas preview**: Shows the speaker name in the header and the first 2 lines of text.

## Choice Node

Presents the player with options to choose from.

- **Ports**: 1 input, N outputs (one per choice option)
- **Properties**:
    - **Prompt**: The question shown to the player
    - **Options**: A list of choice texts, each with an optional visibility condition
- Each option creates its own output port. Connect each output to the next node in that branch.
- **Minimum**: 1 option (cannot remove the last one)
- New Choice nodes start with 2 default options.

!!! tip
    Choice options can have conditions — a choice is only visible to the player if the condition is met. Set this in the Inspector.

## Condition Node

Branches the flow based on a variable's value.

- **Ports**: 1 input, 2 outputs (labeled **True** and **False**)
- **Properties**:
    - **Variable**: The variable name to evaluate
    - **Operator**: `==`, `!=`, `>`, `<`, `>=`, `<=`, `contains`
    - **Value**: The comparison value (Bool, Int, Float, or Text)
- **Canvas preview**: Shows `variable operator value` (e.g., `gold >= 100`).

## Event Node

Triggers game actions — set variables, add/remove items, play sounds, or fire custom events.

- **Ports**: 1 input, 1 output
- **Properties**:
    - **Actions**: A list of actions, each with:
        - **Type**: `SetVariable`, `AddItem`, `RemoveItem`, `PlaySound`, or `Custom`
        - **Key**: The target variable or item name
        - **Value**: The value to set
- **Canvas preview**: Shows up to 3 action summaries, with "+N more" for overflow.

## Random Node

Randomly selects one of several branches based on weights.

- **Ports**: 1 input, N outputs (one per branch)
- **Properties**:
    - **Branches**: A list of branches, each with a **weight** (0%–100%)
- Weights should sum to 100% — the Inspector shows a warning if they don't.
- **Minimum**: 1 branch.
- New Random nodes start with 2 branches at 50% each.

## End Node

Marks the end of a conversation path.

- **Ports**: 1 input, no outputs
- **Properties**:
    - **Tag**: An identifier for the ending (e.g., `good_ending`, `bad_ending`, `continue`)
- **Canvas preview**: Shows the tag if set.

## Node Dimensions

All nodes are **200px wide** (canvas units). Height is calculated based on the number of ports and text content. The header is 28px tall. Port circles have a 6px radius.
