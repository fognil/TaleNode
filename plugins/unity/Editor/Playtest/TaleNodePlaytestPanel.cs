// TaleNode — In-editor playtest panel for running dialogues.
// Uses TaleNodeRunner from the Runtime assembly to execute dialogue JSON.

using System.Collections.Generic;
using UnityEditor;
using UnityEditor.UIElements;
using UnityEngine;
using UnityEngine.UIElements;

namespace TaleNode.Editor
{
    public class TaleNodePlaytestPanel : EditorWindow
    {
        private TaleNodeDialogue _asset;
        private TaleNodeRunner _runner;
        private string _currentLocale;

        // UI elements
        private ObjectField _assetField;
        private DropdownField _localeDropdown;
        private Button _playBtn;
        private Button _stopBtn;
        private Button _restartBtn;
        private Button _advanceBtn;
        private ScrollView _logScroll;
        private VisualElement _choiceContainer;
        private ScrollView _variableWatch;

        // State
        private bool _waitingForChoice;
        private readonly List<string> _currentChoices = new();

        [MenuItem("Window/TaleNode/Playtest")]
        public static void ShowWindow()
        {
            GetWindow<TaleNodePlaytestPanel>("TaleNode Playtest");
        }

        public static void OpenWithAsset(TaleNodeDialogue asset)
        {
            var window = GetWindow<TaleNodePlaytestPanel>("TaleNode Playtest");
            window.SetAsset(asset);
        }

        private void CreateGUI()
        {
            var root = rootVisualElement;

            // Toolbar
            var toolbar = new Toolbar();

            _assetField = new ObjectField("Dialogue")
            {
                objectType = typeof(TaleNodeDialogue),
                allowSceneObjects = false
            };
            _assetField.style.minWidth = 200;
            _assetField.RegisterValueChangedCallback(evt =>
                SetAsset(evt.newValue as TaleNodeDialogue));
            toolbar.Add(_assetField);

            _localeDropdown = new DropdownField("Locale",
                new List<string>(), 0);
            _localeDropdown.style.minWidth = 100;
            _localeDropdown.RegisterValueChangedCallback(evt =>
                _currentLocale = evt.newValue);
            _localeDropdown.SetEnabled(false);
            toolbar.Add(_localeDropdown);

            _playBtn = new ToolbarButton(OnPlay) { text = "Play" };
            toolbar.Add(_playBtn);

            _stopBtn = new ToolbarButton(OnStop) { text = "Stop" };
            _stopBtn.SetEnabled(false);
            toolbar.Add(_stopBtn);

            _restartBtn = new ToolbarButton(OnRestart) { text = "Restart" };
            _restartBtn.SetEnabled(false);
            toolbar.Add(_restartBtn);

            root.Add(toolbar);

            // Main split: log left, variables right
            var split = new TwoPaneSplitView(
                0, 250, TwoPaneSplitViewOrientation.Horizontal);
            split.style.flexGrow = 1;

            // Left pane: log + choices
            var leftPane = new VisualElement();
            leftPane.style.flexGrow = 1;
            leftPane.style.minWidth = 200;

            _logScroll = new ScrollView(ScrollViewMode.Vertical);
            _logScroll.style.flexGrow = 1;
            _logScroll.style.backgroundColor = new Color(0.12f, 0.12f, 0.12f);
            _logScroll.style.paddingLeft = 8;
            _logScroll.style.paddingRight = 8;
            _logScroll.style.paddingTop = 4;
            leftPane.Add(_logScroll);

            _advanceBtn = new Button(OnAdvance) { text = "Continue >>>" };
            _advanceBtn.style.height = 28;
            _advanceBtn.style.display = DisplayStyle.None;
            leftPane.Add(_advanceBtn);

            _choiceContainer = new VisualElement();
            _choiceContainer.style.paddingLeft = 8;
            _choiceContainer.style.paddingRight = 8;
            _choiceContainer.style.paddingTop = 4;
            _choiceContainer.style.paddingBottom = 4;
            leftPane.Add(_choiceContainer);

            split.Add(leftPane);

            // Right pane: variable watch
            var rightPane = new VisualElement();
            rightPane.style.minWidth = 150;

            var watchHeader = new Label("Variables");
            watchHeader.style.unityFontStyleAndWeight = FontStyle.Bold;
            watchHeader.style.paddingLeft = 4;
            watchHeader.style.paddingTop = 4;
            rightPane.Add(watchHeader);

            _variableWatch = new ScrollView(ScrollViewMode.Vertical);
            _variableWatch.style.flexGrow = 1;
            _variableWatch.style.backgroundColor =
                new Color(0.14f, 0.14f, 0.14f);
            rightPane.Add(_variableWatch);

            split.Add(rightPane);
            root.Add(split);
        }

