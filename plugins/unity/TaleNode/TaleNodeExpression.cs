// TaleNode Expression Engine for Unity
// Parses and evaluates expressions with variables, arithmetic,
// comparisons, boolean logic, and text interpolation.
//
// Usage:
//   var result = TaleNodeExpression.Evaluate("gold >= 100", variables);
//   bool check = TaleNodeExpression.EvaluateBool("has_key && level > 5", variables);
//   string text = TaleNodeExpression.InterpolateText("You have {gold} gold.", variables);

using System;
using System.Collections.Generic;
using System.Globalization;
using System.Text;

namespace TaleNode
{
    /// <summary>Represents a dynamically-typed dialogue variable value.</summary>
    public class TaleValue
    {
        public enum ValueType { Bool, Int, Float, Text }

        public ValueType Type { get; private set; }
        public bool BoolVal { get; private set; }
        public long IntVal { get; private set; }
        public double FloatVal { get; private set; }
        public string TextVal { get; private set; }

        private TaleValue() { TextVal = ""; }

        public static TaleValue FromBool(bool v) => new TaleValue { Type = ValueType.Bool, BoolVal = v };
        public static TaleValue FromInt(long v) => new TaleValue { Type = ValueType.Int, IntVal = v };
        public static TaleValue FromFloat(double v) => new TaleValue { Type = ValueType.Float, FloatVal = v };
        public static TaleValue FromText(string v) => new TaleValue { Type = ValueType.Text, TextVal = v ?? "" };

        /// <summary>Create a TaleValue from a System.Object (JSON parsed value).</summary>
        public static TaleValue FromObject(object obj)
        {
            if (obj == null) return FromInt(0);
            if (obj is bool b) return FromBool(b);
            if (obj is long l) return FromInt(l);
            if (obj is int i) return FromInt(i);
            if (obj is double d) return FromFloat(d);
            if (obj is float f) return FromFloat(f);
            if (obj is string s)
            {
                if (long.TryParse(s, out long li)) return FromInt(li);
                if (double.TryParse(s, NumberStyles.Float, CultureInfo.InvariantCulture, out double di))
                    return FromFloat(di);
                return FromText(s);
            }
            return FromText(obj.ToString());
        }

        public bool ToBool()
        {
            switch (Type)
            {
                case ValueType.Bool: return BoolVal;
                case ValueType.Int: return IntVal != 0;
                case ValueType.Float: return FloatVal != 0.0;
                case ValueType.Text: return !string.IsNullOrEmpty(TextVal) && TextVal != "false";
                default: return false;
            }
        }

        public double ToDouble()
        {
            switch (Type)
            {
                case ValueType.Bool: return BoolVal ? 1.0 : 0.0;
                case ValueType.Int: return IntVal;
                case ValueType.Float: return FloatVal;
                default: return 0.0;
            }
        }

        public override string ToString()
        {
            switch (Type)
            {
                case ValueType.Bool: return BoolVal ? "true" : "false";
                case ValueType.Int: return IntVal.ToString();
                case ValueType.Float: return FloatVal.ToString(CultureInfo.InvariantCulture);
                case ValueType.Text: return TextVal;
                default: return "";
            }
        }

        public bool IsNumeric => Type == ValueType.Bool || Type == ValueType.Int || Type == ValueType.Float;
    }

    /// <summary>Expression parser, evaluator, and text interpolation engine.</summary>
    public static class TaleNodeExpression
    {
        // --- Token types ---
        private enum TokenType
        {
            Int, Float, Str, Bool, Ident,
            Plus, Minus, Star, Slash, Percent,
            Eq, Neq, Gt, Lt, Gte, Lte,
            And, Or, Not,
            LParen, RParen
        }

        private struct Token
        {
            public TokenType Type;
            public object Value; // long, double, string, bool, or null
            public bool IsUnaryMinus;
        }

