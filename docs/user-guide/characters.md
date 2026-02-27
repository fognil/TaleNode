# Characters

Characters represent the speakers in your dialogue. Define them once, then reference them across all Dialogue nodes for consistent naming and styling.

## Managing Characters

Characters are managed in the **left panel** under the **Characters** section.

### Adding a Character

Click **+ Add** in the Characters section. A new character is created with:

- **Name**: `Character` (edit to rename)
- **Color**: Blue (`rgba(74, 144, 217, 255)`) by default
- **Portrait Path**: Empty

### Editing a Character

Each character card shows aligned property fields:

| Field | Description |
|---|---|
| **Name** | Display name (e.g., "Guard", "Princess", "Merchant") |
| **Color** | RGB color picker — used for visual identification in the UI |
| **Portrait** | File path to a portrait image, with a live thumbnail preview and **[...]** browse button |
| **Voice** | Voice assignment dropdown (visible when voices are loaded — see [Voice Synthesis](voice-synthesis.md)) |

A colored indicator square is shown next to each character name in the left panel header, matching the assigned color.

### Portrait Preview

When a portrait path is set, a small thumbnail is displayed inline next to the path field. Click **[...]** to open a file picker (supports PNG, JPG, JPEG, BMP, GIF). Portraits are cached in memory for fast rendering — changing the path updates the preview immediately.

Portraits also appear on **Dialogue nodes** on the canvas (at medium zoom and above) and in the **Inspector** when editing a Dialogue node with a linked speaker.

### Removing a Character

Click the delete button to remove a character. Dialogue nodes that reference the removed character will retain the speaker name as plain text.

### Relationships

Each character can have named relationship tracks (e.g., Friendship, Trust, Fear) with numeric values and configurable min/max ranges. Relationships are managed in a collapsible **Relationships** section within each character.

See [NPC Relationships](relationships.md) for full details on defining relationships, modifying them via Event nodes, and using them in conditions.

## Using Characters in Dialogue Nodes

When editing a Dialogue node in the Inspector, the **Speaker** field lets you type a character name. Characters you've defined are available for consistent reference.

The `speaker_id` field in the data model links the Dialogue node to a specific Character by UUID, ensuring updates to the character name propagate correctly.

### Bark Dialogue

Characters can also have bark/ambient dialogue lines — short context-sensitive lines for use outside the main dialogue tree. Manage barks in the dedicated **Barks** panel.

See [Bark Dialogue](bark-dialogue.md) for full details.

## Characters in Export

Characters are exported with human-readable IDs. Relationships are included when defined:

```json
{
  "characters": [
    {
      "id": "char_1",
      "name": "Guard",
      "color": "#4A90D9",
      "portrait": "portraits/guard.png",
      "relationships": [
        { "name": "Trust", "default_value": 0, "min": -100, "max": 100 }
      ]
    },
    {
      "id": "char_2",
      "name": "Princess",
      "color": "#E84393",
      "portrait": "portraits/princess.png"
    }
  ]
}
```

Dialogue nodes reference characters by their exported ID (e.g., `"speaker": "char_1"`).

## Tips

!!! tip
    Define all your characters in the left panel before building the graph. This makes it easy to maintain consistent speaker names.

!!! tip
    Use distinct colors for each character — this makes it easier to visually scan the left panel and identify speakers at a glance.
