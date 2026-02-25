// TaleNode — Custom GraphView Node for each dialogue node type.

using System.Collections.Generic;
using UnityEditor.Experimental.GraphView;
using UnityEngine;
using UnityEngine.UIElements;

namespace TaleNode.Editor
{
    public class TaleNodeGraphNode : Node
    {
        public string NodeId { get; private set; }
        public string NodeType { get; private set; }
        public Port InputPort { get; private set; }
        public Dictionary<string, Port> OutputPorts { get; } = new();

        private Label _bodyLabel;
        private TaleNodeNodeData _nodeData;

        public static TaleNodeGraphNode Create(TaleNodeNodeData data,
            TaleNodeDialogue dialogue, string locale)
        {
            var node = new TaleNodeGraphNode();
            node._nodeData = data;
            node.NodeId = data.Id;
            node.NodeType = data.NodeType;
            node.title = FormatTitle(data);
            node.tooltip = data.Id;

            ApplyColor(node, data.NodeType);
            CreatePorts(node, data);
            CreateBody(node, data, dialogue, locale);

            node.RefreshExpandedState();
            node.RefreshPorts();
            return node;
        }

        public void UpdateDisplayText(TaleNodeDialogue dialogue, string locale)
        {
            if (_bodyLabel == null) return;
            _bodyLabel.text = GetBodyText(_nodeData, dialogue, locale);
        }

        private static string FormatTitle(TaleNodeNodeData data)
        {
            string type = data.NodeType ?? "unknown";
            string upper = char.ToUpperInvariant(type[0]) + type.Substring(1);
            return $"{upper} ({data.Id})";
        }

        private static void ApplyColor(TaleNodeGraphNode node, string nodeType)
        {
            Color color = GetNodeColor(nodeType);
            var titleContainer = node.titleContainer;
            titleContainer.style.backgroundColor = color;

            // Light text on dark backgrounds, dark text on yellow
            bool darkText = nodeType == "choice" || nodeType == "random";
            var titleLabel = titleContainer.Q<Label>("title-label");
            if (titleLabel != null)
                titleLabel.style.color = darkText
                    ? new Color(0.1f, 0.1f, 0.1f)
                    : Color.white;
        }

        private static Color GetNodeColor(string nodeType)
        {
            return nodeType switch
            {
                "start" => FromHex("#4CAF50"),
                "dialogue" => FromHex("#4285F4"),
                "choice" => FromHex("#FBBC04"),
                "condition" => FromHex("#FF9800"),
                "event" => FromHex("#AB47BC"),
                "random" => FromHex("#9E9E9E"),
                "end" => FromHex("#F44336"),
                _ => FromHex("#607D8B"),
            };
        }

        private static Color FromHex(string hex)
        {
            ColorUtility.TryParseHtmlString(hex, out var c);
            return c;
        }

        private static void CreatePorts(TaleNodeGraphNode node,
            TaleNodeNodeData data)
        {
            // Input port (all except start)
            if (data.NodeType != "start")
            {
                var input = Port.Create<Edge>(
                    Orientation.Horizontal, Direction.Input,
                    Port.Capacity.Single, typeof(bool));
                input.portName = "In";
                node.inputContainer.Add(input);
                node.InputPort = input;
            }

            // Output ports
            switch (data.NodeType)
            {
                case "start":
                case "dialogue":
                case "event":
                case "subgraph":
                    AddOutputPort(node, "output", "Out");
                    break;

                case "choice":
                    if (data.Options != null)
                    {
                        for (int i = 0; i < data.Options.Count; i++)
                        {
                            string label = TruncateText(
                                data.Options[i].Text ?? $"Option {i}", 24);
                            AddOutputPort(node, $"option_{i}", label);
                        }
                    }
                    break;

                case "condition":
                    AddOutputPort(node, "true", "True");
                    AddOutputPort(node, "false", "False");
                    break;

                case "random":
                    if (data.Branches != null)
                    {
                        for (int i = 0; i < data.Branches.Count; i++)
                        {
                            string label = $"w={data.Branches[i].Weight:F1}";
                            AddOutputPort(node, $"branch_{i}", label);
                        }
                    }
                    break;

                // end: no output
            }
        }

