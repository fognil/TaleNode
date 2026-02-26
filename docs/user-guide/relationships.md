# NPC Relationships

Relationships track affinity, reputation, or any numeric value between the player and NPCs. Each character can have multiple named relationship tracks with configurable ranges.

## Defining Relationships

Relationships are configured per character in the **left panel** under each character's **Relationships** section.

### Adding a Relationship

Click **+ Add** in the Relationships section of a character. A new relationship is created with:

| Field | Default | Description |
|---|---|---|
| **Name** | `"Relationship"` | Label for this track (e.g., "Friendship", "Trust", "Fear") |
| **Value** | `0` | Starting/default value |
| **Min** | `-100` | Minimum allowed value |
| **Max** | `100` | Maximum allowed value |

### Editing a Relationship

- **Name**: Type to rename (e.g., "Friendship", "Reputation", "Romance")
- **Value**: Drag slider to set the default starting value
- **Min/Max**: The value is clamped to this range at runtime

### Removing a Relationship

Click the **X** button next to a relationship to remove it.

## Modifying Relationships in Dialogue

Use **Event nodes** with the `ModifyRelationship` action type to change relationship values during dialogue.

- **Action type**: `modify_relationship`
- **Key**: Format as `CharacterName.RelationshipName` (e.g., `Guard.Trust`)
- **Value**: The amount to add (use negative values to decrease)

Example: An Event node with action `modify_relationship`, key `Princess.Friendship`, value `10` increases the Princess's Friendship by 10.

## Export

Relationships are exported as part of the character data in JSON:

```json
{
  "characters": [
    {
      "id": "char_1",
      "name": "Guard",
      "color": "#4A90D9",
      "relationships": [
        {
          "name": "Trust",
          "default_value": 0,
          "min": -100,
          "max": 100
        }
      ]
    }
  ]
}
```

The `relationships` array is omitted when a character has no relationships defined.

Event nodes that modify relationships export as:

```json
{
  "id": "evt_1",
  "type": "event",
  "actions": [
    { "action": "modify_relationship", "key": "Guard.Trust", "value": 10 }
  ],
  "next": "dlg_2"
}
```

## Runtime Integration

Your game engine should:

1. Initialize each character's relationships using `default_value` from the export
2. When processing a `modify_relationship` event action, parse the key as `Character.Relationship`, find the matching relationship, and add the value
3. Clamp the result to the `min`/`max` range
4. Use the current relationship value in Condition nodes to branch dialogue based on affinity

## Tips

!!! tip
    Use meaningful relationship names that map to your game's systems. "Trust" and "Fear" are more expressive than "Relationship1".

!!! tip
    You can define multiple relationships per character. A guard might have both "Trust" (-100 to 100) and "Bribe Amount" (0 to 1000).

!!! tip
    Combine relationships with Condition nodes to gate dialogue options. For example, a romance option only appears when `Princess.Friendship >= 50`.
