# Node Reference

Detailed specification for all 7 node types. For a quicker overview, see [Nodes](../user-guide/nodes.md).

---

## Start Node

<span class="node-badge node-badge--start">Start</span>

Entry point of the dialogue graph.

| Property | Value |
|---|---|
| Color | Green (`#4CAF50`) |
| Input ports | 0 |
| Output ports | 1 |
| Editable properties | None |

**Validation rules:**

- Exactly 1 Start node per graph (error if missing, warning if multiple)

**Export format:**

```json
{
  "id": "start_1",
  "type": "start",
  "next": "dlg_1"
}
```

---

## Dialogue Node

<span class="node-badge node-badge--dialogue">Dialogue</span>

A line of character speech.

| Property | Type | Default | Description |
|---|---|---|---|
| `speaker_name` | String | `""` | Display name of the speaker |
| `speaker_id` | Option&lt;Uuid&gt; | `None` | Reference to a Character |
| `text` | String | `""` | The dialogue line (multi-line) |
| `emotion` | String | `"neutral"` | One of: neutral, happy, sad, angry, surprised, scared |
| `portrait_override` | Option&lt;String&gt; | `None` | Custom portrait image path |
| `audio_clip` | Option&lt;String&gt; | `None` | Voice audio file path |
| `metadata` | HashMap | `{}` | Custom key-value pairs |

| Port | Direction | Count |
|---|---|---|
| Input | In | 1 |
| Output | Out | 1 |

**Canvas display:** Header shows speaker name (or "Dialogue"). Body shows first 2 lines of text.

**Validation:** Warning if text is empty or whitespace-only.

**Export format:**

```json
{
  "id": "dlg_1",
  "type": "dialogue",
  "speaker": "char_1",
  "text": "Hello there!",
  "emotion": "happy",
  "portrait": null,
  "audio": null,
  "next": "choice_1"
}
```

---

## Choice Node

<span class="node-badge node-badge--choice">Choice</span>

Presents branching options to the player.

| Property | Type | Default | Description |
|---|---|---|---|
| `prompt` | String | `""` | The question shown to the player |
| `choices` | Vec&lt;ChoiceOption&gt; | 2 options | List of player choices |

Each **ChoiceOption**:

| Field | Type | Description |
|---|---|---|
| `id` | Uuid | Unique identifier |
| `text` | String | Choice label shown to player |
| `condition` | Option&lt;ConditionExpr&gt; | Visibility condition |

**ConditionExpr** (optional per choice):

| Field | Type | Description |
|---|---|---|
| `variable` | String | Variable name to check |
| `operator` | CompareOp | Comparison operator |
| `value` | VariableValue | Value to compare against |

| Port | Direction | Count |
|---|---|---|
| Input | In | 1 |
| Output | Out | N (one per choice option) |

**Rules:** Minimum 1 option. Output port labels sync with choice text.

**Export format:**

```json
{
  "id": "choice_1",
  "type": "choice",
  "prompt": "What do you do?",
  "options": [
    { "text": "Fight", "next": "evt_1", "condition": null },
    { "text": "Flee", "next": "dlg_3", "condition": { "variable": "courage", "operator": ">=", "value": 5 } }
  ]
}
```

---

## Condition Node

<span class="node-badge node-badge--condition">Condition</span>

Branches based on a variable comparison.

| Property | Type | Default | Description |
|---|---|---|---|
| `variable_name` | String | `""` | Variable to evaluate |
| `operator` | CompareOp | `==` | Comparison operator |
| `value` | VariableValue | `Bool(false)` | Comparison value |

**CompareOp values:** `==`, `!=`, `>`, `<`, `>=`, `<=`, `contains`

| Port | Direction | Count | Label |
|---|---|---|---|
| Input | In | 1 | — |
| Output | Out | 2 | "True", "False" |

**Canvas display:** Shows `variable operator value` summary (e.g., `gold >= 100`).

**Export format:**

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

---

## Event Node

<span class="node-badge node-badge--event">Event</span>

Triggers game actions when the dialogue passes through.

| Property | Type | Default | Description |
|---|---|---|---|
| `actions` | Vec&lt;EventAction&gt; | `[]` | List of actions to execute |

Each **EventAction**:

| Field | Type | Description |
|---|---|---|
| `action_type` | EventActionType | Type of action |
| `key` | String | Target variable/item name |
| `value` | VariableValue | Value to set |

**EventActionType values:** `SetVariable`, `AddItem`, `RemoveItem`, `PlaySound`, `Custom(String)`

| Port | Direction | Count |
|---|---|---|
| Input | In | 1 |
| Output | Out | 1 |

**Canvas display:** Shows up to 3 action summaries (`key: value`), with "+N more" for overflow.

**Export format:**

```json
{
  "id": "evt_1",
  "type": "event",
  "actions": [
    { "action": "SetVariable", "key": "has_key", "value": true },
    { "action": "AddItem", "key": "iron_sword", "value": 1 }
  ],
  "next": "dlg_6"
}
```

---

## Random Node

<span class="node-badge node-badge--random">Random</span>

Randomly selects a branch based on weights.

| Property | Type | Default | Description |
|---|---|---|---|
| `branches` | Vec&lt;RandomBranch&gt; | 2 at 50% | Weighted branch list |

Each **RandomBranch**:

| Field | Type | Description |
|---|---|---|
| `id` | Uuid | Unique identifier |
| `weight` | f32 | Weight value (0.0–1.0) |

| Port | Direction | Count |
|---|---|---|
| Input | In | 1 |
| Output | Out | N (one per branch) |

**Rules:** Minimum 1 branch. Weights should sum to 1.0 (100%). Output port labels show weight percentage.

**Export format:**

```json
{
  "id": "rand_1",
  "type": "random",
  "branches": [
    { "weight": 0.6, "next": "dlg_7" },
    { "weight": 0.4, "next": "dlg_8" }
  ]
}
```

---

## End Node

<span class="node-badge node-badge--end">End</span>

Marks the end of a conversation path.

| Property | Type | Default | Description |
|---|---|---|---|
| `tag` | String | `""` | Ending identifier |

| Port | Direction | Count |
|---|---|---|
| Input | In | 1 |
| Output | Out | 0 |

**Canvas display:** Shows `tag: value` if tag is set.

**Common tags:** `good_ending`, `bad_ending`, `continue`, `death`, `quest_complete`

**Export format:**

```json
{
  "id": "end_1",
  "type": "end",
  "tag": "good_ending"
}
```