        // --- AST ---
        private abstract class AstNode { }
        private class LiteralNode : AstNode { public TaleValue Value; }
        private class VariableNode : AstNode { public string Name; }
        private class BinaryNode : AstNode
        {
            public string Op;
            public AstNode Left, Right;
        }
        private class NotNode : AstNode { public AstNode Expr; }
        private class NegNode : AstNode { public AstNode Expr; }

        // --- Public API ---

        public static TaleValue Evaluate(string expression, Dictionary<string, TaleValue> variables)
        {
            var tokens = Tokenize(expression);
            if (tokens == null || tokens.Count == 0) return TaleValue.FromInt(0);
            var parser = new Parser(tokens);
            var ast = parser.ParseExpr();
            if (ast == null || parser.Pos < tokens.Count) return TaleValue.FromInt(0);
            return Eval(ast, variables);
        }

        public static bool EvaluateBool(string expression, Dictionary<string, TaleValue> variables)
        {
            return Evaluate(expression, variables).ToBool();
        }

        public static string InterpolateText(string text, Dictionary<string, TaleValue> variables)
        {
            if (text == null || !text.Contains("{")) return text ?? "";

            try
            {
                var sb = new StringBuilder();
                int i = 0;
                InterpolateOuter(text, ref i, sb, variables);
                return sb.ToString();
            }
            catch
            {
                return text;
            }
        }

        // --- Tokenizer ---

        private static List<Token> Tokenize(string input)
        {
            var tokens = new List<Token>();
            int i = 0;
            int len = input.Length;

            while (i < len)
            {
                char c = input[i];
                if (c == ' ' || c == '\t' || c == '\r' || c == '\n') { i++; continue; }

                if (c == '(') { tokens.Add(new Token { Type = TokenType.LParen }); i++; continue; }
                if (c == ')') { tokens.Add(new Token { Type = TokenType.RParen }); i++; continue; }
                if (c == '+') { tokens.Add(new Token { Type = TokenType.Plus }); i++; continue; }
                if (c == '*') { tokens.Add(new Token { Type = TokenType.Star }); i++; continue; }
                if (c == '/') { tokens.Add(new Token { Type = TokenType.Slash }); i++; continue; }
                if (c == '%') { tokens.Add(new Token { Type = TokenType.Percent }); i++; continue; }

                if (c == '-')
                {
                    bool isUnary = tokens.Count == 0 || IsUnaryContext(tokens[tokens.Count - 1]);
                    tokens.Add(new Token { Type = TokenType.Minus, IsUnaryMinus = isUnary });
                    i++; continue;
                }

                if (c == '!' && i + 1 < len && input[i + 1] == '=')
                { tokens.Add(new Token { Type = TokenType.Neq }); i += 2; continue; }
                if (c == '!') { tokens.Add(new Token { Type = TokenType.Not }); i++; continue; }
                if (c == '=' && i + 1 < len && input[i + 1] == '=')
                { tokens.Add(new Token { Type = TokenType.Eq }); i += 2; continue; }
                if (c == '>' && i + 1 < len && input[i + 1] == '=')
                { tokens.Add(new Token { Type = TokenType.Gte }); i += 2; continue; }
                if (c == '>') { tokens.Add(new Token { Type = TokenType.Gt }); i++; continue; }
                if (c == '<' && i + 1 < len && input[i + 1] == '=')
                { tokens.Add(new Token { Type = TokenType.Lte }); i += 2; continue; }
                if (c == '<') { tokens.Add(new Token { Type = TokenType.Lt }); i++; continue; }
                if (c == '&' && i + 1 < len && input[i + 1] == '&')
                { tokens.Add(new Token { Type = TokenType.And }); i += 2; continue; }
                if (c == '|' && i + 1 < len && input[i + 1] == '|')
                { tokens.Add(new Token { Type = TokenType.Or }); i += 2; continue; }

                if (c == '"')
                {
                    i++;
                    int start = i;
                    while (i < len && input[i] != '"') i++;
                    if (i >= len) return null; // unterminated
                    string s = input.Substring(start, i - start);
                    tokens.Add(new Token { Type = TokenType.Str, Value = s });
                    i++; continue;
                }

                if (char.IsDigit(c))
                {
                    int start = i;
                    while (i < len && char.IsDigit(input[i])) i++;
                    if (i < len && input[i] == '.' && i + 1 < len && char.IsDigit(input[i + 1]))
                    {
                        i++;
                        while (i < len && char.IsDigit(input[i])) i++;
                        string s = input.Substring(start, i - start);
                        double f = double.Parse(s, CultureInfo.InvariantCulture);
                        tokens.Add(new Token { Type = TokenType.Float, Value = f });
                    }
                    else
                    {
                        string s = input.Substring(start, i - start);
                        long n = long.Parse(s);
                        tokens.Add(new Token { Type = TokenType.Int, Value = n });
                    }
                    continue;
                }

                if (char.IsLetter(c) || c == '_')
                {
                    int start = i;
                    while (i < len && (char.IsLetterOrDigit(input[i]) || input[i] == '_')) i++;
                    string word = input.Substring(start, i - start);
                    if (word == "true") tokens.Add(new Token { Type = TokenType.Bool, Value = true });
                    else if (word == "false") tokens.Add(new Token { Type = TokenType.Bool, Value = false });
                    else tokens.Add(new Token { Type = TokenType.Ident, Value = word });
                    continue;
                }

                return null; // unexpected char
            }
            return tokens;
        }

