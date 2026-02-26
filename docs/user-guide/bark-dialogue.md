# Bark Dialogue

Bark dialogue (also called ambient dialogue) is short, context-sensitive lines that characters speak outside of the main dialogue tree. Think of NPCs muttering as you walk by, guards commenting on the weather, or shopkeepers calling out to passersby.

## Overview

Barks are attached to **characters**, not to the node graph. Each character can have a list of bark lines with optional weights and conditions. At runtime, your game engine picks a bark line based on weight and condition state.

## Managing Barks

Open the **Barks** panel via **View > Bark Dialogue** in the menu bar. The panel appears as a dockable tab.

### Selecting a Character

Use the **Character** dropdown at the top of the Barks panel to choose which character's bark lines to edit. Only characters defined in the left panel appear here.

### Adding a Bark Line

Click **+ Add Bark** to create a new line. Each bark line has:

| Field | Type | Description |
|---|---|---|
| **Text** | String | The bark dialogue text |
| **Weight** | Float (0.0–10.0) | Selection probability weight. Higher = more likely |
| **Condition** | String (optional) | Variable name that must be truthy for this line to play |

### Editing Bark Lines

- **Text**: Click the text field to edit the line
- **Weight**: Use the drag slider (0.0 to 10.0, default 1.0)
- **Condition**: Enter a variable name. If set, this bark only plays when that variable is truthy at runtime

### Removing a Bark Line

Click the **X** button next to a bark line to remove it.

## Bark Selection at Runtime

Your game engine should:

1. Filter bark lines by condition (skip lines whose condition variable is falsy)
2. From the remaining lines, select one randomly using weights

For example, if a character has three bark lines with weights 2.0, 1.0, and 1.0, the first line plays ~50% of the time and the others ~25% each.

## Export

### JSON Export

Bark groups are included in the JSON export when characters have bark lines defined:

```json
{
  "barks": [
    {
      "character": "Guard",
      "lines": [
        { "text": "Move along.", "weight": 1.0 },
        { "text": "Stay out of trouble.", "weight": 1.0, "condition": "is_night" }
      ]
    }
  ]
}
```

The `barks` array is omitted entirely when no characters have bark lines.

### CSV Export

Export bark lines as a CSV file via **File > Export Bark Dialogue (CSV)...**. The CSV format:

```
Character,Text,Weight,Condition
Guard,Move along.,1.00,
Guard,Stay out of trouble.,1.00,is_night
Merchant,Fine wares for sale!,2.00,shop_open
```

This format is useful for spreadsheet review, voice actor scripts, or importing into external tools.

## Tips

!!! tip
    Use weights to control bark variety. Give common greetings higher weights and rare remarks lower weights.

!!! tip
    Conditions let you make barks context-sensitive. A guard might say different things during day vs. night by using `is_day` and `is_night` condition variables.

!!! tip
    Deleting a character from the left panel also removes all their bark lines.
