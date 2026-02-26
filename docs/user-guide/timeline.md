# Timeline / Cutscene Sequencer

TaleNode includes a timeline system for sequencing cutscene events alongside dialogue. Define camera moves, animations, audio cues, and waits in a named sequence that your game engine plays back.

## Overview

A timeline is an ordered list of steps. Each step has an action type (camera, animation, audio, etc.), action-specific parameters, and an optional delay before execution. Timelines are independent from the node graph — use them for scripted sequences, cutscenes, or any timed event chain.

## Managing Timelines

Open the **Timeline** panel via **View > Timeline** in the menu bar. The panel appears as a dockable tab.

### Adding a Timeline

Click **+ Add Timeline** to create a new timeline.

| Field | Type | Description |
|---|---|---|
| **Name** | String | Timeline name (e.g., "Intro Cutscene") |
| **Description** | String | Optional description |
| **Loop** | Bool | Whether the timeline loops after the last step |

### Adding Steps

Click **+ Step** within a timeline to add a new step (defaults to `Wait 1s`).

Each step has:

| Field | Description |
|---|---|
| **Action Type** | Dropdown: Dialogue, Camera, Animation, Audio, Wait, SetVariable, Custom |
| **Action Fields** | Type-specific parameters (see below) |
| **Delay** | Seconds to wait before executing this step (0 = immediate) |

### Action Types

| Action | Fields | Description |
|---|---|---|
| **Dialogue** | *(linked node)* | Reference a dialogue node to play |
| **Camera** | `target`, `duration` | Move camera to target over duration seconds |
| **Animation** | `target`, `clip` | Play animation clip on target object |
| **Audio** | `clip`, `volume` | Play audio clip at specified volume (0.0–1.0) |
| **Wait** | `seconds` | Pause the timeline for N seconds |
| **SetVariable** | `key`, `value` | Set a game variable during the sequence |
| **Custom** | `action_type`, `data` | Engine-specific custom action |

### Reordering Steps

Use the **^** (up) and **v** (down) buttons next to each step to reorder the sequence.

### Removing Steps and Timelines

- Click **X** next to a step to remove it
- Click **Delete Timeline** at the bottom of a timeline's section to remove the entire timeline

## Export

Timelines are included in the JSON export:

```json
{
  "timelines": [
    {
      "name": "Intro Cutscene",
      "description": "Opening sequence",
      "steps": [
        {
          "action": { "type": "camera", "target": "player", "duration": 2.0 }
        },
        {
          "action": { "type": "wait", "seconds": 1.0 },
          "delay": 0.5
        },
        {
          "action": { "type": "audio", "clip": "intro_music.ogg", "volume": 0.8 }
        },
        {
          "action": { "type": "dialogue", "node_id": null }
        }
      ],
      "loop_playback": false
    }
  ]
}
```

The `timelines` array is omitted when no timelines are defined. The `delay` field is omitted when zero. The `description` field is omitted when empty. The `loop_playback` field is omitted when false.

## Runtime Integration

Your game engine should:

1. Load the `timelines` array
2. For each timeline, iterate through steps in order
3. Apply the `delay` before executing each step's action
4. Dispatch the action to the appropriate system (camera controller, animation player, audio manager, etc.)
5. If `loop_playback` is true, restart from the first step after the last one completes

### Example Pseudocode

```
func play_timeline(timeline):
    for step in timeline.steps:
        if step.delay > 0:
            wait(step.delay)
        match step.action.type:
            "camera": move_camera(step.action.target, step.action.duration)
            "animation": play_anim(step.action.target, step.action.clip)
            "audio": play_sound(step.action.clip, step.action.volume)
            "wait": wait(step.action.seconds)
            "set_variable": set_var(step.action.key, step.action.value)
            "dialogue": show_dialogue(step.action.node_id)
            "custom": handle_custom(step.action)
    if timeline.loop_playback:
        play_timeline(timeline)
```

## Tips

!!! tip
    Use the `delay` field to stagger actions. A delay of 0.5 on a camera step means "wait 0.5s, then start the camera move."

!!! tip
    Combine timelines with Event nodes. Trigger a timeline from an event action, then resume dialogue after the cutscene completes.

!!! tip
    The Custom action type is a catch-all for engine-specific behavior. Store whatever data your engine needs in the `data` field.