        private void SetAsset(TaleNodeDialogue asset)
        {
            _asset = asset;
            _assetField.SetValueWithoutNotify(asset);
            OnStop();

            if (asset == null)
            {
                _localeDropdown.SetEnabled(false);
                return;
            }

            if (asset.Locales != null && asset.Locales.Count > 0)
            {
                _localeDropdown.choices = new List<string>(asset.Locales);
                _currentLocale = asset.DefaultLocale ?? asset.Locales[0];
                _localeDropdown.SetValueWithoutNotify(_currentLocale);
                _localeDropdown.SetEnabled(true);
            }
            else
            {
                _localeDropdown.choices = new List<string> { "(none)" };
                _localeDropdown.SetValueWithoutNotify("(none)");
                _localeDropdown.SetEnabled(false);
                _currentLocale = null;
            }
        }

        private void OnPlay()
        {
            if (_asset == null || _asset.Data == null) return;

            _logScroll.Clear();
            _choiceContainer.Clear();
            _variableWatch.Clear();
            _waitingForChoice = false;

            _runner = new TaleNodeRunner();
            _runner.OnDialogueLine += HandleDialogueLine;
            _runner.OnChoicePresented += HandleChoice;
            _runner.OnDialogueEnded += HandleEnd;
            _runner.OnEventTriggered += HandleEvent;
            _runner.OnVariableChanged += HandleVariableChanged;

            _runner.LoadFromString(_asset.RawJson);

            // Initialize variable watch
            if (_asset.Data.Variables != null)
            {
                foreach (var v in _asset.Data.Variables)
                {
                    AddVariableRow(v.Name,
                        v.Default?.ToString() ?? "null", false);
                }
            }

            _runner.Start();

            _playBtn.SetEnabled(false);
            _stopBtn.SetEnabled(true);
            _restartBtn.SetEnabled(true);
        }

        private void OnStop()
        {
            if (_runner != null)
            {
                _runner.Stop();
                _runner = null;
            }
            _waitingForChoice = false;
            _advanceBtn.style.display = DisplayStyle.None;
            _choiceContainer.Clear();
            _playBtn.SetEnabled(true);
            _stopBtn.SetEnabled(false);
            _restartBtn.SetEnabled(false);
        }

        private void OnRestart()
        {
            OnStop();
            OnPlay();
        }

        private void OnAdvance()
        {
            _advanceBtn.style.display = DisplayStyle.None;
            _runner?.Advance();
        }

        private void HandleDialogueLine(object sender,
            DialogueLineEventArgs e)
        {
            string text = ResolveLocaleText(e.NodeId, e.Text);
            AddLogEntry(e.Speaker, text, "#4285F4");
            SyncGraphHighlight(e.NodeId);

            _advanceBtn.style.display = DisplayStyle.Flex;
        }

        private void HandleChoice(object sender, ChoiceEventArgs e)
        {
            if (!string.IsNullOrEmpty(e.Prompt))
                AddLogEntry("", e.Prompt, "#FBBC04");

            _choiceContainer.Clear();
            _currentChoices.Clear();
            _waitingForChoice = true;

            for (int i = 0; i < e.Options.Count; i++)
            {
                int index = i;
                string optText = e.Options[i];
                _currentChoices.Add(optText);

                var btn = new Button(() => OnChoiceSelected(index))
                {
                    text = $"{i + 1}. {optText}"
                };
                btn.style.height = 26;
                btn.style.marginBottom = 2;
                btn.style.unityTextAlign = TextAnchor.MiddleLeft;
                _choiceContainer.Add(btn);
            }
        }

