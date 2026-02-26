# Dock & Panel Layout

TaleNode uses a dockable panel system that lets you arrange your workspace to fit your workflow.

## Available Panels

TaleNode has **20 panels**, each accessible from the **View** menu:

| Panel | Description |
|---|---|
| **Canvas** | Main node graph workspace |
| **Project** | Characters, variables, and groups |
| **Inspector** | Edit properties of the selected node |
| **Script Editor** | Yarn-style text view of the graph |
| **Validation** | Real-time error and warning list |
| **Playtest** | Walk through dialogue interactively |
| **Comments** | Per-node comments and review status |
| **Bookmarks** | Tag-based node navigation |
| **Analytics** | Graph statistics and path analysis |
| **Version History** | Snapshot versioning with diff |
| **Templates** | Reusable node pattern library |
| **Localization** | Multi-language string management |
| **Voice Generation** | ElevenLabs AI voice synthesis |
| **Collaboration** | Real-time multiplayer editing |
| **Barks** | Ambient dialogue per character |
| **Quests** | Quest and objective tracking |
| **Extensions** | Plugin management |
| **Timeline** | Cutscene/event sequencer |
| **World Database** | Items, locations, and lore entities |
| **AI Writing** | AI-powered dialogue suggestions |

## Opening and Closing Panels

- **Open**: Use the **View** menu and click on a panel name to toggle it on
- **Close**: Click the **X** button on a panel tab, or toggle it off from the View menu

## Arranging Panels

Panels can be rearranged by dragging their tabs:

- **Drag a tab** to move it to a different position in the dock
- **Drop between panels** to create a new split (left/right or top/bottom)
- **Drop on top of another panel** to create a tabbed group

## Auto-Focus Inspector

When you select a single node on the canvas, the **Inspector** panel automatically receives focus and scrolls to the top. This lets you quickly edit node properties without manually switching tabs.

## Reset Layout

If your layout becomes cluttered or panels are lost off-screen:

- **Menu**: View > Reset Layout

This restores the default panel arrangement:

- **Center**: Canvas (main workspace)
- **Left**: Project panel (characters, variables, groups)
- **Right**: Inspector panel
- **Bottom**: Validation panel

## Layout Persistence

Your panel layout is saved automatically as part of the `.talenode` project file. When you reopen a project, all panels are restored to their previous positions and sizes.

!!! tip
    Different projects can have different layouts — a writing-focused project might emphasize the Script Editor, while a testing-focused project might keep Playtest and Validation prominent.
