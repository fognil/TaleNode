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

## Tips

!!! tip
    Use groups to label sections of a long dialogue: "Introduction", "Quest Offer", "Reward", etc. This makes complex graphs easier to navigate.

!!! tip
    Groups are saved in the `.talenode` project file but are **not** included in the exported JSON — they're purely for editor organization.
