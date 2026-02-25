// TaleNode — BFS auto-layout for graph nodes.
// Places nodes left-to-right by distance from start node(s).

using System.Collections.Generic;
using System.Linq;
using UnityEngine;

namespace TaleNode.Editor
{
    public static class TaleNodeGraphLayout
    {
        private const float ColumnSpacing = 300f;
        private const float RowSpacing = 120f;

        public static void ApplyLayout(
            Dictionary<string, TaleNodeGraphNode> nodeMap,
            TaleNodeDialogueData data)
        {
            if (data?.Nodes == null || data.Nodes.Count == 0) return;

            var adjacency = BuildAdjacency(data);
            var depths = ComputeDepths(data, adjacency);

            // Group nodes by depth column
            var columns = new SortedDictionary<int, List<string>>();
            var orphans = new List<string>();

            foreach (var node in data.Nodes)
            {
                if (depths.TryGetValue(node.Id, out int depth))
                {
                    if (!columns.ContainsKey(depth))
                        columns[depth] = new List<string>();
                    columns[depth].Add(node.Id);
                }
                else
                {
                    orphans.Add(node.Id);
                }
            }

            // Assign positions per column
            foreach (var kv in columns)
            {
                float x = kv.Key * ColumnSpacing;
                var ids = kv.Value;
                OrderWithinColumn(ids, data, depths);
                float startY = -(ids.Count - 1) * RowSpacing / 2f;

                for (int i = 0; i < ids.Count; i++)
                {
                    if (nodeMap.TryGetValue(ids[i], out var graphNode))
                    {
                        graphNode.SetPosition(new Rect(
                            x, startY + i * RowSpacing, 240, 150));
                    }
                }
            }

            // Place orphans at far right
            if (orphans.Count > 0)
            {
                int maxCol = columns.Count > 0 ? columns.Keys.Max() + 2 : 0;
                float x = maxCol * ColumnSpacing;
                float startY = -(orphans.Count - 1) * RowSpacing / 2f;

                for (int i = 0; i < orphans.Count; i++)
                {
                    if (nodeMap.TryGetValue(orphans[i], out var graphNode))
                    {
                        graphNode.SetPosition(new Rect(
                            x, startY + i * RowSpacing, 240, 150));
                    }
                }
            }
        }

        private static Dictionary<string, List<string>> BuildAdjacency(
            TaleNodeDialogueData data)
        {
            var adj = new Dictionary<string, List<string>>();

            foreach (var node in data.Nodes)
            {
                var targets = new List<string>();

                if (!string.IsNullOrEmpty(node.Next))
                    targets.Add(node.Next);

                if (!string.IsNullOrEmpty(node.TrueNext))
                    targets.Add(node.TrueNext);
                if (!string.IsNullOrEmpty(node.FalseNext))
                    targets.Add(node.FalseNext);

                if (node.Options != null)
                    foreach (var opt in node.Options)
                        if (!string.IsNullOrEmpty(opt.Next))
                            targets.Add(opt.Next);

                if (node.Branches != null)
                    foreach (var br in node.Branches)
                        if (!string.IsNullOrEmpty(br.Next))
                            targets.Add(br.Next);

                adj[node.Id] = targets;
            }

            return adj;
        }

        private static Dictionary<string, int> ComputeDepths(
            TaleNodeDialogueData data,
            Dictionary<string, List<string>> adjacency)
        {
            var depths = new Dictionary<string, int>();
            var queue = new Queue<string>();

            // Seed BFS from all start nodes
            foreach (var node in data.Nodes)
            {
                if (node.NodeType == "start")
                {
                    depths[node.Id] = 0;
                    queue.Enqueue(node.Id);
                }
            }

            // If no start node, seed from first node
            if (queue.Count == 0 && data.Nodes.Count > 0)
            {
                depths[data.Nodes[0].Id] = 0;
                queue.Enqueue(data.Nodes[0].Id);
            }

            while (queue.Count > 0)
            {
                string current = queue.Dequeue();
                int currentDepth = depths[current];

                if (!adjacency.TryGetValue(current, out var neighbors))
                    continue;

                foreach (string next in neighbors)
                {
                    int newDepth = currentDepth + 1;
                    // Place multi-path nodes at max depth
                    if (!depths.ContainsKey(next) || depths[next] < newDepth)
                    {
                        depths[next] = newDepth;
                        queue.Enqueue(next);
                    }
                }
            }

            return depths;
        }

        /// <summary>
        /// Order nodes within a column so that branch outputs fan out
        /// vertically in a predictable order (true above false for conditions,
        /// options in order for choices).
        /// </summary>
        private static void OrderWithinColumn(List<string> ids,
            TaleNodeDialogueData data, Dictionary<string, int> depths)
        {
            // Build a lookup for node data
            var dataMap = new Dictionary<string, TaleNodeNodeData>();
            foreach (var n in data.Nodes) dataMap[n.Id] = n;

            // Sort by the index at which the parent references this node.
            // This keeps true_next above false_next, option 0 above option 1.
            ids.Sort((a, b) =>
            {
                int orderA = GetParentOutputIndex(a, dataMap, depths);
                int orderB = GetParentOutputIndex(b, dataMap, depths);
                if (orderA != orderB) return orderA.CompareTo(orderB);
                return string.Compare(a, b, System.StringComparison.Ordinal);
            });
        }

        private static int GetParentOutputIndex(string nodeId,
            Dictionary<string, TaleNodeNodeData> dataMap,
            Dictionary<string, int> depths)
        {
            if (!depths.TryGetValue(nodeId, out int myDepth) || myDepth == 0)
                return 0;

            foreach (var kv in dataMap)
            {
                var parent = kv.Value;
                if (!depths.TryGetValue(parent.Id, out int pDepth)
                    || pDepth != myDepth - 1)
                    continue;

                // Check which output slot points to nodeId
                if (parent.Next == nodeId) return 0;
                if (parent.TrueNext == nodeId) return 0;
                if (parent.FalseNext == nodeId) return 1;

                if (parent.Options != null)
                {
                    for (int i = 0; i < parent.Options.Count; i++)
                        if (parent.Options[i].Next == nodeId) return i;
                }

                if (parent.Branches != null)
                {
                    for (int i = 0; i < parent.Branches.Count; i++)
                        if (parent.Branches[i].Next == nodeId) return i;
                }
            }

            return 0;
        }
    }
}
