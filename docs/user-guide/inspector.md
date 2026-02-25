# Inspector Panel

The Inspector panel appears on the right side of the screen when exactly one node is selected. It provides property editors specific to each node type.

**Width**: 280px default, minimum 220px.

## General Information

All node types show:

- **Node ID**: The first 8 characters of the node's UUID (for debugging/reference)
- **Locale Switcher**: A dropdown to switch between default and translation locales (only visible when extra locales are defined — see [Localization](localization.md))

## Per-Type Editors

### Start Node

No editable properties. The Inspector shows a brief description of the Start node's purpose.

### Dialogue Node

| Field | Description |
|---|---|
| **Speaker** | Character name. Type freely or select from defined Characters. |
| **Text** | Multi-line text area for the dialogue line. Supports `{variable}` interpolation syntax. |
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

### SubGraph Node

| Field | Description |
|---|---|
| **Name** | Label for the nested dialogue. |

Shows the count of child nodes and connections inside the sub-graph. Double-click the node on the canvas to enter and edit the nested graph.

## Tags

Below the node-specific fields, every node has a **Tags** section:

- View existing tags as labels with **x** buttons to remove
- Type a tag name and click **+** to add a new tag
- Tags are used for organization and can be filtered in the [Bookmark Panel](bookmarks-tags.md)

## Review Status

Every node has a **Review** section at the bottom of the Inspector:

| Field | Description |
|---|---|
| **Status** | Dropdown: `Draft`, `Needs Review`, `Approved` |
| **Comments** | Shows the comment count for this node |

Review statuses appear as colored badges on nodes in the canvas. Use the [Comments Panel](comments-review.md) to manage comments and filter by review status.

## Locale Editing

When extra locales are configured and a non-default locale is selected in the Locale dropdown, translatable text fields (dialogue text, choice prompts, choice options) show an additional translation input labeled with the locale code (e.g., `[fr]`). Type the translation directly — untranslated fields show a dim "(untranslated)" placeholder.

See [Localization](localization.md) for full details on managing translations.

## Tips

!!! tip
    Changes in the Inspector take effect immediately and are tracked by the undo system. Press ++ctrl+z++ to revert any change.

!!! tip
    Click on a different node (or click the empty canvas) to switch the Inspector to a different node or hide it.
