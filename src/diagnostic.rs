//! CLI diagnostic rendering.

use std::path::Path;

use crate::error::ForgeError;

/// Renders a compiler-style diagnostic when source location is available.
pub fn render_file_error(path: &Path, source: Option<&str>, error: &ForgeError) -> String {
    let (headline, details) = split_message(&error.message);
    let mut rendered = format!("error: {headline}");

    match (error.line, error.column) {
        (Some(line), Some(column)) => {
            rendered.push_str(&format!("\n  --> {}:{line}:{column}", path.display()));
            if let Some(source) = source {
                if let Some(source_line) = source.lines().nth(line.saturating_sub(1)) {
                    rendered.push_str("\n   |");
                    let line_number = line.to_string();
                    let expanded_line = expand_tabs(source_line);
                    let marker_padding = caret_padding(source_line, column);
                    rendered.push_str(&format!("\n{line_number:>3} | {expanded_line}"));
                    rendered.push_str(&format!("\n   | {marker_padding}^"));
                }
            }
        }
        _ => rendered.push_str(&format!("\n  --> {}", path.display())),
    }

    if !details.is_empty() {
        rendered.push_str("\n   |");
        for detail in details {
            rendered.push_str(&format!("\n   = {detail}"));
        }
    }

    if let Some(help) = diagnostic_help(headline) {
        rendered.push_str("\n   |");
        rendered.push_str(&format!("\n   = help: {help}"));
    }

    rendered
}

fn split_message(message: &str) -> (&str, Vec<&str>) {
    let mut lines = message.lines();
    let headline = lines.next().unwrap_or("Unknown Forge error.");
    let details = lines
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .collect::<Vec<_>>();
    (headline, details)
}

fn expand_tabs(source_line: &str) -> String {
    source_line.replace('\t', "    ")
}

fn caret_padding(source_line: &str, column: usize) -> String {
    let target = column.saturating_sub(1);
    let mut width = 0;

    for (index, ch) in source_line.chars().enumerate() {
        if index >= target {
            break;
        }
        width += if ch == '\t' { 4 } else { 1 };
    }

    " ".repeat(width)
}

fn diagnostic_help(headline: &str) -> Option<&'static str> {
    if headline.contains("Cannot add incompatible quantities")
        || headline.contains("Cannot subtract incompatible quantities")
    {
        Some("addition and subtraction require matching dimensions")
    } else if headline.contains("Cannot convert expression to the requested unit")
        || headline.contains("Cannot convert incompatible quantities")
    {
        Some("convert only between units with the same physical dimension")
    } else if headline.contains("Unknown unit") {
        Some("run `forge units` to see supported built-in units")
    } else if headline.contains("Unknown variable") {
        Some("assign the variable before using it")
    } else if headline.contains("Invalid exponent usage") {
        Some("exponents must be dimensionless integer literals")
    } else if headline.contains("Invalid syntax") {
        Some("check the statement structure and unit expression syntax")
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use crate::diagnostic::render_file_error;
    use crate::error::ForgeError;

    #[test]
    fn renders_source_line_and_caret() {
        let source = "pressure = 2 bar\nlength = 3 m\nresult = pressure + length\n";
        let error = ForgeError::with_span(
            "Cannot add incompatible quantities.\nLeft operand dimension: [L^-1 M T^-2]\nRight operand dimension: [L]",
            3,
            1,
        );

        let rendered = render_file_error(Path::new("main.forge"), Some(source), &error);
        assert!(rendered.contains("error: Cannot add incompatible quantities."));
        assert!(rendered.contains("--> main.forge:3:1"));
        assert!(rendered.contains("3 | result = pressure + length"));
        assert!(rendered.contains("| ^"));
        assert!(rendered.contains("= Left operand dimension: [L^-1 M T^-2]"));
        assert!(rendered.contains("= help: addition and subtraction require matching dimensions"));
    }

    #[test]
    fn falls_back_when_source_line_is_missing() {
        let error = ForgeError::with_span("Invalid syntax.", 20, 4);
        let rendered = render_file_error(Path::new("main.forge"), Some("x = 1\n"), &error);
        assert!(rendered.contains("--> main.forge:20:4"));
        assert!(!rendered.contains("20 |"));
    }
}