        private static bool IsUnaryContext(Token t)
        {
            return t.Type == TokenType.Plus || t.Type == TokenType.Minus
                || t.Type == TokenType.Star || t.Type == TokenType.Slash
                || t.Type == TokenType.Percent || t.Type == TokenType.Not
                || t.Type == TokenType.LParen
                || t.Type == TokenType.Eq || t.Type == TokenType.Neq
                || t.Type == TokenType.Gt || t.Type == TokenType.Lt
                || t.Type == TokenType.Gte || t.Type == TokenType.Lte
                || t.Type == TokenType.And || t.Type == TokenType.Or;
        }

        // --- Parser ---

        private class Parser
        {
            private readonly List<Token> _tokens;
            public int Pos;

            public Parser(List<Token> tokens) { _tokens = tokens; Pos = 0; }

            private Token? Peek() => Pos < _tokens.Count ? _tokens[Pos] : (Token?)null;
            private Token Advance() => _tokens[Pos++];

            public AstNode ParseExpr() => ParseOr();

            private AstNode ParseOr()
            {
                var left = ParseAnd();
                while (Peek()?.Type == TokenType.Or) { Advance(); var r = ParseAnd(); left = new BinaryNode { Op = "||", Left = left, Right = r }; }
                return left;
            }

            private AstNode ParseAnd()
            {
                var left = ParseEquality();
                while (Peek()?.Type == TokenType.And) { Advance(); var r = ParseEquality(); left = new BinaryNode { Op = "&&", Left = left, Right = r }; }
                return left;
            }

            private AstNode ParseEquality()
            {
                var left = ParseComparison();
                while (Peek()?.Type == TokenType.Eq || Peek()?.Type == TokenType.Neq)
                {
                    var t = Advance();
                    string op = t.Type == TokenType.Eq ? "==" : "!=";
                    var r = ParseComparison();
                    left = new BinaryNode { Op = op, Left = left, Right = r };
                }
                return left;
            }

            private AstNode ParseComparison()
            {
                var left = ParseAdditive();
                while (Peek()?.Type == TokenType.Gt || Peek()?.Type == TokenType.Lt
                    || Peek()?.Type == TokenType.Gte || Peek()?.Type == TokenType.Lte)
                {
                    var t = Advance();
                    string op;
                    switch (t.Type)
                    {
                        case TokenType.Gt: op = ">"; break;
                        case TokenType.Lt: op = "<"; break;
                        case TokenType.Gte: op = ">="; break;
                        case TokenType.Lte: op = "<="; break;
                        default: op = ""; break;
                    }
                    var r = ParseAdditive();
                    left = new BinaryNode { Op = op, Left = left, Right = r };
                }
                return left;
            }

