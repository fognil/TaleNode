## Runtime dialogue runner for TaleNode exported JSON.
## Loads exported dialogue JSON files and steps through nodes,
## emitting signals for each dialogue event.
##
## Usage:
##   var runner = TaleNodeRunner.new()
##   runner.dialogue_line.connect(_on_dialogue_line)
##   runner.choice_presented.connect(_on_choice)
##   runner.load_dialogue("res://dialogues/intro.json")
##   runner.start()
class_name TaleNodeRunner
extends RefCounted

# --- Signals ---

signal dialogue_started()
signal dialogue_ended(tag: String)
signal dialogue_line(speaker: String, text: String, emotion: String, portrait: String, audio: String, node_id: String)
signal choice_presented(prompt: String, options: Array)
signal variable_changed(key: String, value: Variant)
signal event_triggered(action: String, key: String, value: Variant)

# --- State ---

var _data: Dictionary = {}
var _node_map: Dictionary = {}  # id -> node dict
var _variables: Dictionary = {}
var _characters: Dictionary = {}  # id -> character dict
var _current_node_id: String = ""
var _running := false
var _expr := TaleNodeExpr.new()

# --- Public API ---

## Load dialogue from a JSON file path. Returns true on success.
func load_dialogue(json_path: String) -> bool:
	var file := FileAccess.open(json_path, FileAccess.READ)
	if file == null:
		push_error("TaleNodeRunner: Cannot open file: %s" % json_path)
		return false
	var content := file.get_as_text()
	file.close()
	return load_from_string(content)


## Load dialogue from a JSON string. Returns true on success.
func load_from_string(json_string: String) -> bool:
	var parsed = JSON.parse_string(json_string)
	if parsed == null or not (parsed is Dictionary):
		push_error("TaleNodeRunner: Invalid JSON")
		return false

	_data = parsed
	_node_map.clear()
	_variables.clear()
	_characters.clear()
	_current_node_id = ""
	_running = false

	# Build node map
	var nodes: Array = _data.get("nodes", [])
	for node in nodes:
		if node is Dictionary and node.has("id"):
			_node_map[node["id"]] = node

	# Initialize variables from defaults
	var vars: Array = _data.get("variables", [])
	for v in vars:
		if v is Dictionary and v.has("name"):
			_variables[v["name"]] = v.get("default", null)

	# Build character map
	var chars: Array = _data.get("characters", [])
	for c in chars:
		if c is Dictionary and c.has("id"):
			_characters[c["id"]] = c

	return true


## Start the dialogue. Optionally specify a start node ID.
## If not provided, finds the first "start" type node.
func start(start_node_id: String = "") -> void:
	if _node_map.is_empty():
		push_error("TaleNodeRunner: No dialogue loaded")
		return

	if start_node_id != "":
		_current_node_id = start_node_id
	else:
		# Find first start node
		for id in _node_map:
			var node: Dictionary = _node_map[id]
			if node.get("type", "") == "start":
				_current_node_id = id
				break
		if _current_node_id == "":
			push_error("TaleNodeRunner: No start node found")
			return

	_running = true
	dialogue_started.emit()
	_process_node()


## Advance to the next node (after a dialogue line).
func advance() -> void:
	if not _running:
		return
	var node := _get_current_node()
	if node.is_empty():
		_end("")
		return
	var next_id = node.get("next", null)
	_go_to(next_id)


## Choose an option by index (after choice_presented).
func choose(option_index: int) -> void:
	if not _running:
		return
	var node := _get_current_node()
	if node.is_empty() or node.get("type", "") != "choice":
		return
	var options: Array = node.get("options", [])

	# Build filtered options list matching what was presented
	var filtered := _filter_options(options)
	if option_index < 0 or option_index >= filtered.size():
		push_error("TaleNodeRunner: Invalid option index: %d" % option_index)
		return

	var chosen: Dictionary = filtered[option_index]
	var next_id = chosen.get("next", null)
	_go_to(next_id)


## Get a runtime variable value.
func get_variable(name: String) -> Variant:
	return _variables.get(name, null)


## Set a runtime variable value.
func set_variable(name: String, value: Variant) -> void:
	_variables[name] = value
	variable_changed.emit(name, value)


## Returns true if the dialogue is currently running.
func is_running() -> bool:
	return _running


## Stop the dialogue immediately.
func stop() -> void:
	_running = false
	_current_node_id = ""


# --- Internal ---

func _get_current_node() -> Dictionary:
	if _current_node_id == "" or not _node_map.has(_current_node_id):
		return {}
	return _node_map[_current_node_id]


func _go_to(next_id: Variant) -> void:
	if next_id == null or (next_id is String and next_id == ""):
		_end("")
		return
	if not (next_id is String):
		_end("")
		return
	_current_node_id = next_id
	_process_node()


func _end(tag: String) -> void:
	_running = false
	_current_node_id = ""
	dialogue_ended.emit(tag)


func _process_node() -> void:
	if not _running:
		return

	var node := _get_current_node()
	if node.is_empty():
		_end("")
		return

	var node_type: String = node.get("type", "")

	match node_type:
		"start":
			_go_to(node.get("next", null))

		"dialogue":
			_process_dialogue(node)

		"choice":
			_process_choice(node)

		"condition":
			_process_condition(node)

		"event":
			_process_event(node)

		"random":
			_process_random(node)

		"end":
			var tag: String = node.get("tag", "")
			_end(tag)

		_:
			push_error("TaleNodeRunner: Unknown node type: %s" % node_type)
			_end("")


