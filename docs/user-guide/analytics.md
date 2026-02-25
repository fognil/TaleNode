# Analytics

The Analytics panel provides statistics about your dialogue graph — node counts, path analysis, branching complexity, and connectivity checks.

## Opening the Analytics Panel

- **Menu**: View > Analytics Panel

## Metrics

### Node Counts

| Metric | Description |
|---|---|
| **Total nodes** | Number of nodes in the graph |
| **Total connections** | Number of wires between nodes |
| **Nodes by type** | Breakdown per node type (Start, Dialogue, Choice, etc.) |

### Path Analysis

Counts all possible paths from Start nodes to End nodes.

| Metric | Description |
|---|---|
| **Total paths** | Number of unique routes through the dialogue |
| **Longest path** | Maximum steps from Start to End |
| **Shortest path** | Minimum steps from Start to End |

### Branching

| Metric | Description |
|---|---|
| **Max fan-out** | Highest number of outgoing connections from any single node |
| **Avg choices per Choice node** | Average number of options across all Choice nodes |

### Connectivity

| Metric | Description |
|---|---|
| **Unreachable nodes** | Nodes that cannot be reached from any Start node |
| **Dead ends** | Non-End nodes with output ports but no outgoing connections |

Unreachable nodes and dead ends often indicate disconnected or incomplete dialogue branches.

## Exporting Analytics

The panel provides two export options:

| Button | Output |
|---|---|
| **Export CSV** | Comma-separated values file with all metrics |
| **Export Report** | Human-readable text report |

## Tips

!!! tip
    Check analytics before exporting your dialogue. Zero unreachable nodes and zero dead ends means every node is part of a valid path.

!!! tip
    A high number of total paths relative to node count indicates a well-branched dialogue with good replay value.
