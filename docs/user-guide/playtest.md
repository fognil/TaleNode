# Playtest Mode

Playtest mode lets you walk through your dialogue directly in the editor, testing branching paths and verifying flow without launching your game.

## Opening Playtest

Toggle the playtest panel from **View > Playtest Panel**. A panel appears at the bottom or side of the editor.

## Controls

| Button | Action |
|---|---|
| **Start** | Begin playtest from the Start node |
| **Stop** | End the current playtest session |
| **Restart** | Stop and start again from the beginning |

## How It Works

### Automatic Nodes

These node types are passed through automatically:

- **Start**: Immediately advances to the next node.
- **Event**: Executes `SetVariable` actions, updating runtime variables, then advances.
- **Condition**: Evaluates the condition against runtime variables and takes the **True** or **False** branch accordingly.
- **Random**: Uses weighted random selection to pick a branch.

### Interactive Nodes

- **Dialogue**: Displays the speaker, interpolated text, and emotion. Click **Continue >>** to advance.
- **Choice**: Displays the prompt and available options as clickable buttons. Choices with conditions that evaluate to false are shown disabled. Click an available option to follow that branch.
- **End**: Displays "Dialogue ended: [tag]" and stops.

## Runtime Variables

When you start a playtest, all variables defined in your project are initialized to their default values. As the playtest progresses:

- **Event nodes** with `SetVariable` actions update variable values in real time
- **Condition nodes** evaluate against current variable values
- **Choice conditions** are evaluated to enable/disable choices

A collapsible **Variables** section at the bottom of the playtest panel shows all current variable values, so you can monitor state changes as you play through.

## Text Interpolation

Dialogue text and choice text support a rich inline scripting syntax. During playtest, these are evaluated and replaced with their values:

```
Hello {player_name}, you have {gold} gold.
You need {100 - gold} more gold.
{if gold >= 100}Rich!{elseif gold >= 50}Okay.{else}Broke.{/if}
{gold > 100 ? "wealthy" : "modest"}
{~First visit|Welcome back|Old friend}
```

Inline commands also execute during playtest:

```
You found treasure!<<set gold += 50>> Now you have {gold} gold.
```

See [Variables — Text Interpolation](variables.md#text-interpolation) for the full syntax reference including functions, dynamic text variations, inline commands, and operator precedence.

!!! note
    Interpolation only happens during playtest preview. The exported JSON preserves `{...}` and `<<...>>` syntax as-is for your game engine to evaluate at runtime.

## Dialogue Log

A scrollable log shows all dialogue entries you've passed through during the session. This is useful for reviewing the full conversation after testing.

## SubGraph Traversal

SubGraph nodes are handled automatically during playtest:

1. When the playtest reaches a SubGraph node, it **enters the child graph** and finds the child's Start node
2. The playtest runs through the child graph normally (dialogue, choices, events, conditions)
3. When the child graph reaches an End node, the playtest **exits back** to the parent graph and continues from the SubGraph node's output
4. Nested SubGraphs work the same way — the playtest maintains a stack and can go multiple levels deep

## Checkpoints

You can save and restore playtest state at any point during a session:

- **Save checkpoint**: Captures the current node, all variable values, and dialogue log
- **Load checkpoint**: Restores a previously saved checkpoint and continues from that point
- **Maximum**: Up to 20 checkpoints can be stored per session

Checkpoints are useful for testing different branches from the same decision point without replaying the entire dialogue from the start.

## Playtest Glow

While playtesting, the currently active node displays a **pulsing green glow** on the canvas. This animated border makes it easy to spot which node the playtest is currently on, even in a large graph.

## Canvas Sync

While playtesting, the currently active node is **selected and highlighted** on the canvas with the playtest glow effect. The canvas does not auto-pan, but you can see which node is active by its green pulsing border.

## Tips

!!! tip
    Use playtest mode after wiring up a new branch to verify it flows correctly before exporting.

!!! tip
    Check the dialogue log to review the full conversation path you just tested.

!!! tip
    Watch the Variables panel to verify that Event nodes are updating state correctly and Condition nodes are branching as expected.