            private AstNode ParseAdditive()
            {
                var left = ParseMultiplicative();
                while (Peek()?.Type == TokenType.Plus
                    || (Peek()?.Type == TokenType.Minus && !Peek().Value.IsUnaryMinus))
                {
                    var t = Advance();
                    string op = t.Type == TokenType.Plus ? "+" : "-";
                    var r = ParseMultiplicative();
                    left = new BinaryNode { Op = op, Left = left, Right = r };
                }
                return left;
            }

            private AstNode ParseMultiplicative()
            {
                var left = ParseUnary();
                while (Peek()?.Type == TokenType.Star || Peek()?.Type == TokenType.Slash
                    || Peek()?.Type == TokenType.Percent)
                {
                    var t = Advance();
                    string op;
                    switch (t.Type)
                    {
                        case TokenType.Star: op = "*"; break;
                        case TokenType.Slash: op = "/"; break;
                        case TokenType.Percent: op = "%"; break;
                        default: op = ""; break;
                    }
                    var r = ParseUnary();
                    left = new BinaryNode { Op = op, Left = left, Right = r };
                }
                return left;
            }

            private AstNode ParseUnary()
            {
                if (Peek()?.Type == TokenType.Not) { Advance(); return new NotNode { Expr = ParseUnary() }; }
                if (Peek()?.Type == TokenType.Minus && Peek().Value.IsUnaryMinus) { Advance(); return new NegNode { Expr = ParseUnary() }; }
                return ParsePrimary();
            }

            private AstNode ParsePrimary()
            {
                var tok = Peek();
                if (tok == null) return null;

                switch (tok.Value.Type)
                {
                    case TokenType.Int:
                        Advance();
                        return new LiteralNode { Value = TaleValue.FromInt((long)tok.Value.Value) };
                    case TokenType.Float:
                        Advance();
                        return new LiteralNode { Value = TaleValue.FromFloat((double)tok.Value.Value) };
                    case TokenType.Str:
                        Advance();
                        return new LiteralNode { Value = TaleValue.FromText((string)tok.Value.Value) };
                    case TokenType.Bool:
                        Advance();
                        return new LiteralNode { Value = TaleValue.FromBool((bool)tok.Value.Value) };
                    case TokenType.Ident:
                        Advance();
                        return new VariableNode { Name = (string)tok.Value.Value };
                    case TokenType.LParen:
                        Advance();
                        var expr = ParseExpr();
                        if (Peek()?.Type != TokenType.RParen) return null;
                        Advance();
                        return expr;
                    default:
                        return null;
                }
            }
        }

        // --- Evaluator ---

        private static TaleValue Eval(AstNode node, Dictionary<string, TaleValue> vars)
        {
            if (node is LiteralNode lit) return lit.Value;

            if (node is VariableNode varNode)
            {
                if (vars != null && vars.TryGetValue(varNode.Name, out var val))
                    return val;
                return TaleValue.FromInt(0); // undefined -> 0
            }

            if (node is NotNode notNode)
            {
                var v = Eval(notNode.Expr, vars);
                return TaleValue.FromBool(!v.ToBool());
            }

            if (node is NegNode negNode)
            {
                var v = Eval(negNode.Expr, vars);
                switch (v.Type)
                {
                    case TaleValue.ValueType.Int: return TaleValue.FromInt(-v.IntVal);
                    case TaleValue.ValueType.Float: return TaleValue.FromFloat(-v.FloatVal);
                    case TaleValue.ValueType.Bool: return TaleValue.FromInt(v.BoolVal ? -1 : 0);
                    default: return TaleValue.FromInt(0);
                }
            }

            if (node is BinaryNode bin)
            {
                var lv = Eval(bin.Left, vars);

                // Short-circuit
                if (bin.Op == "&&")
                {
                    if (!lv.ToBool()) return TaleValue.FromBool(false);
                    return TaleValue.FromBool(Eval(bin.Right, vars).ToBool());
                }
                if (bin.Op == "||")
                {
                    if (lv.ToBool()) return TaleValue.FromBool(true);
                    return TaleValue.FromBool(Eval(bin.Right, vars).ToBool());
                }

                var rv = Eval(bin.Right, vars);

                // String concatenation
                if (bin.Op == "+" && (lv.Type == TaleValue.ValueType.Text || rv.Type == TaleValue.ValueType.Text))
                    return TaleValue.FromText(lv.ToString() + rv.ToString());

                switch (bin.Op)
                {
                    case "+": case "-": case "*": case "/": case "%":
                        return EvalArithmetic(bin.Op, lv, rv);
                    case "==":
                        return TaleValue.FromBool(ValuesEqual(lv, rv));
                    case "!=":
                        return TaleValue.FromBool(!ValuesEqual(lv, rv));
                    case ">": case "<": case ">=": case "<=":
                        return TaleValue.FromBool(EvalComparison(bin.Op, lv, rv));
                }
            }

            return TaleValue.FromInt(0);
        }

