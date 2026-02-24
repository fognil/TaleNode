## Expression engine for TaleNode dialogue runner.
## Parses and evaluates expressions with variables, arithmetic,
## comparisons, boolean logic, and text interpolation.
##
## Usage:
##   var expr = TaleNodeExpr.new()
##   var ast = expr.parse("gold >= 100")
##   var result = expr.evaluate(ast, {"gold": 50})  # false
##   var text = expr.interpolate_text("You have {gold} gold.", {"gold": 50})
class_name TaleNodeExpr
extends RefCounted

# --- Tokenizer ---

enum TokenType {
	INT, FLOAT, STR, BOOL, IDENT,
	PLUS, MINUS, STAR, SLASH, PERCENT,
	EQ, NEQ, GT, LT, GTE, LTE,
	AND, OR, NOT,
	LPAREN, RPAREN
}

## Tokenize an expression string into an array of token dictionaries.
func tokenize(input: String) -> Array:
	var tokens: Array = []
	var chars := input
	var i := 0
	var length := chars.length()

	while i < length:
		var c := chars[i]

		if c == " " or c == "\t" or c == "\r" or c == "\n":
			i += 1
			continue

		if c == "(":
			tokens.append({"type": TokenType.LPAREN})
			i += 1
			continue
		if c == ")":
			tokens.append({"type": TokenType.RPAREN})
			i += 1
			continue
		if c == "+":
			tokens.append({"type": TokenType.PLUS})
			i += 1
			continue
		if c == "*":
			tokens.append({"type": TokenType.STAR})
			i += 1
			continue
		if c == "/":
			tokens.append({"type": TokenType.SLASH})
			i += 1
			continue
		if c == "%":
			tokens.append({"type": TokenType.PERCENT})
			i += 1
			continue

		if c == "-":
			var is_unary := tokens.is_empty() or _is_unary_context(tokens.back())
			if is_unary:
				tokens.append({"type": TokenType.MINUS, "unary": true})
			else:
				tokens.append({"type": TokenType.MINUS, "unary": false})
			i += 1
			continue

		if c == "!" and i + 1 < length and chars[i + 1] == "=":
			tokens.append({"type": TokenType.NEQ})
			i += 2
			continue
		if c == "!":
			tokens.append({"type": TokenType.NOT})
			i += 1
			continue

		if c == "=" and i + 1 < length and chars[i + 1] == "=":
			tokens.append({"type": TokenType.EQ})
			i += 2
			continue

		if c == ">" and i + 1 < length and chars[i + 1] == "=":
			tokens.append({"type": TokenType.GTE})
			i += 2
			continue
		if c == ">":
			tokens.append({"type": TokenType.GT})
			i += 1
			continue

		if c == "<" and i + 1 < length and chars[i + 1] == "=":
			tokens.append({"type": TokenType.LTE})
			i += 2
			continue
		if c == "<":
			tokens.append({"type": TokenType.LT})
			i += 1
			continue

		if c == "&" and i + 1 < length and chars[i + 1] == "&":
			tokens.append({"type": TokenType.AND})
			i += 2
			continue

		if c == "|" and i + 1 < length and chars[i + 1] == "|":
			tokens.append({"type": TokenType.OR})
			i += 2
			continue

		if c == "\"":
			i += 1
			var start := i
			while i < length and chars[i] != "\"":
				i += 1
			if i >= length:
				push_error("TaleNodeExpr: Unterminated string literal")
				return []
			var s := chars.substr(start, i - start)
			tokens.append({"type": TokenType.STR, "value": s})
			i += 1
			continue

		if _is_digit(c):
			var start := i
			while i < length and _is_digit(chars[i]):
				i += 1
			if i < length and chars[i] == "." and i + 1 < length and _is_digit(chars[i + 1]):
				i += 1
				while i < length and _is_digit(chars[i]):
					i += 1
				var s := chars.substr(start, i - start)
				tokens.append({"type": TokenType.FLOAT, "value": s.to_float()})
			else:
				var s := chars.substr(start, i - start)
				tokens.append({"type": TokenType.INT, "value": s.to_int()})
			continue

		if _is_alpha(c) or c == "_":
			var start := i
			while i < length and (_is_alnum(chars[i]) or chars[i] == "_"):
				i += 1
			var word := chars.substr(start, i - start)
			if word == "true":
				tokens.append({"type": TokenType.BOOL, "value": true})
			elif word == "false":
				tokens.append({"type": TokenType.BOOL, "value": false})
			else:
				tokens.append({"type": TokenType.IDENT, "value": word})
			continue

		push_error("TaleNodeExpr: Unexpected character: '%s'" % c)
		return []

	return tokens


