# Comments & Review

TaleNode includes a review workflow for teams working on dialogue content. Each node can have comments and a review status, visible in the Comments panel.

## Review Status

Every node has a review status, set in the [Inspector](inspector.md):

| Status | Color | Description |
|---|---|---|
| **Draft** | Gray | Default. Work in progress. |
| **Needs Review** | Yellow | Ready for someone to review. |
| **Approved** | Green | Reviewed and accepted. |

Review statuses appear as colored badges on nodes in the canvas, making it easy to see progress at a glance.

## Comments Panel

Open the Comments panel from the right-side panel tabs. It shows:

1. **Filter buttons** — Filter by review status: All, Draft, Needs Review, or Approved
2. **Node list** — All nodes matching the filter, sorted alphabetically
3. **Comment threads** — Each node shows its comments with delete buttons
4. **Add comment input** — Type a comment and press Enter or click Add

### Adding a Comment

1. Click a node name in the Comments panel (or select a node on the canvas)
2. Type your comment in the text field at the bottom
3. Press ++enter++ or click **Add**

### Deleting a Comment

Click the **X** button next to any comment to remove it.

### Navigating to a Node

Click any node name in the Comments panel to select it on the canvas and pan to its location.

## Workflow Example

1. Writer creates dialogue nodes (status: **Draft**)
2. Writer finishes a section and sets status to **Needs Review**
3. Reviewer opens Comments panel, filters by "Needs Review"
4. Reviewer adds comments to nodes that need changes
5. Writer addresses comments, sets status to **Approved**

## Tips

!!! tip
    Use the status filter to focus on nodes that need attention. Filter by "Needs Review" to see your review queue.

!!! tip
    Comments are saved in the `.talenode` project file. They are not included in the exported JSON.
