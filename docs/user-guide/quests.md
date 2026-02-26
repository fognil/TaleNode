# Quests

TaleNode includes a quest/journal system for tracking multi-step objectives that span across dialogue trees. Define quests with objectives, then trigger them from Event nodes during dialogue.

## Managing Quests

Open the **Quests** panel via **View > Quests** in the menu bar. The panel appears as a dockable tab.

### Adding a Quest

Click **+ Add Quest** to create a new quest with:

| Field | Default | Description |
|---|---|---|
| **Name** | `"New Quest"` | Quest title shown to the player |
| **Description** | `""` | Optional quest summary |
| **Status** | `NotStarted` | Current state of the quest |

### Editing a Quest

Each quest has a collapsible section in the Quests panel:

- **Name**: Click to edit the quest title
- **Description**: Click to edit the quest description

### Adding Objectives

Click **+ Objective** within a quest to add an objective. Each objective has:

| Field | Type | Description |
|---|---|---|
| **Text** | String | Objective description (e.g., "Find the key") |
| **Optional** | Bool | Whether this objective is required for quest completion |

### Removing Quests and Objectives

- Click **X** next to a quest to remove it and all its objectives
- Click **X** next to an objective to remove just that objective

## Quest Event Actions

Use **Event nodes** to control quest progression during dialogue. Three action types are available:

| Action | Description |
|---|---|
| `start_quest` | Begins a quest (sets status to InProgress) |
| `complete_objective` | Marks a specific objective as complete |
| `fail_quest` | Marks a quest as Failed |

Set the **Key** field to the quest name (or `QuestName.ObjectiveName` for objectives) and configure accordingly.

## Quest Status

Quests have four possible states:

| Status | Description |
|---|---|
| **NotStarted** | Quest has not been triggered yet |
| **InProgress** | Quest is active, objectives may be pending |
| **Completed** | All required objectives are done |
| **Failed** | Quest was explicitly failed via an event |

## Export

Quests are included in the JSON export:

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

## Markdown Export

Quests also appear in Markdown exports as a checklist:

```markdown
## Quests

### Find the Lost Artifact

The elder has asked you to recover the ancient artifact.

- [ ] Talk to the elder
- [ ] Search the ruins
- [ ] *(optional)* Find the secret passage
```

## Runtime Integration

Your game engine should:

1. Load the quests array and initialize all quests as `NotStarted`
2. When a `start_quest` event fires, find the quest by name and set its status to `InProgress`
3. When a `complete_objective` event fires, mark the matching objective as complete
4. When all non-optional objectives are complete, set the quest status to `Completed`
5. When a `fail_quest` event fires, set the quest status to `Failed`
6. Display active quests and objectives in your game's journal UI

## Tips

!!! tip
    Mark side content as optional objectives. This lets completionists track extra goals without blocking main quest progression.

!!! tip
    Use descriptive quest names — they appear in Event node summaries on the canvas, making it easy to see quest flow at a glance.

!!! tip
    Combine quests with Condition nodes to gate dialogue based on quest progress. For example, check if a quest is InProgress before showing certain dialogue options.
