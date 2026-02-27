# Canvas & Navigation

The canvas is the main workspace where you build your dialogue graph. It's an infinite 2D surface with pan, zoom, and a grid background.

## Panning

Move around the canvas using any of these methods:

| Method | Action |
|---|---|
| Middle mouse drag | Hold middle button and drag |
| Space + left mouse drag | Hold Space, then click and drag |
| Scroll wheel | Scroll to pan vertically and horizontally |

## Zooming

| Method | Action |
|---|---|
| ++ctrl++ + scroll wheel | Zoom in/out at cursor position |
| Trackpad pinch (macOS) | Pinch to zoom |

- **Zoom range**: 25% to 400%
- The current zoom level is displayed in the status bar at the bottom.

## Grid

The canvas displays a grid to help align nodes:

- **Grid spacing**: 40px (canvas units)
- **Major gridlines**: Every 5th line is drawn slightly brighter
- The grid fades out automatically when zoomed out far enough (below 4px effective spacing)

## Coordinate System

TaleNode uses two coordinate systems internally:

- **Screen coordinates**: Pixel position on your monitor
- **Canvas coordinates**: Logical position on the infinite canvas

The conversion is: `canvas_pos = (screen_pos - pan_offset) / zoom`

All node positions are stored in canvas coordinates, so they stay consistent regardless of how you pan or zoom.

## Selecting Nodes

| Action | Result |
|---|---|
| Left-click a node | Select that node (deselects others) |
| Left-click empty canvas | Deselect all nodes |
| Left-click drag on empty canvas | Box selection — selects all nodes within the rectangle |
| ++ctrl+a++ | Select all nodes |

Selected nodes are highlighted with a bright border.

## Moving Nodes

Click and drag a selected node to move it. If multiple nodes are selected, they all move together.

!!! tip
    Node movement supports undo — the position change is recorded when you release the mouse button.

## Deleting Nodes

Select one or more nodes and press ++delete++ or ++backspace++. All connections to/from deleted nodes are automatically removed.

## Duplicating Nodes

Select nodes and press ++ctrl+d++. Duplicated nodes appear offset by 30px down and to the right. Connections are not duplicated — only the node structure is copied.

## Minimap

A minimap is displayed in the bottom-right corner of the canvas (160x160 pixels). It shows:

- All nodes as small colored rectangles matching their type color
- The current viewport as a white rectangle outline

Click or drag on the minimap to quickly navigate to a different part of your graph.

## Search

Press ++ctrl+f++ to open the search bar. Search finds matches across:

- Node titles and types
- Dialogue text and speaker names
- Choice option text and prompts
- Condition variable names
- Event action keys
- End tags

Navigate through results with the **<** / **>** buttons or press ++enter++ to cycle. The canvas auto-centers on each result. Press ++escape++ to close search.

## Search & Replace

Press ++ctrl+h++ (or ++cmd+shift+h++ on macOS) to open the search bar with the replace row, or click **Replace** in the search bar to toggle it.

| Button | Action |
|---|---|
| **Replace** | Replace the match in the currently focused node |
| **Replace All** | Replace all matches across all matching nodes |

Replace works on the same text fields as search: dialogue text, speaker names, choice text, prompts, condition variable names, event action keys, and end tags. The search is **case-insensitive**.

!!! tip
    Replace operations support undo — press ++ctrl+z++ to revert any replacement.

## Collapsed Nodes

Nodes can be collapsed to show only their header, hiding the body and ports. This reduces visual clutter on busy canvases.

- **Collapse**: Click the triangle indicator (▾) on the left side of the node header
- **Expand**: Click the triangle again (▸) to restore the full node

When collapsed:

- The node shrinks to header-only height with full rounded corners
- Body content (text preview, action list) and ports are hidden
- Existing wires connect to the center of the header edges instead of individual ports
- Port hit-testing is disabled — you cannot start or end new connections on a collapsed node

!!! tip
    Collapsing supports undo — press ++ctrl+z++ to revert a collapse/expand toggle.

## Canvas Filter

Press ++ctrl+shift+f++ to toggle the **filter bar**, which lets you focus the canvas on a subset of nodes by tag or type.

- **Tags dropdown**: Select one or more tags — only nodes with at least one matching tag are shown
- **Type checkboxes**: Check node types (Dialogue, Choice, Condition, Event, Random) to show only those types
- **Clear button**: Reset all filters and show everything

When a filter is active:

- Nodes that don't match the filter are hidden from the canvas
- Connections where both endpoints are filtered out are also hidden
- **Start**, **End**, and **SubGraph** nodes are always visible (structural nodes)

!!! tip
    Combine tag and type filters to quickly isolate specific parts of a large graph. For example, filter by tag "Act 2" and type "Choice" to see only choice nodes in Act 2.

## Zoom to Fit

Press ++f++ to automatically zoom and pan the canvas so that **all nodes** are visible on screen. This is useful after importing a graph or when you've lost track of nodes placed far apart.

## Level of Detail

When you zoom out on a large graph, TaleNode automatically reduces rendering detail for better performance:

| Zoom Level | Detail |
|---|---|
| **50% and above** | Full detail — all text, ports, labels, body content |
| **25%–50%** | Medium — header, body rectangle, port circles, no text |
| **Below 25%** | Low — single colored rectangle per node, no text or ports |

This happens automatically — no configuration needed. Port hover detection and node tooltips are also disabled at medium/low zoom levels since those details aren't visible.

## Performance at Scale

TaleNode is optimized for graphs with 1000+ nodes:

- **Viewport culling**: Only nodes and connections visible on screen are rendered
- **Spatial indexing**: Click and hover hit-testing uses a grid index instead of checking every node
- **Adaptive wires**: Connection curves use fewer segments when they're small on screen
- **Minimap caching**: The minimap caches node bounds and renders dots instead of rectangles above 500 nodes

These optimizations are automatic and require no user configuration.

## Background Color

The canvas background is dark gray (`rgb(30, 30, 30)`) by default. Switch to light theme via **View > Light/Dark Theme**.
