# JSON Export Format

When you export via **File > Export JSON...**, TaleNode produces a clean, flat JSON file designed for easy parsing in any game engine.

## Top-Level Structure

```json
{
  "version": "1.0",
  "name": "My Dialogue",
  "variables": [...],
  "characters": [...],
  "nodes": [...]
}
```

| Field | Type | Description |
|---|---|---|
| `version` | string | Format version, currently `"1.0"` |
| `name` | string | Project name |
| `variables` | array | Variable definitions with defaults |
| `characters` | array | Character database |
| `nodes` | array | Flat list of all nodes with baked connections |

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
      "portrait": "portraits/guard.png"
    }
  ]
}
```

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

- `action` types: `SetVariable`, `AddItem`, `RemoveItem`, `PlaySound`, `Custom`

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
