# World Database

TaleNode includes a world-building database for cataloging items, locations, lore entries, and other game world entities. Use it to organize your game's universe alongside your dialogue trees.

## Overview

World entities are standalone data records that describe things in your game world — weapons, cities, historical events, important characters, or any custom category. Each entity has a name, category, description, tags, and key-value properties.

## Managing Entities

Open the **World DB** panel via **View > World Database** in the menu bar. The panel appears as a dockable tab.

### Category Filter

Use the **Filter** dropdown at the top to show only entities of a specific category, or select **All** to see everything.

### Adding an Entity

Click **+ Add Entity** to create a new entity. It starts as an `Item` by default.

| Field | Type | Description |
|---|---|---|
| **Name** | String | Entity name (e.g., "Iron Sword", "Riverwood") |
| **Category** | Enum | `Item`, `Location`, `Lore`, or `Character` |
| **Description** | String | Detailed description or lore text |
| **Tags** | List | Comma-separated tags for filtering |
| **Properties** | Key-Value | Custom properties (e.g., `damage = 15`, `population = 500`) |

### Editing an Entity

Each entity has a collapsible section in the World DB panel:

- **Name**: Click to edit the entity name
- **Category**: Use the dropdown to change the category
- **Description**: Click to edit the multiline description
- **Properties**: Each property is a key-value pair. Click either field to edit

### Adding Properties

Click **+ Property** within an entity to add a new key-value property. Properties are freeform — use them for whatever metadata your game needs:

- `damage = 15`
- `weight = 2.5`
- `rarity = legendary`
- `region = Northern Mountains`

### Removing Entities and Properties

- Click **X** next to a property to remove it
- Click **Delete Entity** at the bottom of an entity's section to remove the entire entity

## Categories

| Category | Use Case |
|---|---|
| **Item** | Weapons, armor, consumables, quest items, key items |
| **Location** | Cities, dungeons, points of interest, shops |
| **Lore** | Historical events, legends, in-world books, notes |
| **Character** | NPCs, factions, organizations (separate from dialogue characters) |

## Export

World entities are included in the JSON export:

```json
{
  "world_entities": [
    {
      "name": "Iron Sword",
      "category": "Item",
      "description": "A basic iron sword.",
      "tags": ["weapon", "starter"],
      "properties": [
        { "key": "damage", "value": "15" },
        { "key": "weight", "value": "2.5" }
      ]
    },
    {
      "name": "Riverwood",
      "category": "Location",
      "description": "A small village by the river.",
      "properties": [
        { "key": "population", "value": "50" },
        { "key": "region", "value": "Whiterun Hold" }
      ]
    }
  ]
}
```

The `world_entities` array is omitted when no entities are defined. The `description`, `tags`, and `properties` fields are omitted when empty.

## Runtime Integration

Your game engine should:

1. Load the `world_entities` array at startup
2. Index entities by name or category for quick lookup
3. Use properties to populate UI (item stats, location descriptions, etc.)
4. Reference entity names from dialogue text or Event node actions

## Tips

!!! tip
    Use tags to create cross-cutting categories. An entity can be both a "quest_item" and a "weapon", making it easy to filter in your game's inventory UI.

!!! tip
    Properties are string-based. Parse numeric values in your game engine as needed (e.g., `parseInt("15")` for damage).

!!! tip
    Use the Category filter to focus on one type at a time when your database grows large.
