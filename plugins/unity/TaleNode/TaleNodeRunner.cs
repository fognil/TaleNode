// TaleNode Runtime Dialogue Runner for Unity
// Pure C# class — no MonoBehaviour dependency.
// Uses Newtonsoft.Json (com.unity.nuget.newtonsoft-json).
//
// Usage:
//   var runner = new TaleNodeRunner();
//   runner.OnDialogueLine += (s, e) => Debug.Log($"{e.Speaker}: {e.Text}");
//   runner.LoadDialogue("Assets/Dialogues/intro.json");
//   runner.Start();

using System;
using System.Collections.Generic;
using System.IO;
using Newtonsoft.Json.Linq;

namespace TaleNode
{
    // --- Event Args ---

    public class DialogueLineEventArgs : EventArgs
    {
        public string Speaker { get; set; }
        public string Text { get; set; }
        public string Emotion { get; set; }
        public string Portrait { get; set; }
        public string Audio { get; set; }
        public string NodeId { get; set; }
    }

    public class ChoiceEventArgs : EventArgs
    {
        public string Prompt { get; set; }
        public List<string> Options { get; set; }
    }

    public class DialogueEndedEventArgs : EventArgs
    {
        public string Tag { get; set; }
    }

    public class EventTriggeredEventArgs : EventArgs
    {
        public string Action { get; set; }
        public string Key { get; set; }
        public object Value { get; set; }
    }

    public class VariableChangedEventArgs : EventArgs
    {
        public string Key { get; set; }
        public object Value { get; set; }
    }

    /// <summary>Runtime dialogue runner for TaleNode exported JSON.</summary>
    public class TaleNodeRunner
    {
        // --- Events ---

        public event EventHandler OnDialogueStarted;
        public event EventHandler<DialogueLineEventArgs> OnDialogueLine;
        public event EventHandler<ChoiceEventArgs> OnChoicePresented;
        public event EventHandler<DialogueEndedEventArgs> OnDialogueEnded;
        public event EventHandler<EventTriggeredEventArgs> OnEventTriggered;
        public event EventHandler<VariableChangedEventArgs> OnVariableChanged;

        // --- State ---

        private JObject _data;
        private Dictionary<string, JObject> _nodeMap = new Dictionary<string, JObject>();
        private Dictionary<string, TaleValue> _variables = new Dictionary<string, TaleValue>();
        private Dictionary<string, JObject> _characters = new Dictionary<string, JObject>();
        private string _currentNodeId = "";
        private bool _running;

        // Cached filtered options for the current choice node
        private List<JObject> _currentFilteredOptions;

        public bool IsRunning => _running;

        // --- Public API ---

        /// <summary>Load dialogue from a JSON file path. Returns true on success.</summary>
        public bool LoadDialogue(string jsonPath)
        {
            try
            {
                string content = File.ReadAllText(jsonPath);
                return LoadFromString(content);
            }
            catch (Exception ex)
            {
                UnityLog("Cannot open file: " + ex.Message);
                return false;
            }
        }

        /// <summary>Load dialogue from a JSON string. Returns true on success.</summary>
        public bool LoadFromString(string jsonString)
        {
            try
            {
                _data = JObject.Parse(jsonString);
            }
            catch
            {
                UnityLog("Invalid JSON");
                return false;
            }

            _nodeMap.Clear();
            _variables.Clear();
            _characters.Clear();
            _currentNodeId = "";
            _running = false;
            _currentFilteredOptions = null;

            // Build node map
            var nodes = _data["nodes"] as JArray;
            if (nodes != null)
            {
                foreach (var n in nodes)
                {
                    if (n is JObject node && node["id"] != null)
                        _nodeMap[node["id"].ToString()] = node;
                }
            }

            // Initialize variables
            var vars = _data["variables"] as JArray;
            if (vars != null)
            {
                foreach (var v in vars)
                {
                    if (v is JObject varObj && varObj["name"] != null)
                    {
                        string name = varObj["name"].ToString();
                        _variables[name] = JTokenToTaleValue(varObj["default"]);
                    }
                }
            }

            // Build character map
            var chars = _data["characters"] as JArray;
            if (chars != null)
            {
                foreach (var c in chars)
                {
                    if (c is JObject charObj && charObj["id"] != null)
                        _characters[charObj["id"].ToString()] = charObj;
                }
            }

            return true;
        }

