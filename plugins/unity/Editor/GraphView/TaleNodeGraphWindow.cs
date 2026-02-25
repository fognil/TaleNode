// TaleNode — EditorWindow host for the dialogue graph visualization.

using System.Collections.Generic;
using UnityEditor;
using UnityEditor.UIElements;
using UnityEngine;
using UnityEngine.UIElements;

namespace TaleNode.Editor
{
    public class TaleNodeGraphWindow : EditorWindow
    {
        private TaleNodeGraphView _graphView;
        private ObjectField _assetField;
        private DropdownField _localeDropdown;
        private TextField _searchField;
        private TaleNodeDialogue _currentAsset;
        private string _currentLocale;

        [MenuItem("Window/TaleNode/Dialogue Graph")]
        public static void ShowWindow()
        {
            GetWindow<TaleNodeGraphWindow>("TaleNode Graph");
        }

        public static void OpenDialogue(TaleNodeDialogue asset)
        {
            var window = GetWindow<TaleNodeGraphWindow>("TaleNode Graph");
            window.LoadAsset(asset);
        }

        public void HighlightNode(string nodeId)
        {
            _graphView?.HighlightNode(nodeId);
        }

        private void CreateGUI()
        {
            var toolbar = new Toolbar();

            _assetField = new ObjectField("Dialogue")
            {
                objectType = typeof(TaleNodeDialogue),
                allowSceneObjects = false
            };
            _assetField.style.minWidth = 200;
            _assetField.RegisterValueChangedCallback(evt =>
            {
                LoadAsset(evt.newValue as TaleNodeDialogue);
            });
            toolbar.Add(_assetField);

            _localeDropdown = new DropdownField("Locale", new List<string> { "(none)" }, 0);
            _localeDropdown.style.minWidth = 120;
            _localeDropdown.RegisterValueChangedCallback(evt =>
            {
                _currentLocale = evt.newValue;
                _graphView?.UpdateLocale(_currentLocale);
            });
            _localeDropdown.SetEnabled(false);
            toolbar.Add(_localeDropdown);

            var frameBtn = new ToolbarButton(() => _graphView?.FrameAll())
            { text = "Frame All" };
            toolbar.Add(frameBtn);

            _searchField = new TextField { tooltip = "Search nodes..." };
            _searchField.style.minWidth = 140;
            _searchField.RegisterValueChangedCallback(evt =>
            {
                _graphView?.SearchNodes(evt.newValue);
            });
            toolbar.Add(_searchField);

            rootVisualElement.Add(toolbar);

            _graphView = new TaleNodeGraphView();
            _graphView.StretchToParentSize();
            _graphView.style.top = EditorGUIUtility.singleLineHeight + 4;
            rootVisualElement.Add(_graphView);
        }

        private void LoadAsset(TaleNodeDialogue asset)
        {
            _currentAsset = asset;
            _assetField.SetValueWithoutNotify(asset);

            if (asset == null || asset.Data == null)
            {
                _graphView?.Clear();
                _localeDropdown.SetEnabled(false);
                return;
            }

            // Populate locale dropdown
            var locales = new List<string>();
            if (asset.Locales != null && asset.Locales.Count > 0)
            {
                locales.AddRange(asset.Locales);
                _currentLocale = asset.DefaultLocale ?? locales[0];
                _localeDropdown.choices = locales;
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

            _graphView.BuildFromDialogue(asset, _currentLocale);
        }
    }
}