func _is_unary_context(last_token: Dictionary) -> bool:
	var t: TokenType = last_token["type"]
	return (t == TokenType.PLUS or t == TokenType.MINUS
		or t == TokenType.STAR or t == TokenType.SLASH
		or t == TokenType.PERCENT or t == TokenType.NOT
		or t == TokenType.LPAREN
		or t == TokenType.EQ or t == TokenType.NEQ
		or t == TokenType.GT or t == TokenType.LT
		or t == TokenType.GTE or t == TokenType.LTE
		or t == TokenType.AND or t == TokenType.OR)


func _is_digit(c: String) -> bool:
	return c >= "0" and c <= "9"

func _is_alpha(c: String) -> bool:
	return (c >= "a" and c <= "z") or (c >= "A" and c <= "Z")

func _is_alnum(c: String) -> bool:
	return _is_digit(c) or _is_alpha(c)

# --- Parser ---
# Builds AST as dictionaries:
#   {"type": "literal", "value": <Variant>}
#   {"type": "variable", "name": <String>}
#   {"type": "binary", "op": <String>, "left": <AST>, "right": <AST>}
#   {"type": "not", "expr": <AST>}
#   {"type": "neg", "expr": <AST>}

var _tokens: Array = []
var _pos: int = 0

## Parse an expression string into an AST dictionary.
## Returns null on error.
func parse(input: String) -> Variant:
	_tokens = tokenize(input)
	_pos = 0
	if _tokens.is_empty():
		return null
	var ast := _parse_or()
	if ast == null:
		return null
	if _pos < _tokens.size():
		push_error("TaleNodeExpr: Unexpected token after expression")
		return null
	return ast

func _peek() -> Variant:
	if _pos >= _tokens.size():
		return null
	return _tokens[_pos]

func _advance() -> Dictionary:
	var tok: Dictionary = _tokens[_pos]
	_pos += 1
	return tok

func _parse_or() -> Variant:
	var left := _parse_and()
	if left == null:
		return null
	while _peek() != null and _peek()["type"] == TokenType.OR:
		_advance()
		var right := _parse_and()
		if right == null:
			return null
		left = {"type": "binary", "op": "||", "left": left, "right": right}
	return left

func _parse_and() -> Variant:
	var left := _parse_equality()
	if left == null:
		return null
	while _peek() != null and _peek()["type"] == TokenType.AND:
		_advance()
		var right := _parse_equality()
		if right == null:
			return null
		left = {"type": "binary", "op": "&&", "left": left, "right": right}
	return left

func _parse_equality() -> Variant:
	var left := _parse_comparison()
	if left == null:
		return null
	while _peek() != null and (_peek()["type"] == TokenType.EQ or _peek()["type"] == TokenType.NEQ):
		var tok := _advance()
		var op_str := "==" if tok["type"] == TokenType.EQ else "!="
		var right := _parse_comparison()
		if right == null:
			return null
		left = {"type": "binary", "op": op_str, "left": left, "right": right}
	return left

func _parse_comparison() -> Variant:
	var left := _parse_additive()
	if left == null:
		return null
	while _peek() != null and (_peek()["type"] == TokenType.GT or _peek()["type"] == TokenType.LT or _peek()["type"] == TokenType.GTE or _peek()["type"] == TokenType.LTE):
		var tok := _advance()
		var op_str: String
		match tok["type"]:
			TokenType.GT: op_str = ">"
			TokenType.LT: op_str = "<"
			TokenType.GTE: op_str = ">="
			TokenType.LTE: op_str = "<="
		var right := _parse_additive()
		if right == null:
			return null
		left = {"type": "binary", "op": op_str, "left": left, "right": right}
	return left

