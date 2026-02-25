# SubGraph & Nested Dialogues

SubGraph nodes let you organize complex conversations into reusable nested dialogues. Each SubGraph contains its own complete dialogue graph inside a single node.

## Creating a SubGraph

1. Right-click the canvas and select **Add Node > SubGraph**
2. The node appears with a default name and an empty child graph (with a Start node)
3. Set the **Name** in the [Inspector](inspector.md)

## Entering a SubGraph

**Double-click** the SubGraph node on the canvas to enter it. When inside a sub-graph:

- A **breadcrumb bar** appears at the top of the canvas showing the navigation path (e.g., `Root > Side Quest`)
- You can edit the nested graph exactly like the main graph — add nodes, create connections, etc.
- Press ++escape++ or click the parent breadcrumb to exit back to the parent graph

## How It Works

- Each sub-graph has its own **Start node** (created automatically when the SubGraph is created)
- When the dialogue flow reaches a SubGraph node, it enters the nested graph at its Start node
- When the nested graph reaches an End node, flow returns to the SubGraph node's output port and continues in the parent graph

## Nesting

SubGraphs can be nested inside other SubGraphs. The breadcrumb bar shows the full navigation path. There is no hard limit on nesting depth, but deep nesting can make graphs harder to follow.

## Canvas Display

On the parent canvas, the SubGraph node shows:

- The sub-graph name in the header
- The count of child nodes and connections

## Export

When exporting, SubGraph contents are flattened into the node list with their connections preserved. The sub-graph structure is included in the export for engines that support hierarchical dialogue.

## Tips

!!! tip
    Use SubGraphs to break large dialogues into manageable chunks. A quest with 50+ nodes is much easier to manage as several SubGraphs.

!!! tip
    Give SubGraphs descriptive names — they appear in the node header and make the parent graph much easier to read at a glance.