        /// <summary>Start the dialogue. Optionally specify a start node ID.</summary>
        public void Start(string startNodeId = null)
        {
            if (_nodeMap.Count == 0)
            {
                UnityLog("No dialogue loaded");
                return;
            }

            if (!string.IsNullOrEmpty(startNodeId))
            {
                _currentNodeId = startNodeId;
            }
            else
            {
                _currentNodeId = "";
                foreach (var kv in _nodeMap)
                {
                    if (kv.Value["type"]?.ToString() == "start")
                    {
                        _currentNodeId = kv.Key;
                        break;
                    }
                }
                if (string.IsNullOrEmpty(_currentNodeId))
                {
                    UnityLog("No start node found");
                    return;
                }
            }

            _running = true;
            OnDialogueStarted?.Invoke(this, EventArgs.Empty);
            ProcessNode();
        }

        /// <summary>Advance to the next node (after a dialogue line).</summary>
        public void Advance()
        {
            if (!_running) return;
            var node = GetCurrentNode();
            if (node == null) { End(""); return; }
            GoTo(node["next"]?.ToString());
        }

        /// <summary>Choose an option by index (after OnChoicePresented).</summary>
        public void Choose(int optionIndex)
        {
            if (!_running) return;
            var node = GetCurrentNode();
            if (node == null || node["type"]?.ToString() != "choice") return;

            if (_currentFilteredOptions == null) return;
            if (optionIndex < 0 || optionIndex >= _currentFilteredOptions.Count)
            {
                UnityLog("Invalid option index: " + optionIndex);
                return;
            }

            var chosen = _currentFilteredOptions[optionIndex];
            GoTo(chosen["next"]?.ToString());
        }

        /// <summary>Get a runtime variable value.</summary>
        public TaleValue GetVariable(string name)
        {
            return _variables.TryGetValue(name, out var val) ? val : null;
        }

        /// <summary>Set a runtime variable value.</summary>
        public void SetVariable(string name, TaleValue value)
        {
            _variables[name] = value;
            OnVariableChanged?.Invoke(this, new VariableChangedEventArgs
            {
                Key = name,
                Value = TaleValueToObject(value)
            });
        }

        /// <summary>Stop the dialogue immediately.</summary>
        public void Stop()
        {
            _running = false;
            _currentNodeId = "";
            _currentFilteredOptions = null;
        }

        // --- Internal ---

        private JObject GetCurrentNode()
        {
            if (string.IsNullOrEmpty(_currentNodeId)) return null;
            return _nodeMap.TryGetValue(_currentNodeId, out var node) ? node : null;
        }

        private void GoTo(string nextId)
        {
            if (string.IsNullOrEmpty(nextId)) { End(""); return; }
            _currentNodeId = nextId;
            ProcessNode();
        }

        private void End(string tag)
        {
            _running = false;
            _currentNodeId = "";
            _currentFilteredOptions = null;
            OnDialogueEnded?.Invoke(this, new DialogueEndedEventArgs { Tag = tag ?? "" });
        }

        private void ProcessNode()
        {
            if (!_running) return;
            var node = GetCurrentNode();
            if (node == null) { End(""); return; }

            string nodeType = node["type"]?.ToString() ?? "";

            switch (nodeType)
            {
                case "start":
                    GoTo(node["next"]?.ToString());
                    break;

                case "dialogue":
                    ProcessDialogue(node);
                    break;

                case "choice":
                    ProcessChoice(node);
                    break;

                case "condition":
                    ProcessCondition(node);
                    break;

                case "event":
                    ProcessEvent(node);
                    break;

                case "random":
                    ProcessRandom(node);
                    break;

                case "end":
                    End(node["tag"]?.ToString() ?? "");
                    break;

                default:
                    UnityLog("Unknown node type: " + nodeType);
                    End("");
                    break;
            }
        }

