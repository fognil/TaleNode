# Version History

TaleNode tracks version snapshots of your dialogue graph, letting you save checkpoints, compare changes, and restore previous states.

## Saving a Version

1. Open the Version History panel from the right-side panel tabs
2. Type a description in the text field at the bottom (e.g., "Added quest branching")
3. Press ++enter++ or click **Save Version**

Each version captures a complete snapshot of the current graph state.

## Version List

Saved versions are displayed newest-first, showing:

- **Version number** — Auto-incremented (e.g., #1, #2, #3)
- **Timestamp** — When the version was saved
- **Description** — Your description text
- **Restore button** — Restore this version

## Restoring a Version

Click **Restore** on any version to revert the graph to that state. A **confirmation dialog** will appear before restoring, since this replaces the current graph.

!!! warning
    Restoring a version replaces your current graph. Save a new version first if you want to keep your current state.

## Comparing Versions

1. Check the boxes next to two versions in the list
2. Click **Compare Selected**

The diff summary shows:

| Change Type | Display |
|---|---|
| Added nodes | Green `+N nodes` |
| Removed nodes | Red `-N nodes` |
| Modified nodes | Yellow `~N modified` |
| Added connections | Green `+N connections` |
| Removed connections | Red `-N connections` |
| Added variables | Green `+var:name` |
| Removed variables | Red `-var:name` |
| Added characters | Green `+char:name` |
| Removed characters | Red `-char:name` |

## Tips

!!! tip
    Save versions before making major structural changes. It's much easier to compare and revert than relying on undo history alone.

!!! tip
    Version snapshots are stored in a separate `.talenode.versions` sidecar file alongside your project. This keeps the main `.talenode` file small and git-friendly. If you use version control, you can optionally add `*.talenode.versions` to `.gitignore` to exclude snapshots from your repository.