func _process_dialogue(node: Dictionary) -> void:
	var speaker_id: String = node.get("speaker", "")
	var speaker_name := speaker_id

	# Resolve character name from ID
	if _characters.has(speaker_id):
		speaker_name = _characters[speaker_id].get("name", speaker_id)

	var raw_text: String = node.get("text", "")
	var text := _expr.interpolate_text(raw_text, _variables)
	var emotion: String = node.get("emotion", "")
	var portrait: String = node.get("portrait", "")
	var audio: String = node.get("audio", "")
	var node_id: String = node.get("id", "")

	dialogue_line.emit(speaker_name, text, emotion, portrait, audio, node_id)
	# Wait for advance() call


func _process_choice(node: Dictionary) -> void:
	var prompt: String = node.get("prompt", "")
	prompt = _expr.interpolate_text(prompt, _variables)

	var options: Array = node.get("options", [])
	var filtered := _filter_options(options)

	var option_texts: Array = []
	for opt in filtered:
		var raw_text: String = opt.get("text", "")
		option_texts.append(_expr.interpolate_text(raw_text, _variables))

	choice_presented.emit(prompt, option_texts)
	# Wait for choose() call


func _filter_options(options: Array) -> Array:
	var filtered: Array = []
	for opt in options:
		if not (opt is Dictionary):
			continue
		var cond = opt.get("condition", null)
		if cond == null or not (cond is Dictionary):
			filtered.append(opt)
			continue
		if _evaluate_condition(cond):
			filtered.append(opt)
	return filtered


func _process_condition(node: Dictionary) -> void:
	var variable_name: String = node.get("variable", "")
	var operator: String = node.get("operator", "==")
	var value = node.get("value", null)

	var result := _evaluate_condition_fields(variable_name, operator, value)

	if result:
		_go_to(node.get("true_next", null))
	else:
		_go_to(node.get("false_next", null))


func _process_event(node: Dictionary) -> void:
	var actions: Array = node.get("actions", [])
	for action_data in actions:
		if not (action_data is Dictionary):
			continue
		var action_type: String = action_data.get("action", "")
		var key: String = action_data.get("key", "")
		var value = action_data.get("value", null)

		# Normalize action type
		var normalized := action_type.to_lower().replace("_", "")
		if normalized == "setvariable":
			_variables[key] = value
			variable_changed.emit(key, value)
		else:
			event_triggered.emit(action_type, key, value)

	_go_to(node.get("next", null))


func _process_random(node: Dictionary) -> void:
	var branches: Array = node.get("branches", [])
	if branches.is_empty():
		_go_to(node.get("next", null))
		return

	# Calculate total weight
	var total_weight := 0.0
	for branch in branches:
		if branch is Dictionary:
			total_weight += float(branch.get("weight", 1.0))

	if total_weight <= 0.0:
		# Equal weight fallback
		var idx := randi() % branches.size()
		var chosen: Dictionary = branches[idx]
		_go_to(chosen.get("next", null))
		return

	# Weighted random pick
	var roll := randf() * total_weight
	var cumulative := 0.0
	for branch in branches:
		if not (branch is Dictionary):
			continue
		cumulative += float(branch.get("weight", 1.0))
		if roll <= cumulative:
			_go_to(branch.get("next", null))
			return

	# Fallback: last branch
	var last: Dictionary = branches.back()
	_go_to(last.get("next", null))


## Evaluate a condition object {"variable", "operator", "value"}.
func _evaluate_condition(cond: Dictionary) -> bool:
	var variable_name: String = cond.get("variable", "")
	var operator: String = cond.get("operator", "==")
	var value = cond.get("value", null)
	return _evaluate_condition_fields(variable_name, operator, value)


## Evaluate a condition by variable name, operator, and target value.
func _evaluate_condition_fields(variable_name: String, operator: String, value: Variant) -> bool:
	var current = _variables.get(variable_name, null)
	if current == null:
		return false

	match operator:
		"==":
			return _loose_equal(current, value)
		"!=":
			return not _loose_equal(current, value)
		">":
			return _to_float(current) > _to_float(value)
		"<":
			return _to_float(current) < _to_float(value)
		">=":
			return _to_float(current) >= _to_float(value)
		"<=":
			return _to_float(current) <= _to_float(value)
		"contains":
			return str(current).to_lower().contains(str(value).to_lower())

	return false


func _loose_equal(a: Variant, b: Variant) -> bool:
	if a is bool and b is bool:
		return a == b
	if a is String and b is String:
		return a == b
	if _is_numeric_val(a) and _is_numeric_val(b):
		return abs(_to_float(a) - _to_float(b)) < 0.00001
	return str(a) == str(b)


func _is_numeric_val(val: Variant) -> bool:
	return val is int or val is float or val is bool


func _to_float(val: Variant) -> float:
	if val is int:
		return float(val)
	if val is float:
		return val
	if val is bool:
		return 1.0 if val else 0.0
	if val is String:
		if val.is_valid_float():
			return val.to_float()
		if val.is_valid_int():
			return float(val.to_int())
	return 0.0