        private void ProcessDialogue(JObject node)
        {
            string speakerId = node["speaker"]?.ToString() ?? "";
            string speakerName = speakerId;

            if (_characters.TryGetValue(speakerId, out var charObj))
                speakerName = charObj["name"]?.ToString() ?? speakerId;

            string rawText = node["text"]?.ToString() ?? "";
            string text = TaleNodeExpression.InterpolateText(rawText, _variables);
            string emotion = node["emotion"]?.ToString() ?? "";
            string portrait = node["portrait"]?.ToString() ?? "";
            string audio = node["audio"]?.ToString() ?? "";
            string nodeId = node["id"]?.ToString() ?? "";

            OnDialogueLine?.Invoke(this, new DialogueLineEventArgs
            {
                Speaker = speakerName,
                Text = text,
                Emotion = emotion,
                Portrait = portrait,
                Audio = audio,
                NodeId = nodeId
            });
        }

        private void ProcessChoice(JObject node)
        {
            string prompt = node["prompt"]?.ToString() ?? "";
            prompt = TaleNodeExpression.InterpolateText(prompt, _variables);

            var options = node["options"] as JArray;
            _currentFilteredOptions = new List<JObject>();
            var optionTexts = new List<string>();

            if (options != null)
            {
                foreach (var opt in options)
                {
                    if (opt is not JObject optObj) continue;

                    var cond = optObj["condition"] as JObject;
                    if (cond != null && cond.Type != JTokenType.Null)
                    {
                        if (!EvaluateCondition(cond)) continue;
                    }

                    _currentFilteredOptions.Add(optObj);
                    string rawText = optObj["text"]?.ToString() ?? "";
                    optionTexts.Add(TaleNodeExpression.InterpolateText(rawText, _variables));
                }
            }

            OnChoicePresented?.Invoke(this, new ChoiceEventArgs
            {
                Prompt = prompt,
                Options = optionTexts
            });
        }

        private void ProcessCondition(JObject node)
        {
            string variable = node["variable"]?.ToString() ?? "";
            string op = node["operator"]?.ToString() ?? "==";
            var value = node["value"];

            bool result = EvaluateConditionFields(variable, op, value);
            GoTo(result ? node["true_next"]?.ToString() : node["false_next"]?.ToString());
        }

        private void ProcessEvent(JObject node)
        {
            var actions = node["actions"] as JArray;
            if (actions != null)
            {
                foreach (var act in actions)
                {
                    if (act is not JObject actionObj) continue;

                    string actionType = actionObj["action"]?.ToString() ?? "";
                    string key = actionObj["key"]?.ToString() ?? "";
                    var value = actionObj["value"];

                    string normalized = actionType.ToLowerInvariant().Replace("_", "");
                    if (normalized == "setvariable")
                    {
                        var tv = JTokenToTaleValue(value);
                        _variables[key] = tv;
                        OnVariableChanged?.Invoke(this, new VariableChangedEventArgs
                        {
                            Key = key,
                            Value = TaleValueToObject(tv)
                        });
                    }
                    else
                    {
                        OnEventTriggered?.Invoke(this, new EventTriggeredEventArgs
                        {
                            Action = actionType,
                            Key = key,
                            Value = value?.ToObject<object>()
                        });
                    }
                }
            }

            GoTo(node["next"]?.ToString());
        }

