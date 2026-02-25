// TaleNode — Serializable data classes for dialogue JSON.
// Maps directly to the exported JSON schema from TaleNode desktop app.

using System;
using System.Collections.Generic;
using Newtonsoft.Json;

namespace TaleNode
{
    [Serializable]
    public class TaleNodeDialogueData
    {
        [JsonProperty("version")] public string Version = "";
        [JsonProperty("name")] public string Name = "";
        [JsonProperty("default_locale")] public string DefaultLocale;
        [JsonProperty("locales")] public List<string> Locales;
        [JsonProperty("variables")] public List<TaleNodeVariableData> Variables = new();
        [JsonProperty("characters")] public List<TaleNodeCharacterData> Characters = new();
        [JsonProperty("nodes")] public List<TaleNodeNodeData> Nodes = new();
        [JsonProperty("strings")] public Dictionary<string, Dictionary<string, string>> Strings;
    }

    [Serializable]
    public class TaleNodeVariableData
    {
        [JsonProperty("name")] public string Name = "";
        [JsonProperty("type")] public string Type = "";
        [JsonProperty("default")] public object Default;
    }

    [Serializable]
    public class TaleNodeCharacterData
    {
        [JsonProperty("id")] public string Id = "";
        [JsonProperty("name")] public string Name = "";
        [JsonProperty("color")] public string Color = "#FFFFFF";
        [JsonProperty("portrait")] public string Portrait;
    }

    [Serializable]
    public class TaleNodeNodeData
    {
        [JsonProperty("id")] public string Id = "";
        [JsonProperty("type")] public string NodeType = "";

        // Dialogue fields
        [JsonProperty("speaker")] public string Speaker;
        [JsonProperty("text")] public string Text;
        [JsonProperty("emotion")] public string Emotion;
        [JsonProperty("portrait")] public string Portrait;
        [JsonProperty("audio")] public string Audio;

        // Choice fields
        [JsonProperty("prompt")] public string Prompt;
        [JsonProperty("options")] public List<TaleNodeOptionData> Options;

        // Condition fields
        [JsonProperty("variable")] public string Variable;
        [JsonProperty("operator")] public string Operator;
        [JsonProperty("value")] public object Value;
        [JsonProperty("true_next")] public string TrueNext;
        [JsonProperty("false_next")] public string FalseNext;

        // Event fields
        [JsonProperty("actions")] public List<TaleNodeActionData> Actions;

        // Random fields
        [JsonProperty("branches")] public List<TaleNodeBranchData> Branches;

        // End fields
        [JsonProperty("tag")] public string Tag;

        // SubGraph fields
        [JsonProperty("name")] public string SubGraphName;

        // Common navigation
        [JsonProperty("next")] public string Next;
    }

    [Serializable]
    public class TaleNodeOptionData
    {
        [JsonProperty("text")] public string Text = "";
        [JsonProperty("next")] public string Next;
        [JsonProperty("condition")] public TaleNodeConditionData Condition;
    }

    [Serializable]
    public class TaleNodeConditionData
    {
        [JsonProperty("variable")] public string Variable = "";
        [JsonProperty("operator")] public string Operator = "==";
        [JsonProperty("value")] public object Value;
    }

    [Serializable]
    public class TaleNodeActionData
    {
        [JsonProperty("action")] public string Action = "";
        [JsonProperty("key")] public string Key = "";
        [JsonProperty("value")] public object Value;
    }

    [Serializable]
    public class TaleNodeBranchData
    {
        [JsonProperty("weight")] public float Weight = 1f;
        [JsonProperty("next")] public string Next;
    }
}