func _parse_additive() -> Variant:
	var left := _parse_multiplicative()
	if left == null:
		return null
	while _peek() != null and _peek()["type"] == TokenType.PLUS:
		_advance()
		var right := _parse_multiplicative()
		if right == null:
			return null
		left = {"type": "binary", "op": "+", "left": left, "right": right}
	# Handle binary minus (not unary)
	while _peek() != null and _peek()["type"] == TokenType.MINUS and not _peek().get("unary", false):
		_advance()
		var right := _parse_multiplicative()
		if right == null:
			return null
		left = {"type": "binary", "op": "-", "left": left, "right": right}
	return left

func _parse_multiplicative() -> Variant:
	var left := _parse_unary()
	if left == null:
		return null
	while _peek() != null and (_peek()["type"] == TokenType.STAR or _peek()["type"] == TokenType.SLASH or _peek()["type"] == TokenType.PERCENT):
		var tok := _advance()
		var op_str: String
		match tok["type"]:
			TokenType.STAR: op_str = "*"
			TokenType.SLASH: op_str = "/"
			TokenType.PERCENT: op_str = "%"
		var right := _parse_unary()
		if right == null:
			return null
		left = {"type": "binary", "op": op_str, "left": left, "right": right}
	return left

func _parse_unary() -> Variant:
	if _peek() != null and _peek()["type"] == TokenType.NOT:
		_advance()
		var expr := _parse_unary()
		if expr == null:
			return null
		return {"type": "not", "expr": expr}
	if _peek() != null and _peek()["type"] == TokenType.MINUS and _peek().get("unary", false):
		_advance()
		var expr := _parse_unary()
		if expr == null:
			return null
		return {"type": "neg", "expr": expr}
	return _parse_primary()

func _parse_primary() -> Variant:
	var tok := _peek()
	if tok == null:
		push_error("TaleNodeExpr: Unexpected end of expression")
		return null

	match tok["type"]:
		TokenType.INT:
			_advance()
			return {"type": "literal", "value": tok["value"]}
		TokenType.FLOAT:
			_advance()
			return {"type": "literal", "value": tok["value"]}
		TokenType.STR:
			_advance()
			return {"type": "literal", "value": tok["value"]}
		TokenType.BOOL:
			_advance()
			return {"type": "literal", "value": tok["value"]}
		TokenType.IDENT:
			_advance()
			return {"type": "variable", "name": tok["value"]}
		TokenType.LPAREN:
			_advance()
			var expr := _parse_or()
			if expr == null:
				return null
			if _peek() == null or _peek()["type"] != TokenType.RPAREN:
				push_error("TaleNodeExpr: Expected ')'")
				return null
			_advance()
			return expr

	push_error("TaleNodeExpr: Unexpected token")
	return null


# --- Evaluator ---

## Evaluate an AST node with given variables dictionary.
## Returns the result as a Variant (int, float, bool, or String).
func evaluate(ast: Dictionary, variables: Dictionary) -> Variant:
	match ast["type"]:
		"literal":
			return ast["value"]
		"variable":
			var name: String = ast["name"]
			if variables.has(name):
				return variables[name]
			push_error("TaleNodeExpr: Undefined variable: %s" % name)
			return null
		"not":
			var val = evaluate(ast["expr"], variables)
			return not to_bool(val)
		"neg":
			var val = evaluate(ast["expr"], variables)
			if val is int:
				return -val
			if val is float:
				return -val
			if val is bool:
				return -1 if val else 0
			push_error("TaleNodeExpr: Cannot negate text")
			return null
		"binary":
			return _eval_binary(ast, variables)

	push_error("TaleNodeExpr: Unknown AST node type: %s" % ast["type"])
	return null


func _eval_binary(ast: Dictionary, variables: Dictionary) -> Variant:
	var op: String = ast["op"]
	var lv = evaluate(ast["left"], variables)

	# Short-circuit for && and ||
	if op == "&&":
		if not to_bool(lv):
			return false
		var rv = evaluate(ast["right"], variables)
		return to_bool(rv)
	if op == "||":
		if to_bool(lv):
			return true
		var rv = evaluate(ast["right"], variables)
		return to_bool(rv)

	var rv = evaluate(ast["right"], variables)

	# String concatenation
	if op == "+" and (lv is String or rv is String):
		return to_string_val(lv) + to_string_val(rv)

	# Arithmetic
	if op in ["+", "-", "*", "/", "%"]:
		return _eval_arithmetic(op, lv, rv)

	# Equality
	if op == "==":
		return _values_equal(lv, rv)
	if op == "!=":
		return not _values_equal(lv, rv)

	# Comparison
	if op in [">", "<", ">=", "<="]:
		return _eval_comparison(op, lv, rv)

	push_error("TaleNodeExpr: Unknown operator: %s" % op)
	return null