        private void ProcessRandom(JObject node)
        {
            var branches = node["branches"] as JArray;
            if (branches == null || branches.Count == 0)
            {
                GoTo(node["next"]?.ToString());
                return;
            }

            double totalWeight = 0;
            foreach (var b in branches)
            {
                if (b is JObject branch)
                    totalWeight += branch["weight"]?.ToObject<double>() ?? 1.0;
            }

            if (totalWeight <= 0)
            {
                var rng = new Random();
                int idx = rng.Next(branches.Count);
                GoTo((branches[idx] as JObject)?["next"]?.ToString());
                return;
            }

            var rand = new Random();
            double roll = rand.NextDouble() * totalWeight;
            double cumulative = 0;
            foreach (var b in branches)
            {
                if (b is not JObject branch) continue;
                cumulative += branch["weight"]?.ToObject<double>() ?? 1.0;
                if (roll <= cumulative)
                {
                    GoTo(branch["next"]?.ToString());
                    return;
                }
            }

            // Fallback: last branch
            GoTo((branches[branches.Count - 1] as JObject)?["next"]?.ToString());
        }

        private bool EvaluateCondition(JObject cond)
        {
            string variable = cond["variable"]?.ToString() ?? "";
            string op = cond["operator"]?.ToString() ?? "==";
            var value = cond["value"];
            return EvaluateConditionFields(variable, op, value);
        }

        private bool EvaluateConditionFields(string variableName, string op, JToken value)
        {
            if (!_variables.TryGetValue(variableName, out var current))
                return false;

            var target = JTokenToTaleValue(value);

            switch (op)
            {
                case "==":
                    return LooseEqual(current, target);
                case "!=":
                    return !LooseEqual(current, target);
                case ">":
                    return current.ToDouble() > target.ToDouble();
                case "<":
                    return current.ToDouble() < target.ToDouble();
                case ">=":
                    return current.ToDouble() >= target.ToDouble();
                case "<=":
                    return current.ToDouble() <= target.ToDouble();
                case "contains":
                    return current.ToString().IndexOf(target.ToString(),
                        StringComparison.OrdinalIgnoreCase) >= 0;
                default:
                    return false;
            }
        }

        private static bool LooseEqual(TaleValue a, TaleValue b)
        {
            if (a.Type == TaleValue.ValueType.Bool && b.Type == TaleValue.ValueType.Bool)
                return a.BoolVal == b.BoolVal;
            if (a.Type == TaleValue.ValueType.Text && b.Type == TaleValue.ValueType.Text)
                return a.TextVal == b.TextVal;
            if (a.IsNumeric && b.IsNumeric)
                return Math.Abs(a.ToDouble() - b.ToDouble()) < 1e-9;
            return a.ToString() == b.ToString();
        }

        private static TaleValue JTokenToTaleValue(JToken token)
        {
            if (token == null || token.Type == JTokenType.Null)
                return TaleValue.FromInt(0);

            switch (token.Type)
            {
                case JTokenType.Boolean:
                    return TaleValue.FromBool(token.ToObject<bool>());
                case JTokenType.Integer:
                    return TaleValue.FromInt(token.ToObject<long>());
                case JTokenType.Float:
                    return TaleValue.FromFloat(token.ToObject<double>());
                case JTokenType.String:
                    return TaleValue.FromText(token.ToString());
                default:
                    return TaleValue.FromText(token.ToString());
            }
        }

        private static object TaleValueToObject(TaleValue v)
        {
            if (v == null) return null;
            switch (v.Type)
            {
                case TaleValue.ValueType.Bool: return v.BoolVal;
                case TaleValue.ValueType.Int: return v.IntVal;
                case TaleValue.ValueType.Float: return v.FloatVal;
                case TaleValue.ValueType.Text: return v.TextVal;
                default: return null;
            }
        }

        private static void UnityLog(string msg)
        {
#if UNITY_5_3_OR_NEWER
            UnityEngine.Debug.LogWarning("[TaleNodeRunner] " + msg);
#else
            Console.WriteLine("[TaleNodeRunner] " + msg);
#endif
        }
    }
}
