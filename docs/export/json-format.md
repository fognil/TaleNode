# JSON Export Format

When you export via **File > Export JSON...**, TaleNode produces a clean, flat JSON file designed for easy parsing in any game engine.

## Top-Level Structure

```json
{
  "version": "1.0",
  "name": "My Dialogue",
  "variables": [...],
  "characters": [...],
  "nodes": [...],
  "barks": [...],
  "quests": [...],
  "world_entities": [...],
  "timelines": [...]
}
```

| Field | Type | Description |
|---|---|---|
| `version` | string | Format version, currently `"1.0"` |
| `name` | string | Project name |
| `variables` | array | Variable definitions with defaults |
| `characters` | array | Character database (with optional relationships) |
| `nodes` | array | Flat list of all nodes with baked connections |
| `barks` | array | *(optional)* Bark/ambient dialogue groups per character |
| `quests` | array | *(optional)* Quest definitions with objectives |
| `world_entities` | array | *(optional)* World-building entities (items, locations, lore) |
| `timelines` | array | *(optional)* Timeline/cutscene sequences |

## Node IDs

Exported nodes use **human-readable IDs** instead of UUIDs:

| Node Type | ID Pattern | Example |
|---|---|---|
| Start | `start_N` | `start_1` |
| Dialogue | `dlg_N` | `dlg_1`, `dlg_2` |
| Choice | `choice_N` | `choice_1` |
| Condition | `cond_N` | `cond_1` |
| Event | `evt_N` | `evt_1` |
| Random | `rand_N` | `rand_1` |
| End | `end_N` | `end_1`, `end_2` |

IDs are assigned deterministically, sorted by node position (top-to-bottom, left-to-right).

## Variables

```json
{
  "variables": [
    {
      "name": "gold",
      "type": "Int",
      "default": 500
    },
    {
      "name": "has_key",
      "type": "Bool",
      "default": false
    },
    {
      "name": "player_name",
      "type": "Text",
      "default": "Hero"
    }
  ]
}
```

## Characters

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
    }
  ]
}
```

The `relationships` array is omitted when a character has no relationships defined.

## Node Types in Export

### Start

```json
{
  "id": "start_1",
  "type": "start",
  "next": "dlg_1"
}
```

### Dialogue

```json
{
  "id": "dlg_1",
  "type": "dialogue",
  "speaker": "char_1",
  "text": "Halt! Who goes there?",
  "emotion": "angry",
  "portrait": null,
  "audio": null,
  "next": "choice_1"
}
```

- `speaker` references a character ID from the `characters` array
- `portrait` and `audio` are `null` when not set
- `next` is `null` if the output is not connected

### Choice

```json
{
  "id": "choice_1",
  "type": "choice",
  "prompt": "What do you say?",
  "options": [
    {
      "text": "I'm a friend.",
      "next": "dlg_2",
      "condition": null
    },
    {
      "text": "None of your business.",
      "next": "dlg_3",
      "condition": {
        "variable": "has_sword",
        "operator": "==",
        "value": true
      }
    }
  ]
}
```

- Each option has its own `next` target
- `condition` is `null` if no visibility condition is set

### Condition

```json
{
  "id": "cond_1",
  "type": "condition",
  "variable": "gold",
  "operator": ">=",
  "value": 100,
  "true_next": "dlg_4",
  "false_next": "dlg_5"
}
```

- Uses `true_next` / `false_next` instead of a single `next`
- `value` type matches the variable type (bool, int, float, or string)

### Event

```json
{
  "id": "evt_1",
  "type": "event",
  "actions": [
    {
      "action": "SetVariable",
      "key": "has_key",
      "value": true
    },
    {
      "action": "PlaySound",
      "key": "item_acquired",
      "value": ""
    }
  ],
  "next": "dlg_6"
}
```

- `action` types: `set_variable`, `add_item`, `remove_item`, `play_sound`, `modify_relationship`, `start_quest`, `complete_objective`, `fail_quest`, `custom`

### Random

```json
{
  "id": "rand_1",
  "type": "random",
  "branches": [
    {
      "weight": 0.7,
      "next": "dlg_7"
    },
    {
      "weight": 0.3,
      "next": "dlg_8"
    }
  ]
}
```

- Weights are floats (0.0 to 1.0), should sum to 1.0

### End

```json
{
  "id": "end_1",
  "type": "end",
  "tag": "good_ending"
}
```

## Localization

When the project has extra locales defined, three additional top-level fields appear:

```json
{
  "version": "1.0",
  "name": "My Dialogue",
  "default_locale": "en",
  "locales": ["en", "fr", "ja"],
  "strings": {
    "dlg_1": { "en": "Hello!", "fr": "Bonjour!", "ja": "こんにちは！" },
    "choice_1": { "en": "What next?", "fr": "Et maintenant?", "ja": "次は？" },
    "opt_choice_1_0": { "en": "Fight", "fr": "Combattre", "ja": "戦う" }
  },
  "variables": [...],
  "characters": [...],
  "nodes": [...]
}
```

| Field | Type | Description |
|---|---|---|
| `default_locale` | string | The primary language code (e.g., `"en"`) |
| `locales` | array | All locale codes including the default |
| `strings` | object | String table mapping readable IDs to locale→text |

- These fields are **omitted entirely** when no extra locales are defined
- String keys in `strings` use the same readable IDs as nodes (`dlg_1`, `choice_1`, `opt_choice_1_0`)
- The `text` fields in nodes still contain the default locale text
- See [Localization](../user-guide/localization.md) for the full localization workflow

## Barks

When characters have bark/ambient dialogue lines defined, a `barks` array is included:

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

The `barks` array is omitted when no characters have bark lines.

## Quests

When quests are defined, a `quests` array is included:

```json
{
  "quests": [
    {
      "name": "Find the Lost Artifact",
      "description": "The elder has asked you to recover the ancient artifact.",
      "objectives": [
        { "text": "Talk to the elder", "optional": false },
        { "text": "Search the ruins", "optional": false },
        { "text": "Find the secret passage", "optional": true }
      ]
    }
  ]
}
```

The `quests` array is omitted when no quests are defined. The `description` field is omitted when empty.

## World Entities

When world entities are defined, a `world_entities` array is included:

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
    }
  ]
}
```

