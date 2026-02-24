# Validation

TaleNode validates your dialogue graph in real time and reports errors and warnings so you can fix issues before exporting.

## Validation Panel

Toggle the validation panel from **View > Validation Panel**. The panel displays a list of issues found in the current graph.

The **status bar** at the bottom also shows a summary:

- **Green** "No issues" — graph is clean
- **Yellow** "N warning(s)" — only warnings
- **Red** "N error(s), N warning(s)" — errors present

Click the validation summary in the status bar to toggle the panel.

## Validation Checks

### Errors

| Check | Description |
|---|---|
| **No Start node** | The graph has no Start node. Every dialogue needs exactly one. |

### Warnings

| Check | Description |
|---|---|
| **Multiple Start nodes** | More than one Start node exists. Only one entry point is expected. |
| **Disconnected outputs** | A non-End node has output ports with no outgoing connections. |
| **Unreachable nodes** | A node is not reachable from any Start node (BFS traversal). |
| **Empty dialogue** | A Dialogue node has empty or whitespace-only text. |
| **Dead ends** | A non-End node has output ports but no outgoing connections. |

## Navigating to Issues

Click on any issue in the validation panel to:

1. **Select** the problematic node
2. **Center** the canvas on that node

This makes it easy to find and fix issues in large graphs.

## Common Fixes

!!! example "No Start node"
    Right-click the canvas and add a Start node. Connect it to the beginning of your dialogue.

!!! example "Disconnected outputs"
    Connect the node's output port to the next node in the flow, or add an End node if this is a terminal path.

!!! example "Unreachable nodes"
    This node isn't connected to the main graph. Either wire it in or delete it if it's unused.

!!! example "Empty dialogue"
    Select the Dialogue node and fill in the text field in the Inspector.

## Tips

!!! tip
    Fix all errors and warnings before exporting. While TaleNode will still export with warnings, your game engine may not handle disconnected or unreachable nodes gracefully.

!!! tip
    Validation runs every frame, so issues appear and disappear as you edit. There's no need to manually trigger a validation pass.