        private static void AddOutputPort(TaleNodeGraphNode node,
            string portName, string label)
        {
            var port = Port.Create<Edge>(
                Orientation.Horizontal, Direction.Output,
                Port.Capacity.Single, typeof(bool));
            port.portName = label;
            node.outputContainer.Add(port);
            node.OutputPorts[portName] = port;
        }

        private static void CreateBody(TaleNodeGraphNode node,
            TaleNodeNodeData data, TaleNodeDialogue dialogue, string locale)
        {
            string text = GetBodyText(data, dialogue, locale);
            if (string.IsNullOrEmpty(text)) return;

            var label = new Label(text);
            label.style.whiteSpace = WhiteSpace.Normal;
            label.style.maxWidth = 220;
            label.style.paddingLeft = 8;
            label.style.paddingRight = 8;
            label.style.paddingTop = 4;
            label.style.paddingBottom = 4;
            label.style.fontSize = 11;
            label.style.color = new Color(0.85f, 0.85f, 0.85f);

            node.extensionContainer.Add(label);
            node._bodyLabel = label;
        }

        private static string GetBodyText(TaleNodeNodeData data,
            TaleNodeDialogue dialogue, string locale)
        {
            switch (data.NodeType)
            {
                case "dialogue":
                    string speaker = ResolveSpeaker(data.Speaker, dialogue);
                    string text = ResolveLocalizedText(
                        data.Id, data.Text, dialogue, locale);
                    string emo = string.IsNullOrEmpty(data.Emotion)
                        ? "" : $" [{data.Emotion}]";
                    return $"{speaker}{emo}:\n{TruncateText(text, 80)}";

                case "choice":
                    string prompt = ResolveLocalizedText(
                        data.Id, data.Prompt, dialogue, locale);
                    return string.IsNullOrEmpty(prompt)
                        ? null : TruncateText(prompt, 60);

                case "condition":
                    return $"{data.Variable} {data.Operator} {data.Value}";

                case "event":
                    if (data.Actions == null || data.Actions.Count == 0)
                        return null;
                    var lines = new List<string>();
                    foreach (var a in data.Actions)
                    {
                        lines.Add($"{a.Action}: {a.Key} = {a.Value}");
                        if (lines.Count >= 3) { lines.Add("..."); break; }
                    }
                    return string.Join("\n", lines);

                case "end":
                    return string.IsNullOrEmpty(data.Tag)
                        ? null : $"Tag: {data.Tag}";

                default:
                    return null;
            }
        }

        private static string ResolveSpeaker(string speakerId,
            TaleNodeDialogue dialogue)
        {
            if (string.IsNullOrEmpty(speakerId)) return "???";
            if (dialogue?.Data?.Characters == null) return speakerId;
            foreach (var c in dialogue.Data.Characters)
            {
                if (c.Id == speakerId)
                    return c.Name ?? speakerId;
            }
            return speakerId;
        }

        private static string ResolveLocalizedText(string nodeId,
            string fallback, TaleNodeDialogue dialogue, string locale)
        {
            if (string.IsNullOrEmpty(locale)
                || dialogue?.Data?.Strings == null)
                return fallback ?? "";

            if (dialogue.Data.Strings.TryGetValue(nodeId, out var translations)
                && translations.TryGetValue(locale, out var localized))
                return localized;

            return fallback ?? "";
        }

        private static string TruncateText(string text, int maxLen)
        {
            if (string.IsNullOrEmpty(text)) return "";
            return text.Length <= maxLen ? text : text.Substring(0, maxLen) + "...";
        }
    }
}
