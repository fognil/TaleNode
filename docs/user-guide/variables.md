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

You can also use the keyword aliases `and`, `or`, `not` for readability:

```
{has_key and level > 5}
{not is_hidden or has_detect}
```

### Ternary Expressions

Use `condition ? value_if_true : value_if_false` inline:

```
{gold > 100 ? "rich" : "poor"}
{x < 0 ? -x : x}
```

### Built-in Functions

Functions can be used inside `{...}` expressions:

| Function | Description | Example |
|---|---|---|
| `abs(x)` | Absolute value | `abs(-5)` → `5` |
| `round(x)` | Round to nearest integer | `round(3.7)` → `4` |
| `floor(x)` | Round down | `floor(3.9)` → `3` |
| `ceil(x)` | Round up | `ceil(3.1)` → `4` |
| `min(a, b)` | Smaller value | `min(5, 3)` → `3` |
| `max(a, b)` | Larger value | `max(5, 3)` → `5` |
| `clamp(x, lo, hi)` | Clamp to range | `clamp(hp, 0, 100)` |
| `pow(base, exp)` | Exponentiation | `pow(2, 10)` → `1024` |
| `random(lo, hi)` | Random integer in range | `random(1, 6)` |
| `len(s)` | String length | `len(name)` |
| `upper(s)` | Uppercase | `upper("hello")` → `"HELLO"` |
| `lower(s)` | Lowercase | `lower("HELLO")` → `"hello"` |
| `trim(s)` | Strip whitespace | `trim(" hi ")` → `"hi"` |
| `contains(s, sub)` | Substring check | `contains("hello", "ell")` → `true` |
| `starts_with(s, pre)` | Prefix check | `starts_with("hello", "he")` → `true` |
| `ends_with(s, suf)` | Suffix check | `ends_with("hello", "lo")` → `true` |
| `replace(s, from, to)` | Replace substring | `replace("hello", "l", "r")` → `"herro"` |
| `substr(s, start, len)` | Extract substring | `substr("hello", 1, 3)` → `"ell"` |
| `str(x)` | Convert to text | `str(42)` → `"42"` |
| `int(x)` | Convert to integer | `int(3.9)` → `3` |
| `float(x)` | Convert to float | `float(5)` → `5.0` |
| `visits()` | Visit count for current node | `visits()` → `3` |
| `visits(id)` | Visit count for specific node | `visits("market")` → `2` |
| `visited()` | Whether current node was visited | `visited()` → `true` |
| `visited(id)` | Whether specific node was visited | `visited("market")` → `true` |

Functions can be nested: `abs(min(-5, 3))` → `5`

!!! tip
    Use `visits()` and `visited()` to vary dialogue based on how many times the player has been to a node:
    ```
    {if visits() == 1}Welcome, stranger.{else}Back again, I see.{/if}
    {visits("market") > 3 ? "You're a regular!" : "New around here?"}
    ```

### Inline Conditionals

Show different text based on a condition:

```
{if has_key}You unlock the door.{else}The door is locked.{/if}
{if gold >= 50}You can afford it.{/if}
```

The `{else}` block is optional. Without it, nothing is shown when the condition is false.

#### Elseif Chains

Use `{elseif}` for multi-branch conditionals:

```
{if gold >= 100}You're rich!{elseif gold >= 50}You're getting by.{else}You're broke.{/if}
```

You can chain as many `{elseif}` branches as needed. The first matching condition wins.

### Dynamic Text Variations

These blocks let you vary text across repeated visits to the same node:

| Syntax | Behavior | Example |
|---|---|---|
| `{~a\|b\|c}` | **Sequence** — shows items in order, sticks on last | First visit: "a", second: "b", third+: "c" |
| `{&a\|b\|c}` | **Cycle** — loops through items repeatedly | "a" → "b" → "c" → "a" → ... |
| `{!a\|b\|c}` | **Shuffle** — random pick each time | Random: "a", "c", "b", ... |
| `{?a\|b\|c}` | **Once-only** — shows each item once, then empty | "a" → "b" → "c" → "" → "" |

Items are separated by `|` and can contain nested expressions:

```
{~You have {gold} gold|You still have {gold} gold}
{&Good morning|Good afternoon|Good evening}
{?This is your first visit.|This is your second visit.|This is your last unique greeting.}
```

### Inline Commands

Use `<<...>>` syntax for commands that execute during playtest but produce no visible text:

#### Set Command

Modify variables inline within dialogue text:

```
You found a gem!<<set gold += 50>> Now you have {gold} gold.
<<set hp = hp + 20>>HP is now {hp}.
<<set flag true>>
```

Supported assignment forms:

| Form | Example | Description |
|---|---|---|
| `=` | `<<set gold = 100>>` | Direct assignment |
| `+=` | `<<set gold += 25>>` | Add and assign |
| `-=` | `<<set hp -= 30>>` | Subtract and assign |
| `*=` | `<<set score *= 2>>` | Multiply and assign |
| `/=` | `<<set score /= 2>>` | Divide and assign |
| `%=` | `<<set x %= 3>>` | Modulo and assign |
| *(no `=`)* | `<<set flag true>>` | Assignment without `=` |

The right-hand side can be any expression: `<<set gold += bonus * 2>>`

#### Generic Commands

Other `<<...>>` commands are preserved as markers for game engines:

```
Hello<<wait 2>> world
<<play_sound "door_open">>
```

These produce no output during playtest but are available in the raw text for engines to parse.

### String Literals

String literals in expressions use double quotes with escape sequences:

| Escape | Result |
|---|---|
| `\n` | Newline |
| `\t` | Tab |
| `\\` | Backslash |
| `\"` | Double quote |

### Operator Precedence

From lowest to highest:

1. `? :` (ternary)
2. `||` (or)
3. `&&` (and)
4. `==`, `!=` (equality)
5. `>`, `<`, `>=`, `<=` (comparison)
6. `+`, `-` (additive)
7. `*`, `/`, `%` (multiplicative)
8. `!`, `-` (unary not, negation)
9. Function calls, parentheses

Use parentheses to override: `{(a + b) * c}`

### In Playtest vs Export

| Context | Behavior |
|---|---|
| **Playtest mode** | Expressions are evaluated and text is interpolated in real time |
| **Exported JSON** | `{...}` and `<<...>>` syntax is preserved as-is for game engines to evaluate |

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