        private void OnChoiceSelected(int index)
        {
            if (!_waitingForChoice || _runner == null) return;

            string chosen = index < _currentChoices.Count
                ? _currentChoices[index] : "?";
            AddLogEntry("You", chosen, "#81C784");

            _choiceContainer.Clear();
            _waitingForChoice = false;
            _runner.Choose(index);
        }

        private void HandleEnd(object sender, DialogueEndedEventArgs e)
        {
            string tag = string.IsNullOrEmpty(e.Tag)
                ? "" : $" (tag: {e.Tag})";
            AddLogEntry("", $"--- Dialogue ended{tag} ---", "#F44336");

            _advanceBtn.style.display = DisplayStyle.None;
            _choiceContainer.Clear();
            _waitingForChoice = false;
            _playBtn.SetEnabled(true);
            _stopBtn.SetEnabled(false);
        }

        private void HandleEvent(object sender,
            EventTriggeredEventArgs e)
        {
            AddLogEntry("Event",
                $"{e.Action}: {e.Key} = {e.Value}", "#AB47BC");
        }

        private void HandleVariableChanged(object sender,
            VariableChangedEventArgs e)
        {
            UpdateVariableRow(e.Key,
                e.Value?.ToString() ?? "null");
        }

        private void AddLogEntry(string speaker, string text,
            string hexColor)
        {
            var row = new VisualElement();
            row.style.flexDirection = FlexDirection.Row;
            row.style.marginBottom = 2;
            row.style.paddingTop = 2;
            row.style.paddingBottom = 2;
            row.style.borderBottomWidth = 1;
            row.style.borderBottomColor =
                new Color(0.2f, 0.2f, 0.2f);

            if (!string.IsNullOrEmpty(speaker))
            {
                var speakerLabel = new Label($"{speaker}: ");
                speakerLabel.style.unityFontStyleAndWeight =
                    FontStyle.Bold;
                if (ColorUtility.TryParseHtmlString(hexColor, out var c))
                    speakerLabel.style.color = c;
                speakerLabel.style.minWidth = 60;
                row.Add(speakerLabel);
            }

            var textLabel = new Label(text);
            textLabel.style.whiteSpace = WhiteSpace.Normal;
            textLabel.style.flexShrink = 1;
            row.Add(textLabel);

            _logScroll.Add(row);

            // Auto-scroll to bottom
            _logScroll.schedule.Execute(() =>
                _logScroll.scrollOffset = new Vector2(0, float.MaxValue));
        }

        private void AddVariableRow(string name, string value,
            bool changed)
        {
            var row = new VisualElement();
            row.style.flexDirection = FlexDirection.Row;
            row.style.paddingLeft = 4;
            row.style.paddingTop = 2;
            row.style.paddingBottom = 2;
            row.name = $"var_{name}";

            var nameLabel = new Label(name);
            nameLabel.style.minWidth = 80;
            nameLabel.style.unityFontStyleAndWeight = FontStyle.Bold;
            row.Add(nameLabel);

            var valLabel = new Label(value);
            valLabel.name = "value";
            if (changed)
                valLabel.style.color = new Color(1f, 0.85f, 0.2f);
            row.Add(valLabel);

            _variableWatch.Add(row);
        }

        private void UpdateVariableRow(string name, string value)
        {
            var row = _variableWatch.Q<VisualElement>($"var_{name}");
            if (row != null)
            {
                var valLabel = row.Q<Label>("value");
                if (valLabel != null)
                {
                    valLabel.text = value;
                    valLabel.style.color =
                        new Color(1f, 0.85f, 0.2f);
                }
            }
            else
            {
                AddVariableRow(name, value, true);
            }
        }

        private string ResolveLocaleText(string nodeId, string fallback)
        {
            if (string.IsNullOrEmpty(_currentLocale)
                || _asset?.Data?.Strings == null)
                return fallback ?? "";

            if (_asset.Data.Strings.TryGetValue(nodeId,
                    out var translations)
                && translations.TryGetValue(_currentLocale,
                    out var localized))
                return localized;

            return fallback ?? "";
        }

        private static void SyncGraphHighlight(string nodeId)
        {
            var graphWindow = GetWindow<TaleNodeGraphWindow>(
                "TaleNode Graph", false);
            if (graphWindow != null)
                graphWindow.HighlightNode(nodeId);
        }
    }
}
