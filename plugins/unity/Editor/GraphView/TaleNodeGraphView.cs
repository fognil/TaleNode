// TaleNode — GraphView subclass for read-only dialogue graph visualization.

using System.Collections.Generic;
using System.Linq;
using UnityEditor;
using UnityEditor.Experimental.GraphView;
using UnityEngine;
using UnityEngine.UIElements;

namespace TaleNode.Editor
{
    public class TaleNodeGraphView : GraphView
    {
        private readonly Dictionary<string, TaleNodeGraphNode> _nodeMap = new();
        private TaleNodeDialogue _dialogue;
        private string _highlightedNodeId;

        public TaleNodeGraphView()
        {
            SetupZoom(ContentZoomer.DefaultMinScale, ContentZoomer.DefaultMaxScale);
            this.AddManipulator(new ContentDragger());
            this.AddManipulator(new SelectionDragger());
            this.AddManipulator(new RectangleSelector());

            var grid = new GridBackground();
            Insert(0, grid);
            grid.StretchToParentSize();

            var uss = AssetDatabase.LoadAssetAtPath<StyleSheet>(
                "Packages/com.talenode.dialogue/Editor/GraphView/TaleNodeGraphStyles.uss");
            if (uss != null)
                styleSheets.Add(uss);
        }

        public override List<Port> GetCompatiblePorts(Port startPort, NodeAdapter nodeAdapter)
        {
            // Read-only: no new connections allowed
            return new List<Port>();
        }

        public void Clear()
        {
            _nodeMap.Clear();
            DeleteElements(graphElements.ToList());
            _dialogue = null;
        }

        public void BuildFromDialogue(TaleNodeDialogue dialogue, string locale)
        {
            Clear();
            _dialogue = dialogue;
            if (dialogue?.Data?.Nodes == null) return;

            // Create graph nodes
            foreach (var nodeData in dialogue.Data.Nodes)
            {
                var graphNode = TaleNodeGraphNode.Create(nodeData, dialogue, locale);
                _nodeMap[nodeData.Id] = graphNode;
                AddElement(graphNode);
            }

            // Create edges
            foreach (var nodeData in dialogue.Data.Nodes)
            {
                CreateEdgesForNode(nodeData);
            }

            // Auto-layout
            TaleNodeGraphLayout.ApplyLayout(_nodeMap, dialogue.Data);

            schedule.Execute(_ => FrameAll());
        }

        public void UpdateLocale(string locale)
        {
            if (_dialogue == null) return;
            foreach (var kv in _nodeMap)
            {
                kv.Value.UpdateDisplayText(_dialogue, locale);
            }
        }

        public void HighlightNode(string nodeId)
        {
            // Remove previous highlight
            if (_highlightedNodeId != null
                && _nodeMap.TryGetValue(_highlightedNodeId, out var prev))
            {
                prev.RemoveFromClassList("talenode-active-node");
            }

            _highlightedNodeId = nodeId;
            if (nodeId != null && _nodeMap.TryGetValue(nodeId, out var node))
            {
                node.AddToClassList("talenode-active-node");
                // Scroll to node
                var pos = node.GetPosition();
                var center = new Vector3(
                    pos.x + pos.width / 2f,
                    pos.y + pos.height / 2f, 0);
                UpdateViewTransform(
                    -center * viewTransform.scale.x
                    + new Vector3(layout.width / 2f, layout.height / 2f, 0),
                    viewTransform.scale);
            }
        }

        public void SearchNodes(string query)
        {
            foreach (var kv in _nodeMap)
            {
                bool match = string.IsNullOrEmpty(query)
                    || kv.Value.title.ToLowerInvariant()
                        .Contains(query.ToLowerInvariant())
                    || kv.Key.ToLowerInvariant()
                        .Contains(query.ToLowerInvariant());
                kv.Value.style.opacity = match ? 1f : 0.25f;
            }
        }

        private void CreateEdgesForNode(TaleNodeNodeData nodeData)
        {
            switch (nodeData.NodeType)
            {
                case TaleNodeTypes.Start:
                case TaleNodeTypes.Dialogue:
                case TaleNodeTypes.Event:
                case TaleNodeTypes.SubGraph:
                    ConnectPorts(nodeData.Id, "output", nodeData.Next);
                    break;

                case TaleNodeTypes.Choice:
                    if (nodeData.Options != null)
                    {
                        for (int i = 0; i < nodeData.Options.Count; i++)
                        {
                            ConnectPorts(nodeData.Id, $"option_{i}",
                                nodeData.Options[i].Next);
                        }
                    }
                    break;

                case TaleNodeTypes.Condition:
                    ConnectPorts(nodeData.Id, "true", nodeData.TrueNext);
                    ConnectPorts(nodeData.Id, "false", nodeData.FalseNext);
                    break;

                case TaleNodeTypes.Random:
                    if (nodeData.Branches != null)
                    {
                        for (int i = 0; i < nodeData.Branches.Count; i++)
                        {
                            ConnectPorts(nodeData.Id, $"branch_{i}",
                                nodeData.Branches[i].Next);
                        }
                    }
                    break;
            }
        }

        private void ConnectPorts(string fromNodeId, string outputPortName,
            string toNodeId)
        {
            if (string.IsNullOrEmpty(toNodeId)) return;
            if (!_nodeMap.TryGetValue(fromNodeId, out var fromNode)) return;
            if (!_nodeMap.TryGetValue(toNodeId, out var toNode)) return;

            var outPort = fromNode.OutputPorts.GetValueOrDefault(outputPortName);
            var inPort = toNode.InputPort;
            if (outPort == null || inPort == null) return;

            var edge = outPort.ConnectTo(inPort);
            AddElement(edge);
        }
    }
}
