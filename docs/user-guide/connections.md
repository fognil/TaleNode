# Connections & Wiring

Connections (wires) link nodes together to define the dialogue flow. They are drawn as smooth bezier curves between output and input ports.

## Creating a Connection

1. Hover over a **port** on any node (the small circles on the left/right edges).
2. Click and drag from the port. A yellow preview wire follows your cursor.
3. Release on a compatible port on another node.

You can drag in either direction — output-to-input or input-to-output. TaleNode automatically determines which is the source and which is the target.

## Connection Rules

| Rule | Description |
|---|---|
| Output to input only | Wires must connect an output port to an input port |
| One wire per output | Each output port can have at most 1 outgoing connection |
| One wire per input | Each input port can have at most 1 incoming connection |
| No self-loops | A node cannot connect to itself |
| No same-direction | Output-to-output and input-to-input connections are not allowed |

!!! note
    If you connect to a port that already has a wire, the existing connection is replaced by the new one.

## Port Types

- **Input ports** are on the **left** side of a node
- **Output ports** are on the **right** side of a node

Each node type has a fixed number of input ports, but some types have dynamic outputs:

| Node Type | Input Ports | Output Ports |
|---|---|---|
| Start | 0 | 1 |
| Dialogue | 1 | 1 |
| Choice | 1 | N (one per option) |
| Condition | 1 | 2 (True / False) |
| Event | 1 | 1 |
| Random | 1 | N (one per branch) |
| End | 1 | 0 |

## Port Labels

Output ports on Choice and Condition nodes display labels:

- **Choice**: Each output is labeled with the choice option text
- **Condition**: Outputs are labeled **True** and **False**
- **Random**: Outputs show the weight percentage

## Wire Appearance

- **Default**: Light gray (`rgb(180, 180, 180)`)
- **Selected**: Yellow (`rgb(255, 255, 100)`) — when either connected node is selected
- **Dragging preview**: Yellow
- **Thickness**: 2.5px (scaled by zoom level)
- **Shape**: Cubic bezier curve with horizontal control points

## Removing a Connection

Connections are removed automatically when you:

- **Delete a node** — all wires to/from that node are removed
- **Create a new connection** to a port that already has one — the old wire is replaced
- **Undo** a connection action (++ctrl+z++)

!!! tip
    There is no direct "delete wire" action. To remove a specific connection, you can undo the connection or delete and re-add one of the connected nodes.

## Flow Direction

TaleNode graphs flow **left to right** by convention:

```
[Start] → [Dialogue] → [Choice] → [Dialogue] → [End]
```

Input ports are on the left, output ports on the right. While you can position nodes however you like, left-to-right layout keeps the graph readable.
