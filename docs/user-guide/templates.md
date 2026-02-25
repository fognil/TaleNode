# Templates

Templates let you save and reuse common node patterns. Save a group of connected nodes as a template, then insert it into any graph with a single click.

## Saving a Template

1. Select the nodes you want to save (box select or Ctrl+click)
2. Open the Template Library panel from the right-side panel tabs
3. Type a name for the template
4. Click **Save Selection**

The template captures:

- All selected nodes with their types and properties
- Connections between selected nodes (connections to nodes outside the selection are excluded)
- Node positions relative to the bounding box origin

## Inserting a Template

1. Open the Template Library panel
2. Find the template you want
3. Click **Insert**

The template nodes are added to the canvas at the current view center. All internal connections are recreated. New UUIDs are generated for the inserted nodes.

## Managing Templates

### Builtin Templates

TaleNode includes builtin templates for common patterns (marked with `[builtin]`). These cannot be deleted.

### User Templates

Templates you save are listed with their node count. Click **Delete** to remove a user-created template.

## Template List

Each template shows:

- **Name** — The template name
- **Node count** — How many nodes are in the template
- **Description** — Optional description text
- **Insert / Delete buttons** — Actions

## Tips

!!! tip
    Create templates for patterns you use frequently — like a "Yes/No choice with condition" or a "shop dialogue loop".

!!! tip
    Templates are stored in the `.talenode` project file. They travel with the project, so team members sharing the file get the same templates.