func _eval_arithmetic(op: String, lv: Variant, rv: Variant) -> Variant:
	var l := _to_number(lv)
	var r := _to_number(rv)

	# Promote to float if either is float
	if l is float or r is float:
		var lf: float = float(l)
		var rf: float = float(r)
		match op:
			"+": return lf + rf
			"-": return lf - rf
			"*": return lf * rf
			"/": return lf / rf
			"%": return fmod(lf, rf)
	else:
		var li: int = int(l)
		var ri: int = int(r)
		match op:
			"+": return li + ri
			"-": return li - ri
			"*": return li * ri
			"/":
				if ri == 0:
					push_error("TaleNodeExpr: Division by zero")
					return 0
				return li / ri
			"%":
				if ri == 0:
					push_error("TaleNodeExpr: Division by zero")
					return 0
				return li % ri
	return 0


func _to_number(val: Variant) -> Variant:
	if val is int:
		return val
	if val is float:
		return val
	if val is bool:
		return 1 if val else 0
	return 0


func _values_equal(lv: Variant, rv: Variant) -> bool:
	if lv is bool and rv is bool:
		return lv == rv
	if lv is String and rv is String:
		return lv == rv
	# Numeric comparison
	if _is_numeric(lv) and _is_numeric(rv):
		return abs(float(lv) - float(rv)) < 0.00001
	return to_string_val(lv) == to_string_val(rv)


func _eval_comparison(op: String, lv: Variant, rv: Variant) -> bool:
	# Try numeric first
	if _is_numeric(lv) and _is_numeric(rv):
		var l := float(lv)
		var r := float(rv)
		match op:
			">": return l > r
			"<": return l < r
			">=": return l >= r
			"<=": return l <= r
	# Fall back to string comparison
	var ls := to_string_val(lv)
	var rs := to_string_val(rv)
	match op:
		">": return ls > rs
		"<": return ls < rs
		">=": return ls >= rs
		"<=": return ls <= rs
	return false


func _is_numeric(val: Variant) -> bool:
	return val is int or val is float or val is bool


## Convert any value to boolean for condition evaluation.
## False: 0, 0.0, "", "false", null. Everything else is true.
func to_bool(value: Variant) -> bool:
	if value == null:
		return false
	if value is bool:
		return value
	if value is int:
		return value != 0
	if value is float:
		return value != 0.0
	if value is String:
		return value != "" and value != "false"
	return true


## Convert any value to a display string.
func to_string_val(value: Variant) -> String:
	if value == null:
		return "null"
	if value is bool:
		return "true" if value else "false"
	return str(value)


# --- Text Interpolation ---