        private static TaleValue EvalArithmetic(string op, TaleValue lv, TaleValue rv)
        {
            bool useFloat = lv.Type == TaleValue.ValueType.Float || rv.Type == TaleValue.ValueType.Float;
            if (useFloat)
            {
                double a = lv.ToDouble(), b = rv.ToDouble();
                switch (op)
                {
                    case "+": return TaleValue.FromFloat(a + b);
                    case "-": return TaleValue.FromFloat(a - b);
                    case "*": return TaleValue.FromFloat(a * b);
                    case "/": return TaleValue.FromFloat(b != 0 ? a / b : 0);
                    case "%": return TaleValue.FromFloat(b != 0 ? a % b : 0);
                }
            }
            else
            {
                long a = ToLong(lv), b = ToLong(rv);
                switch (op)
                {
                    case "+": return TaleValue.FromInt(a + b);
                    case "-": return TaleValue.FromInt(a - b);
                    case "*": return TaleValue.FromInt(a * b);
                    case "/": return TaleValue.FromInt(b != 0 ? a / b : 0);
                    case "%": return TaleValue.FromInt(b != 0 ? a % b : 0);
                }
            }
            return TaleValue.FromInt(0);
        }

        private static long ToLong(TaleValue v)
        {
            switch (v.Type)
            {
                case TaleValue.ValueType.Int: return v.IntVal;
                case TaleValue.ValueType.Float: return (long)v.FloatVal;
                case TaleValue.ValueType.Bool: return v.BoolVal ? 1 : 0;
                default: return 0;
            }
        }

        private static bool ValuesEqual(TaleValue lv, TaleValue rv)
        {
            if (lv.Type == TaleValue.ValueType.Bool && rv.Type == TaleValue.ValueType.Bool)
                return lv.BoolVal == rv.BoolVal;
            if (lv.Type == TaleValue.ValueType.Text && rv.Type == TaleValue.ValueType.Text)
                return lv.TextVal == rv.TextVal;
            if (lv.IsNumeric && rv.IsNumeric)
                return Math.Abs(lv.ToDouble() - rv.ToDouble()) < 1e-9;
            return lv.ToString() == rv.ToString();
        }

        private static bool EvalComparison(string op, TaleValue lv, TaleValue rv)
        {
            if (lv.IsNumeric && rv.IsNumeric)
            {
                double a = lv.ToDouble(), b = rv.ToDouble();
                switch (op)
                {
                    case ">": return a > b;
                    case "<": return a < b;
                    case ">=": return a >= b;
                    case "<=": return a <= b;
                }
            }
            int cmp = string.Compare(lv.ToString(), rv.ToString(), StringComparison.Ordinal);
            switch (op)
            {
                case ">": return cmp > 0;
                case "<": return cmp < 0;
                case ">=": return cmp >= 0;
                case "<=": return cmp <= 0;
            }
            return false;
        }

        // --- Text Interpolation ---

