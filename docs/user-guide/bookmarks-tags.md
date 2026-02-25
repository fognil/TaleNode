# Bookmarks & Tags

Tags let you organize and quickly navigate to nodes in your dialogue graph. The Bookmarks panel provides a tag-based filtering system.

## Adding Tags to Nodes

1. Select a node on the canvas
2. In the [Inspector](inspector.md), scroll to the **Tags** section
3. Type a tag name and click **+** to add it
4. Tags appear as colored labels with **x** buttons to remove them

You can also add tags from the Bookmarks panel:

1. Select a node on the canvas
2. Open the Bookmarks panel
3. Type a tag in the input field and click **Add**

## Bookmarks Panel

The Bookmarks panel shows all tagged nodes with filtering capabilities:

### Tag Cloud

At the top, all unique tags are displayed as selectable buttons. Click a tag to filter the list to only nodes with that tag. Click again to deselect.

### Node List

Below the tag cloud, matching nodes are listed alphabetically. Each entry shows:

- **Node name** — Click to navigate to the node on the canvas
- **Tags** — Colored labels for each tag, with **x** buttons to remove

### Quick Navigation

Click any node name in the list to select it on the canvas and pan to its location.

## Tag Suggestions

Common tag patterns:

| Tag | Use Case |
|---|---|
| `important` | Key story moments |
| `wip` | Work in progress |
| `bug` | Known issues |
| `tutorial` | Tutorial dialogue |
| `side_quest` | Optional content |
| `boss` | Boss encounter dialogue |

## Tips

!!! tip
    Tags are saved in the `.talenode` project file and not included in exports. Use them freely for organization.

!!! tip
    Combine tags with [review statuses](comments-review.md) for a complete content management workflow.