## Parse `{expr}` and `{if cond}...{else}...{/if}` in text.
## Returns the interpolated result string.
func interpolate_text(text: String, variables: Dictionary) -> String:
	# Fast path: no braces
	if not "{" in text:
		return text

	var result := ""
	var i := 0
	var length := text.length()

	while i < length:
		if text[i] == "{":
			i += 1
			# Skip whitespace
			while i < length and text[i] in [" ", "\t"]:
				i += 1

			# Check for {/if} — should not appear at top level
			if i < length and text.substr(i).begins_with("/if"):
				return text  # Malformed, return original

			# Check for {else} — should not appear at top level
			if i < length and text.substr(i).begins_with("else"):
				var after_else := i + 4
				while after_else < length and text[after_else] in [" ", "\t"]:
					after_else += 1
				if after_else < length and text[after_else] == "}":
					return text  # Malformed

			# Check for {if ...}
			if text.substr(i).begins_with("if ") or text.substr(i).begins_with("if\t"):
				i += 2
				while i < length and text[i] in [" ", "\t"]:
					i += 1
				var cond_result := _read_until_close(text, i)
				if cond_result[0] == "":
					return text  # Parse error
				var cond_str: String = cond_result[0]
				i = cond_result[1]

				var cond_ast = parse(cond_str)
				if cond_ast == null:
					return text

				var cond_val := to_bool(evaluate(cond_ast, variables))

				# Parse then-branch
				var then_result := _parse_branch(text, i, variables)
				if then_result == null:
					return text
				var then_text: String = then_result["text"]
				i = then_result["pos"]
				var stopped_at: String = then_result["stopped"]

				var else_text := ""
				if stopped_at == "else":
					var else_result := _parse_branch(text, i, variables)
					if else_result == null:
						return text
					else_text = else_result["text"]
					i = else_result["pos"]

				result += then_text if cond_val else else_text
			else:
				# Regular expression {expr}
				var expr_result := _read_until_close(text, i)
				if expr_result[0] == "":
					return text  # Parse error
				var expr_str: String = expr_result[0]
				i = expr_result[1]

				var ast = parse(expr_str.strip_edges())
				if ast == null:
					result += "???"
				else:
					var val = evaluate(ast, variables)
					if val == null:
						result += "???"
					else:
						result += to_string_val(val)
		else:
			result += text[i]
			i += 1

	return result


## Read text until } respecting nested braces. Returns [content, new_pos].
func _read_until_close(text: String, start: int) -> Array:
	var depth := 1
	var i := start
	var length := text.length()
	while i < length:
		if text[i] == "{":
			depth += 1
		elif text[i] == "}":
			depth -= 1
			if depth == 0:
				var content := text.substr(start, i - start).strip_edges()
				return [content, i + 1]
		i += 1
	return ["", start]  # Unterminated


## Parse a conditional branch (then or else) until {else} or {/if}.
## Returns {"text": <interpolated>, "pos": <int>, "stopped": "else"|"endif"}
## or null on error.
func _parse_branch(text: String, start: int, variables: Dictionary) -> Variant:
	var result := ""
	var i := start
	var length := text.length()

	while i < length:
		if text[i] == "{":
			var save := i
			i += 1
			while i < length and text[i] in [" ", "\t"]:
				i += 1

			# Check for {/if}
			if text.substr(i).begins_with("/if"):
				i += 3
				while i < length and text[i] in [" ", "\t"]:
					i += 1
				if i < length and text[i] == "}":
					i += 1
				return {"text": result, "pos": i, "stopped": "endif"}

			# Check for {else}
			if text.substr(i).begins_with("else"):
				var after := i + 4
				while after < length and text[after] in [" ", "\t"]:
					after += 1
				if after < length and text[after] == "}":
					i = after + 1
					return {"text": result, "pos": i, "stopped": "else"}

			# Check for nested {if}
			if text.substr(i).begins_with("if ") or text.substr(i).begins_with("if\t"):
				i += 2
				while i < length and text[i] in [" ", "\t"]:
					i += 1
				var cond_result := _read_until_close(text, i)
				if cond_result[0] == "":
					return null
				var cond_str: String = cond_result[0]
				i = cond_result[1]

				var cond_ast = parse(cond_str)
				if cond_ast == null:
					return null
				var cond_val := to_bool(evaluate(cond_ast, variables))

				var then_r := _parse_branch(text, i, variables)
				if then_r == null:
					return null
				i = then_r["pos"]

				var else_t := ""
				if then_r["stopped"] == "else":
					var else_r := _parse_branch(text, i, variables)
					if else_r == null:
						return null
					else_t = else_r["text"]
					i = else_r["pos"]

				result += then_r["text"] if cond_val else else_t
				continue

			# Regular expression
			var expr_result := _read_until_close(text, i)
			if expr_result[0] == "":
				return null
			var expr_str: String = expr_result[0]
			i = expr_result[1]

			var ast = parse(expr_str.strip_edges())
			if ast == null:
				result += "???"
			else:
				var val = evaluate(ast, variables)
				if val == null:
					result += "???"
				else:
					result += to_string_val(val)
		else:
			result += text[i]
			i += 1

	# Reached end without finding {/if} or {else}
	return null
