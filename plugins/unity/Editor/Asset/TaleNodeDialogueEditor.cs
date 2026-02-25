// TaleNode — Custom Inspector for TaleNodeDialogue assets.
// Shows metadata, node stats, characters, variables, and action buttons.

using UnityEditor;
using UnityEngine;
using UnityEngine.UIElements;

namespace TaleNode.Editor
{
    [CustomEditor(typeof(TaleNodeDialogue))]
    public class TaleNodeDialogueEditor : UnityEditor.Editor
    {
        public override VisualElement CreateInspectorGUI()
        {
            var dialogue = (TaleNodeDialogue)target;
            var root = new VisualElement();
            root.style.paddingTop = 4;

            AddHeader(root, dialogue);
            AddNodeStats(root, dialogue);
            AddCharacters(root, dialogue);
            AddVariables(root, dialogue);
            AddLocales(root, dialogue);
            AddButtons(root, dialogue);

            return root;
        }

        private static void AddHeader(VisualElement root, TaleNodeDialogue d)
        {
            var title = new Label(string.IsNullOrEmpty(d.DialogueName)
                ? "(Unnamed Dialogue)" : d.DialogueName);
            title.style.fontSize = 16;
            title.style.unityFontStyleAndWeight = FontStyle.Bold;
            title.style.marginBottom = 4;
            root.Add(title);

            var info = new Label($"Version {d.Version}  |  {d.NodeCount} nodes");
            info.style.color = new Color(0.6f, 0.6f, 0.6f);
            info.style.marginBottom = 8;
            root.Add(info);
        }

        private static void AddNodeStats(VisualElement root, TaleNodeDialogue d)
        {
            var foldout = new Foldout { text = "Node Statistics", value = true };
            foldout.style.marginBottom = 4;

            AddStatRow(foldout, "Start", d.StartCount, "#4CAF50");
            AddStatRow(foldout, "Dialogue", d.DialogueNodeCount, "#4285F4");
            AddStatRow(foldout, "Choice", d.ChoiceCount, "#FBBC04");
            AddStatRow(foldout, "Condition", d.ConditionCount, "#FF9800");
            AddStatRow(foldout, "Event", d.EventCount, "#AB47BC");
            AddStatRow(foldout, "Random", d.RandomCount, "#9E9E9E");
            AddStatRow(foldout, "End", d.EndCount, "#F44336");

            root.Add(foldout);
        }

        private static void AddStatRow(VisualElement parent, string label,
            int count, string hexColor)
        {
            if (count == 0) return;

            var row = new VisualElement();
            row.style.flexDirection = FlexDirection.Row;
            row.style.marginLeft = 4;
            row.style.marginBottom = 2;

            var dot = new VisualElement();
            dot.style.width = 10;
            dot.style.height = 10;
            dot.style.borderTopLeftRadius = 5;
            dot.style.borderTopRightRadius = 5;
            dot.style.borderBottomLeftRadius = 5;
            dot.style.borderBottomRightRadius = 5;
            dot.style.marginRight = 6;
            dot.style.marginTop = 3;
            if (ColorUtility.TryParseHtmlString(hexColor, out var c))
                dot.style.backgroundColor = c;
            row.Add(dot);

            var text = new Label($"{label}: {count}");
            row.Add(text);
            parent.Add(row);
        }

        private static void AddCharacters(VisualElement root, TaleNodeDialogue d)
        {
            if (d.CharacterNames == null || d.CharacterNames.Count == 0) return;

            var foldout = new Foldout { text = $"Characters ({d.CharacterCount})" };
            foldout.style.marginBottom = 4;

            foreach (var name in d.CharacterNames)
            {
                var label = new Label($"  {name}");
                label.style.marginLeft = 4;
                foldout.Add(label);
            }
            root.Add(foldout);
        }

        private static void AddVariables(VisualElement root, TaleNodeDialogue d)
        {
            if (d.Data?.Variables == null || d.Data.Variables.Count == 0) return;

            var foldout = new Foldout { text = $"Variables ({d.VariableCount})" };
            foldout.style.marginBottom = 4;

            foreach (var v in d.Data.Variables)
            {
                string defaultStr = v.Default?.ToString() ?? "null";
                var label = new Label($"  {v.Name} ({v.Type}) = {defaultStr}");
                label.style.marginLeft = 4;
                foldout.Add(label);
            }
            root.Add(foldout);
        }

        private static void AddLocales(VisualElement root, TaleNodeDialogue d)
        {
            if (d.Locales == null || d.Locales.Count == 0) return;

            var foldout = new Foldout { text = $"Locales ({d.Locales.Count})" };
            foldout.style.marginBottom = 4;

            int totalKeys = d.Data?.Strings?.Count ?? 0;

            foreach (var locale in d.Locales)
            {
                int translated = 0;
                if (d.Data?.Strings != null)
                {
                    foreach (var kv in d.Data.Strings)
                    {
                        if (kv.Value != null && kv.Value.ContainsKey(locale))
                            translated++;
                    }
                }
                bool isDefault = locale == d.DefaultLocale;
                string suffix = isDefault ? " (default)" : "";
                string pct = totalKeys > 0
                    ? $" — {translated}/{totalKeys} ({100 * translated / totalKeys}%)"
                    : "";
                var label = new Label($"  {locale}{suffix}{pct}");
                label.style.marginLeft = 4;
                foldout.Add(label);
            }
            root.Add(foldout);
        }

        private static void AddButtons(VisualElement root, TaleNodeDialogue d)
        {
            var container = new VisualElement();
            container.style.flexDirection = FlexDirection.Row;
            container.style.marginTop = 8;

            var graphBtn = new Button(() => TaleNodeGraphWindow.OpenDialogue(d))
            {
                text = "Open in Graph View"
            };
            graphBtn.style.flexGrow = 1;
            graphBtn.style.height = 28;
            container.Add(graphBtn);

            var playBtn = new Button(() => TaleNodePlaytestPanel.OpenWithAsset(d))
            {
                text = "Open in Playtest"
            };
            playBtn.style.flexGrow = 1;
            playBtn.style.height = 28;
            playBtn.style.marginLeft = 4;
            container.Add(playBtn);

            root.Add(container);
        }
    }
}
