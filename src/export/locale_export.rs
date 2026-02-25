use crate::model::graph::DialogueGraph;
use crate::model::locale::{collect_translatable_strings, LocaleSettings};

/// Export all translatable strings as CSV for translator workflows.
/// Format: key,type,<default_locale>,<extra_locale_1>,<extra_locale_2>,...
pub fn export_locale_csv(graph: &DialogueGraph) -> String {
    let locale = &graph.locale;
    let strings = collect_translatable_strings(graph);

    let mut out = String::new();

    // Header row
    out.push_str("key,type,");
    out.push_str(&csv_field(&locale.default_locale));
    for loc in &locale.extra_locales {
        out.push(',');
        out.push_str(&csv_field(loc));
    }
    out.push('\n');

    // Data rows
    for ts in &strings {
        out.push_str(&csv_field(&ts.key));
        out.push(',');
        out.push_str(&csv_field(ts.string_type.label()));
        out.push(',');
        out.push_str(&csv_field(&ts.default_text));

        for loc in &locale.extra_locales {
            out.push(',');
            let translated = locale
                .get_translation(&ts.key, loc)
                .unwrap_or("");
            out.push_str(&csv_field(translated));
        }
        out.push('\n');
    }

    out
}

/// Import translations from CSV into the locale settings.
/// Returns the number of translation cells updated.
pub fn import_locale_csv(csv: &str, locale: &mut LocaleSettings) -> Result<usize, String> {
    let rows = parse_csv_rows(csv);
    if rows.is_empty() {
        return Err("CSV is empty".to_string());
    }

    let header = &rows[0];
    if header.len() < 3 {
        return Err("CSV must have at least key, type, and default locale columns".to_string());
    }

    // Columns 2.. are locale codes (column 2 = default, 3+ = extra)
    let locale_columns: Vec<&str> = header[2..].iter().map(|s| s.as_str()).collect();

    let mut count = 0;
    for row in rows.iter().skip(1) {
        if row.is_empty() || row[0].is_empty() {
            continue;
        }
        let key = &row[0];

        // Skip the default locale column (index 2 / locale_columns[0]),
        // only import extra locales
        for (col_idx, loc) in locale_columns.iter().enumerate().skip(1) {
            let text = row.get(col_idx + 2).map(|s| s.as_str()).unwrap_or("");
            // Add locale if not already present
            if !locale.extra_locales.contains(&loc.to_string())
                && *loc != locale.default_locale
            {
                locale.add_locale(loc.to_string());
            }
            locale.set_translation(key.to_string(), loc.to_string(), text.to_string());
            count += 1;
        }
    }
    Ok(count)
}

/// Quote a CSV field if it contains comma, quote, or newline.
fn csv_field(s: &str) -> String {
    if s.contains(',') || s.contains('"') || s.contains('\n') || s.contains('\r') {
        let escaped = s.replace('"', "\"\"");
        format!("\"{escaped}\"")
    } else {
        s.to_string()
    }
}

/// Parse CSV rows, handling quoted fields with commas/newlines.
fn parse_csv_rows(input: &str) -> Vec<Vec<String>> {
    let mut rows: Vec<Vec<String>> = Vec::new();
    let mut current_row: Vec<String> = Vec::new();
    let mut current_field = String::new();
    let mut in_quotes = false;
    let mut chars = input.chars().peekable();

    while let Some(ch) = chars.next() {
        if in_quotes {
            if ch == '"' {
                if chars.peek() == Some(&'"') {
                    chars.next();
                    current_field.push('"');
                } else {
                    in_quotes = false;
                }
            } else {
                current_field.push(ch);
            }
        } else {
            match ch {
                '"' => in_quotes = true,
                ',' => {
                    current_row.push(std::mem::take(&mut current_field));
                }
                '\n' => {
                    current_row.push(std::mem::take(&mut current_field));
                    rows.push(std::mem::take(&mut current_row));
                }
                '\r' => {
                    if chars.peek() == Some(&'\n') {
                        chars.next();
                    }
                    current_row.push(std::mem::take(&mut current_field));
                    rows.push(std::mem::take(&mut current_row));
                }
                _ => current_field.push(ch),
            }
        }
    }

    // Final field/row
    if !current_field.is_empty() || !current_row.is_empty() {
        current_row.push(current_field);
        rows.push(current_row);
    }

    rows
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::node::{Node, NodeType};

    fn make_test_graph() -> DialogueGraph {
        let mut graph = DialogueGraph::new();
        let mut dlg = Node::new_dialogue([0.0, 100.0]);
        if let NodeType::Dialogue(ref mut d) = dlg.node_type {
            d.text = "Hello, traveler!".to_string();
        }
        let mut choice = Node::new_choice([0.0, 200.0]);
        if let NodeType::Choice(ref mut c) = choice.node_type {
            c.prompt = "What will you do?".to_string();
        }
        graph.add_node(dlg);
        graph.add_node(choice);
        graph.locale.add_locale("fr".to_string());
        graph
    }

    #[test]
    fn export_csv_header_and_rows() {
        let graph = make_test_graph();
        let csv = export_locale_csv(&graph);
        let lines: Vec<&str> = csv.lines().collect();
        assert!(lines[0].starts_with("key,type,en,fr"));
        // 1 dialogue + 1 prompt + 2 options = 4 data rows
        assert_eq!(lines.len(), 5);
    }

    #[test]
    fn csv_roundtrip() {
        let mut graph = make_test_graph();
        let csv = export_locale_csv(&graph);
        let count = import_locale_csv(&csv, &mut graph.locale).unwrap();
        // 4 strings * 1 extra locale = 4 cells
        assert_eq!(count, 4);
    }

    #[test]
    fn import_adds_translations() {
        let mut locale = LocaleSettings::default();
        let csv = "key,type,en,fr\ndlg_abc,dialogue,Hello,Bonjour\n";
        let count = import_locale_csv(csv, &mut locale).unwrap();
        assert_eq!(count, 1);
        assert_eq!(locale.get_translation("dlg_abc", "fr"), Some("Bonjour"));
    }

    #[test]
    fn import_empty_csv_errors() {
        let mut locale = LocaleSettings::default();
        assert!(import_locale_csv("", &mut locale).is_err());
    }

    #[test]
    fn csv_field_quoting() {
        assert_eq!(csv_field("hello"), "hello");
        assert_eq!(csv_field("hello, world"), "\"hello, world\"");
        assert_eq!(csv_field("say \"hi\""), "\"say \"\"hi\"\"\"");
        assert_eq!(csv_field("line1\nline2"), "\"line1\nline2\"");
    }

    #[test]
    fn parse_csv_with_quotes() {
        let input = "a,\"b,c\",d\n\"e\"\"f\",g,h\n";
        let rows = parse_csv_rows(input);
        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0], vec!["a", "b,c", "d"]);
        assert_eq!(rows[1], vec!["e\"f", "g", "h"]);
    }

    #[test]
    fn import_adds_new_locale() {
        let mut locale = LocaleSettings::default();
        let csv = "key,type,en,ja\ndlg_abc,dialogue,Hello,こんにちは\n";
        import_locale_csv(csv, &mut locale).unwrap();
        assert!(locale.extra_locales.contains(&"ja".to_string()));
    }
}
