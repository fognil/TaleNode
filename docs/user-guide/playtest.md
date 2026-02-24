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
- **Event**: Logs the number of actions and advances. Actions are simulated (not actually executed).
- **Condition**: Evaluates and takes the **True** branch by default (simulated).
- **Random**: Uses weighted random selection to pick a branch.

### Interactive Nodes

- **Dialogue**: Displays the speaker, text, and emotion. Click **Continue >>** to advance.
- **Choice**: Displays the prompt and all options as clickable buttons. Click an option to follow that branch.
- **End**: Displays "Dialogue ended: [tag]" and stops.

## Dialogue Log

A scrollable log shows all dialogue entries you've passed through during the session. This is useful for reviewing the full conversation after testing.

## Canvas Sync

While playtesting, the currently active node is **selected and highlighted** on the canvas. The canvas does not auto-pan, but you can see which node is active by its selection highlight.

## Limitations

!!! note
    Playtest mode is a simulation for testing flow, not a full game runtime:

    - **Condition nodes** always take the True branch (no actual variable evaluation)
    - **Event nodes** log actions but don't modify state
    - **Random nodes** use a simple pseudo-random based on system time
    - **Choice conditions** are not evaluated — all choices are shown

## Tips

!!! tip
    Use playtest mode after wiring up a new branch to verify it flows correctly before exporting.

!!! tip
    Check the dialogue log to review the full conversation path you just tested.
