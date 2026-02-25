// TaleNode — ScriptableObject wrapper for imported dialogue assets.

using System.Collections.Generic;
using Newtonsoft.Json;
using UnityEngine;

namespace TaleNode.Editor
{
    public class TaleNodeDialogue : ScriptableObject
    {
        [HideInInspector] public string RawJson;
        [System.NonSerialized] private TaleNodeDialogueData _data;

        /// <summary>Lazily parsed data — survives domain reload via RawJson.</summary>
        public TaleNodeDialogueData Data
        {
            get
            {
                if (_data == null && !string.IsNullOrEmpty(RawJson))
                    _data = JsonConvert.DeserializeObject<TaleNodeDialogueData>(RawJson);
                return _data;
            }
            private set => _data = value;
        }

        // Metadata cached on import
        public string DialogueName;
        public string Version;
        public int NodeCount;
        public int CharacterCount;
        public int VariableCount;
        public List<string> CharacterNames = new();
        public List<string> VariableNames = new();
        public List<string> Locales = new();
        public string DefaultLocale;

        // Per-type node counts
        public int StartCount;
        public int DialogueNodeCount;
        public int ChoiceCount;
        public int ConditionCount;
        public int EventCount;
        public int RandomCount;
        public int EndCount;

        /// <summary>Parse JSON and populate all metadata fields.</summary>
        public bool Parse(string json)
        {
            RawJson = json;
            Data = JsonConvert.DeserializeObject<TaleNodeDialogueData>(json);
            if (Data == null) return false;

            DialogueName = Data.Name ?? "";
            Version = Data.Version ?? "";
            NodeCount = Data.Nodes?.Count ?? 0;
            CharacterCount = Data.Characters?.Count ?? 0;
            VariableCount = Data.Variables?.Count ?? 0;
            DefaultLocale = Data.DefaultLocale ?? "";

            CharacterNames.Clear();
            if (Data.Characters != null)
                foreach (var c in Data.Characters)
                    CharacterNames.Add(c.Name ?? c.Id);

            VariableNames.Clear();
            if (Data.Variables != null)
                foreach (var v in Data.Variables)
                    VariableNames.Add(v.Name);

            Locales.Clear();
            if (Data.Locales != null)
                Locales.AddRange(Data.Locales);

            CountNodeTypes();
            return true;
        }

        /// <summary>Build a lookup dictionary of node ID to node data.</summary>
        public Dictionary<string, TaleNodeNodeData> BuildNodeMap()
        {
            var map = new Dictionary<string, TaleNodeNodeData>();
            if (Data?.Nodes == null) return map;
            foreach (var node in Data.Nodes)
            {
                if (!string.IsNullOrEmpty(node.Id))
                    map[node.Id] = node;
            }
            return map;
        }

        private void CountNodeTypes()
        {
            StartCount = DialogueNodeCount = ChoiceCount = 0;
            ConditionCount = EventCount = RandomCount = EndCount = 0;

            if (Data?.Nodes == null) return;
            foreach (var node in Data.Nodes)
            {
                switch (node.NodeType)
                {
                    case "start": StartCount++; break;
                    case "dialogue": DialogueNodeCount++; break;
                    case "choice": ChoiceCount++; break;
                    case "condition": ConditionCount++; break;
                    case "event": EventCount++; break;
                    case "random": RandomCount++; break;
                    case "end": EndCount++; break;
                }
            }
        }
    }
}
