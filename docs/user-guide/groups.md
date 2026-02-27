# Node Groups

Groups let you visually organize related nodes by drawing a colored background behind them. Use groups to separate quest stages, conversation branches, or any logical section of your graph.

## Creating a Group

1. **Select** two or more nodes (box select or ++ctrl++-click).
2. **Right-click** on the canvas.
3. Select **Group Selected** from the context menu.

A colored rectangle appears behind the selected nodes with a label.

## Managing Groups

Groups appear in the **left panel** under the **Groups** section (only visible when at least one group exists).

Each group has:

| Field | Description |
|---|---|
| **Name** | Editable label displayed on the canvas |
| **Color** | RGB color picker for the background rectangle |

Click the **delete** button to remove a group (nodes inside are not affected).

## Ungrouping

1. Select one or more nodes that belong to a group.
2. Right-click and select **Ungroup**.

This removes the group but keeps all nodes in place.

## Visual Appearance

- Groups render as semi-transparent colored rectangles **below** connections and nodes.
- The group label is displayed at the top of the rectangle.
- Group bounds are calculated from the positions of member nodes.

## Collapsing Groups

Groups can be collapsed to hide all their member nodes behind a single compact header. This is useful for hiding completed sections or reducing clutter on large graphs.

- **Collapse**: Right-click on the canvas inside a group and select **Collapse Group**
- **Expand**: Right-click the collapsed group and select **Expand Group**

When collapsed:

- All member nodes and their internal connections are hidden
- The group renders as a compact header showing the group name and node count (e.g. "Quest Offer (12 nodes)")
- Connections from outside the group to hidden nodes are drawn with reduced opacity
- The group uses its configured color as the header background

!!! tip
    Collapsing a group with 50+ nodes can significantly reduce visual clutter and improve canvas responsiveness.

## Tips

!!! tip
    Use groups to label sections of a long dialogue: "Introduction", "Quest Offer", "Reward", etc. This makes complex graphs easier to navigate.

!!! tip
    Groups are saved in the `.talenode` project file but are **not** included in the exported JSON — they're purely for editor organization.
