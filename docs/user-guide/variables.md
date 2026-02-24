# Variables

Variables let you track game state within your dialogue graphs. Use them with Condition nodes for branching and Event nodes for state changes.

## Managing Variables

Variables are managed in the **left panel** under the **Variables** section.

### Adding a Variable

Click **+ Add** in the Variables section. A new variable is created with:

- **Name**: Auto-generated (e.g., `var_1`, `var_2`)
- **Type**: `Bool` (default)
- **Default value**: `false`

### Editing a Variable

Each variable has three fields:

| Field | Description |
|---|---|
| **Name** | Unique identifier used in Condition and Event nodes |
| **Type** | Dropdown: `Bool`, `Int`, `Float`, `Text` |
| **Default Value** | The starting value when the dialogue begins |

!!! warning
    Changing a variable's type resets its default value. Make sure to update any Condition or Event nodes that reference it.

### Removing a Variable

Click the delete button next to a variable to remove it. This does not automatically update nodes that reference it — check your Condition and Event nodes after removing variables.

## Variable Types

| Type | Values | Example |
|---|---|---|
| `Bool` | `true` / `false` | `has_key = true` |
| `Int` | Whole numbers | `gold = 500` |
| `Float` | Decimal numbers | `reputation = 0.75` |
| `Text` | Strings | `player_name = "Hero"` |

## Using Variables in Condition Nodes

Condition nodes evaluate a variable against a value:

```
variable operator value → True or False
```

**Example**: `gold >= 100`

- If the player has 100+ gold → follow the **True** output
- Otherwise → follow the **False** output

Available operators: `==`, `!=`, `>`, `<`, `>=`, `<=`, `contains`

!!! tip
    The `contains` operator works with Text variables — it checks if the variable's string value contains the comparison text.

## Using Variables in Event Nodes

Event nodes can modify variable values using the `SetVariable` action type:

- **Key**: The variable name
- **Value**: The new value to assign

**Example**: An Event node with action `SetVariable`, key `has_key`, value `true` — sets the `has_key` variable to `true` when the dialogue passes through this node.

## Using Variables in Choice Conditions

Choice options can have visibility conditions. A choice is only shown to the player if its condition evaluates to true.

**Example**: A choice "Use the key" with condition `has_key == true` only appears if the player has acquired the key.

## Text Interpolation

Dialogue text and choice text support inline variable substitution using `{...}` syntax. This works in playtest mode and is preserved as-is in exported JSON for your game engine to handle.

### Basic Substitution

Use `{variable_name}` to insert a variable's current value:

```
Hello {player_name}, you have {gold} gold.
```

### Math Expressions

Use arithmetic operators inside `{...}`:

```
You need {100 - gold} more gold.
Total cost: {price * quantity} gold.
```

Supported operators: `+`, `-`, `*`, `/`, `%`

### Comparisons

Comparisons evaluate to `true` or `false`:

```
Rich: {gold >= 100}
```

Supported: `==`, `!=`, `>`, `<`, `>=`, `<=`

### Boolean Operators

Combine conditions with `&&` (and) and `||` (or), negate with `!`:

```
{has_key && level > 5}
{!is_hidden || has_detect}
```

### Inline Conditionals

Show different text based on a condition:

```
{if has_key}You unlock the door.{else}The door is locked.{/if}
{if gold >= 50}You can afford it.{/if}
```

The `{else}` block is optional. Without it, nothing is shown when the condition is false.

### Operator Precedence

From lowest to highest:

1. `||` (or)
2. `&&` (and)
3. `==`, `!=` (equality)
4. `>`, `<`, `>=`, `<=` (comparison)
5. `+`, `-` (additive)
6. `*`, `/`, `%` (multiplicative)
7. `!`, `-` (unary not, negation)

Use parentheses to override: `{(a + b) * c}`

### In Playtest vs Export

| Context | Behavior |
|---|---|
| **Playtest mode** | Expressions are evaluated and text is interpolated in real time |
| **Exported JSON** | `{...}` syntax is preserved as-is for game engines to evaluate |

!!! tip
    A hint below the dialogue text editor reminds you of the `{variable}` syntax.

## Variables in Export

Variables are included in the exported JSON:

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
    }
  ]
}
```

Your game engine reads these to initialize dialogue state before playback.
