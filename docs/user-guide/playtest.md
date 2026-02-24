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

Dialogue text and choice text support inline expressions using `{...}` syntax. During playtest, these are evaluated and replaced with their values:

```
Hello {player_name}, you have {gold} gold.
You need {100 - gold} more gold.
{if has_key}You unlock the door.{else}The door is locked.{/if}
```

See [Variables — Text Interpolation](variables.md#text-interpolation) for full syntax details.

!!! note
    Interpolation only happens during playtest preview. The exported JSON preserves `{...}` syntax as-is for your game engine to evaluate at runtime.

## Dialogue Log

A scrollable log shows all dialogue entries you've passed through during the session. This is useful for reviewing the full conversation after testing.

## Canvas Sync

While playtesting, the currently active node is **selected and highlighted** on the canvas. The canvas does not auto-pan, but you can see which node is active by its selection highlight.

## Tips

!!! tip
    Use playtest mode after wiring up a new branch to verify it flows correctly before exporting.

!!! tip
    Check the dialogue log to review the full conversation path you just tested.

!!! tip
    Watch the Variables panel to verify that Event nodes are updating state correctly and Condition nodes are branching as expected.