The `world_entities` array is omitted when no entities are defined. The `description`, `tags`, and `properties` fields are omitted when empty.

## Timelines

When timelines are defined, a `timelines` array is included:

```json
{
  "timelines": [
    {
      "name": "Intro Cutscene",
      "steps": [
        {
          "action": { "type": "camera", "target": "player", "duration": 2.0 }
        },
        {
          "action": { "type": "wait", "seconds": 1.0 },
          "delay": 0.5
        }
      ]
    }
  ]
}
```

The `timelines` array is omitted when no timelines are defined. The `delay` field is omitted when zero. The `description` field is omitted when empty. The `loop_playback` field is omitted when false.

Action types: `dialogue`, `camera`, `animation`, `audio`, `wait`, `set_variable`, `custom`.

## Key Design Decisions

| Decision | Rationale |
|---|---|
| Flat node array | Easy to iterate, no recursive tree parsing needed |
| Baked `next` fields | No separate connections array — follow `next` to traverse |
| No positions | Game engines don't need editor layout data |
| Human-readable IDs | Easier to debug and reference in game scripts |
| Deterministic IDs | Same graph always produces the same export |

## Full Example

```json
{
  "version": "1.0",
  "name": "Guard Dialogue",
  "variables": [
    { "name": "reputation", "type": "Int", "default": 0 }
  ],
  "characters": [
    { "id": "char_1", "name": "Guard", "color": "#4A90D9", "portrait": "guard.png" }
  ],
  "nodes": [
    { "id": "start_1", "type": "start", "next": "dlg_1" },
    { "id": "dlg_1", "type": "dialogue", "speaker": "char_1", "text": "Halt!", "emotion": "angry", "portrait": null, "audio": null, "next": "choice_1" },
    { "id": "choice_1", "type": "choice", "prompt": "Respond:", "options": [
      { "text": "Friend", "next": "end_1", "condition": null },
      { "text": "Enemy", "next": "end_2", "condition": null }
    ]},
    { "id": "end_1", "type": "end", "tag": "friendly" },
    { "id": "end_2", "type": "end", "tag": "hostile" }
  ]
}
```