        private static void InterpolateOuter(string text, ref int i, StringBuilder sb,
            Dictionary<string, TaleValue> vars)
        {
            int len = text.Length;
            while (i < len)
            {
                if (text[i] == '{')
                {
                    i++; SkipWs(text, ref i);

                    // Check for top-level {/if} or {else} — malformed
                    if (i < len && StartsAt(text, i, "/if")) throw new Exception();
                    if (i < len && StartsAt(text, i, "else"))
                    {
                        int after = i + 4; SkipWs(text, ref after);
                        if (after < len && text[after] == '}') throw new Exception();
                    }

                    if (StartsAt(text, i, "if ") || StartsAt(text, i, "if\t"))
                    {
                        i += 2; SkipWs(text, ref i);
                        string condStr = ReadUntilClose(text, ref i);
                        bool condVal = Evaluate(condStr, vars).ToBool();

                        var thenSb = new StringBuilder();
                        string stop = InterpolateBranch(text, ref i, thenSb, vars);

                        var elseSb = new StringBuilder();
                        if (stop == "else")
                            InterpolateBranch(text, ref i, elseSb, vars);

                        sb.Append(condVal ? thenSb.ToString() : elseSb.ToString());
                    }
                    else
                    {
                        string exprStr = ReadUntilClose(text, ref i);
                        try
                        {
                            var val = Evaluate(exprStr.Trim(), vars);
                            sb.Append(val.ToString());
                        }
                        catch { sb.Append("???"); }
                    }
                }
                else
                {
                    sb.Append(text[i]);
                    i++;
                }
            }
        }

        /// <summary>Parse a branch (then or else) until {else} or {/if}. Returns "else" or "endif".</summary>
        private static string InterpolateBranch(string text, ref int i, StringBuilder sb,
            Dictionary<string, TaleValue> vars)
        {
            int len = text.Length;
            while (i < len)
            {
                if (text[i] == '{')
                {
                    int save = i;
                    i++; SkipWs(text, ref i);

                    if (StartsAt(text, i, "/if"))
                    {
                        i += 3; SkipWs(text, ref i);
                        if (i < len && text[i] == '}') i++;
                        return "endif";
                    }

                    if (StartsAt(text, i, "else"))
                    {
                        int after = i + 4; SkipWs(text, ref after);
                        if (after < len && text[after] == '}')
                        {
                            i = after + 1;
                            return "else";
                        }
                    }

                    if (StartsAt(text, i, "if ") || StartsAt(text, i, "if\t"))
                    {
                        i += 2; SkipWs(text, ref i);
                        string condStr = ReadUntilClose(text, ref i);
                        bool condVal = Evaluate(condStr, vars).ToBool();

                        var thenSb = new StringBuilder();
                        string stop = InterpolateBranch(text, ref i, thenSb, vars);

                        var elseSb = new StringBuilder();
                        if (stop == "else")
                            InterpolateBranch(text, ref i, elseSb, vars);

                        sb.Append(condVal ? thenSb.ToString() : elseSb.ToString());
                        continue;
                    }

                    // Regular expression
                    string exprStr = ReadUntilClose(text, ref i);
                    try
                    {
                        var val = Evaluate(exprStr.Trim(), vars);
                        sb.Append(val.ToString());
                    }
                    catch { sb.Append("???"); }
                }
                else
                {
                    sb.Append(text[i]);
                    i++;
                }
            }
            return "eof";
        }

        private static void SkipWs(string text, ref int i)
        {
            while (i < text.Length && (text[i] == ' ' || text[i] == '\t')) i++;
        }

        private static bool StartsAt(string text, int i, string prefix)
        {
            if (i + prefix.Length > text.Length) return false;
            for (int j = 0; j < prefix.Length; j++)
                if (text[i + j] != prefix[j]) return false;
            return true;
        }

        private static string ReadUntilClose(string text, ref int i)
        {
            int start = i;
            int depth = 1;
            while (i < text.Length)
            {
                if (text[i] == '{') depth++;
                else if (text[i] == '}')
                {
                    depth--;
                    if (depth == 0)
                    {
                        string content = text.Substring(start, i - start).Trim();
                        i++;
                        return content;
                    }
                }
                i++;
            }
            throw new Exception("Unterminated '{'");
        }
    }
}
