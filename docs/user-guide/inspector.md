# Inspector Panel

The Inspector panel appears on the right side of the screen when exactly one node is selected. It provides property editors specific to each node type.

**Width**: 280px default, minimum 220px.

## General Information

All node types show:

- **Node ID**: The first 8 characters of the node's UUID (for debugging/reference)

## Per-Type Editors

### Start Node

No editable properties. The Inspector shows a brief description of the Start node's purpose.

### Dialogue Node

| Field | Description |
|---|---|
| **Speaker** | Character name. Type freely or select from defined Characters. |
| **Text** | Multi-line text area for the dialogue line. |
| **Emotion** | Dropdown: `neutral`, `happy`, `sad`, `angry`, `surprised`, `scared` |
| **Audio Clip** | Optional file path for voice acting audio. |

### Choice Node

| Field | Description |
|---|---|
| **Prompt** | The question or prompt shown to the player. |
| **Options** | A list of choice texts. Each has a text field. |

- Click **+ Add Option** to add a new choice
- Click the **X** button to remove a choice (minimum 1 must remain)
- Each option can have an optional **condition** for visibility

### Condition Node

| Field | Description |
|---|---|
| **Variable** | The variable name to evaluate. |
| **Operator** | Dropdown: `==`, `!=`, `>`, `<`, `>=`, `<=`, `contains` |
| **Value** | Type selector (Bool/Int/Float/Text) + value editor. |

### Event Node

| Field | Description |
|---|---|
| **Actions** | A list of game actions to trigger. |

Each action has:

- **Type**: `SetVariable`, `AddItem`, `RemoveItem`, `PlaySound`, or `Custom`
- **Key**: The variable or item identifier
- **Value**: Type selector + value

Click **+ Add Action** to add new actions, or **X** to remove.

### Random Node

| Field | Description |
|---|---|
| **Branches** | List of weighted branches. |

Each branch shows a **weight** as a percentage drag-value (0%–100%). The Inspector displays the total weight and a warning if it doesn't sum to 100%.

- Click **+ Add Branch** to add a branch
- Click **X** to remove (minimum 1 must remain)

### End Node

| Field | Description |
|---|---|
| **Tag** | Text field for the ending identifier. |

Common suggestions: `good_ending`, `bad_ending`, `continue`, `death`, `quest_complete`.

## Tips

!!! tip
    Changes in the Inspector take effect immediately and are tracked by the undo system. Press ++ctrl+z++ to revert any change.

!!! tip
    Click on a different node (or click the empty canvas) to switch the Inspector to a different node or hide it.
