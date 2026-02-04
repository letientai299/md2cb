//! GitHub Flavored Markdown to HTML converter.

use fancy_regex::Regex as FancyRegex;
use regex::Regex;
use std::collections::HashMap;

/// Converts GitHub Flavored Markdown to HTML.
pub fn convert(markdown: &str) -> String {
    let mut html = markdown.to_string();

    // Extract reference links first
    let refs = extract_reference_links(&mut html);

    // Process blocks first
    html = convert_code_blocks(&html);
    html = convert_blockquotes(&html);
    html = convert_headers(&html);
    html = convert_horizontal_rules(&html);
    html = convert_tables(&html);
    html = convert_lists(&html);
    html = convert_paragraphs(&html);

    // Then inline elements
    html = convert_inline_elements(&html, &refs);

    html
}

/// Extracts reference-style link definitions from markdown.
/// Removes the definitions from the text and returns a map of reference keys to URLs.
fn extract_reference_links(text: &mut String) -> HashMap<String, String> {
    let mut refs = HashMap::new();
    let re = Regex::new(r"(?m)^\[([^\]]+)\]:\s*(.+)$").unwrap();

    // Collect matches first to avoid borrow issues
    let matches: Vec<_> = re
        .captures_iter(text)
        .map(|cap| {
            let full = cap.get(0).unwrap();
            let key = cap.get(1).unwrap().as_str().to_lowercase();
            let url = cap.get(2).unwrap().as_str().trim().to_string();
            (full.start(), full.end(), key, url)
        })
        .collect();

    // Remove matches in reverse order to preserve indices
    for (start, end, key, url) in matches.into_iter().rev() {
        refs.insert(key, url);
        text.replace_range(start..end, "");
    }

    refs
}

/// Converts fenced code blocks (```lang ... ```) to HTML.
fn convert_code_blocks(text: &str) -> String {
    let re = Regex::new(r"```(\w*)\n([\s\S]*?)```").unwrap();
    re.replace_all(text, r#"<pre><code class="language-$1">$2</code></pre>"#)
        .to_string()
}

/// Converts blockquote lines (> text) to HTML.
fn convert_blockquotes(text: &str) -> String {
    let lines: Vec<&str> = text.lines().collect();
    let mut result: Vec<String> = Vec::new();
    let mut in_quote = false;
    let mut quote_content: Vec<String> = Vec::new();

    for line in lines {
        if let Some(stripped) = line.strip_prefix('>') {
            if !in_quote {
                in_quote = true;
                quote_content.clear();
            }
            let content = stripped.trim();
            quote_content.push(content.to_string());
        } else {
            if in_quote {
                result.push(format!(
                    "<blockquote><p>{}</p></blockquote>",
                    quote_content.join(" ")
                ));
                in_quote = false;
            }
            result.push(line.to_string());
        }
    }

    if in_quote {
        result.push(format!(
            "<blockquote><p>{}</p></blockquote>",
            quote_content.join(" ")
        ));
    }

    result.join("\n")
}

/// Converts ATX-style headers (# H1, ## H2, etc.) to HTML.
fn convert_headers(text: &str) -> String {
    let mut result = text.to_string();

    // Process H6 first to H1 to avoid partial matches
    for level in (1..=6).rev() {
        let prefix = "#".repeat(level);
        let pattern = format!(r"(?m)^{} (.+)$", regex::escape(&prefix));
        let re = Regex::new(&pattern).unwrap();
        result = re
            .replace_all(&result, format!("<h{level}>$1</h{level}>"))
            .to_string();
    }

    result
}

/// Converts horizontal rules (---, ***, ___) to HTML.
fn convert_horizontal_rules(text: &str) -> String {
    let re = Regex::new(r"(?m)^(---+|\*\*\*+|___+)$").unwrap();
    re.replace_all(text, "<hr>").to_string()
}

/// Converts GFM tables to HTML with alignment support.
fn convert_tables(text: &str) -> String {
    let lines: Vec<&str> = text.lines().collect();
    let mut result: Vec<String> = Vec::new();
    let mut index = 0;

    while index < lines.len() {
        let line = lines[index];

        // Check if this is a table header row (has |)
        if line.contains('|') && index + 1 < lines.len() {
            let next_line = lines[index + 1];
            // Check for separator row (|---|---|)
            if next_line.contains('|') && next_line.contains('-') {
                let mut table_html = String::from("<table>\n<thead>\n<tr>\n");

                // Parse alignments from separator
                let alignments = parse_table_alignments(next_line);

                // Header row
                let header_cells = parse_table_row(line);
                for (idx, cell) in header_cells.iter().enumerate() {
                    let align = alignments.get(idx).map(|s| s.as_str()).unwrap_or("");
                    let style = if align.is_empty() {
                        String::new()
                    } else {
                        format!(r#" style="text-align:{align}""#)
                    };
                    table_html.push_str(&format!("<th{style}>{cell}</th>\n"));
                }
                table_html.push_str("</tr>\n</thead>\n<tbody>\n");

                // Skip header and separator
                index += 2;

                // Body rows
                while index < lines.len() && lines[index].contains('|') {
                    table_html.push_str("<tr>\n");
                    let cells = parse_table_row(lines[index]);
                    for (idx, cell) in cells.iter().enumerate() {
                        let align = alignments.get(idx).map(|s| s.as_str()).unwrap_or("");
                        let style = if align.is_empty() {
                            String::new()
                        } else {
                            format!(r#" style="text-align:{align}""#)
                        };
                        table_html.push_str(&format!("<td{style}>{cell}</td>\n"));
                    }
                    table_html.push_str("</tr>\n");
                    index += 1;
                }

                table_html.push_str("</tbody>\n</table>");
                result.push(table_html);
                continue;
            }
        }

        result.push(line.to_string());
        index += 1;
    }

    result.join("\n")
}

/// Parses a table row into cell contents.
fn parse_table_row(row: &str) -> Vec<String> {
    row.split('|')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect()
}

/// Parses alignment markers from a table separator row.
fn parse_table_alignments(separator: &str) -> Vec<String> {
    separator
        .split('|')
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .map(|cell| {
            let left = cell.starts_with(':');
            let right = cell.ends_with(':');
            if left && right {
                "center".to_string()
            } else if right {
                "right".to_string()
            } else {
                String::new()
            }
        })
        .collect()
}

/// Converts markdown lists (ordered, unordered, task) to HTML.
fn convert_lists(text: &str) -> String {
    let lines: Vec<&str> = text.lines().collect();
    let mut result: Vec<String> = Vec::new();
    let mut list_stack: Vec<(String, usize)> = Vec::new(); // (type, indent)

    // Regex patterns for list items
    let task_re = Regex::new(r"^[-*+] \[([ xX])\] (.*)$").unwrap();
    let unordered_re = Regex::new(r"^[-*+] (.*)$").unwrap();
    let ordered_re = Regex::new(r"^\d+\. (.*)$").unwrap();

    for line in lines {
        let trimmed = line.trim();
        let indent = line.len() - line.trim_start().len();

        if let Some((list_type, content)) =
            parse_list_item(trimmed, &task_re, &unordered_re, &ordered_re)
        {
            handle_list_item(&mut result, &mut list_stack, indent, &list_type);
            result.push(format!("<li>{content}</li>"));
        } else {
            // Close all open lists
            while let Some((list_type, _)) = list_stack.pop() {
                result.push(format!("</{list_type}>"));
            }
            result.push(line.to_string());
        }
    }

    // Close remaining lists
    while let Some((list_type, _)) = list_stack.pop() {
        result.push(format!("</{list_type}>"));
    }

    result.join("\n")
}

/// Parses a list item and returns its type and content.
fn parse_list_item(
    line: &str,
    task_re: &Regex,
    unordered_re: &Regex,
    ordered_re: &Regex,
) -> Option<(String, String)> {
    // Task list item: - [ ] or - [x]
    if let Some(caps) = task_re.captures(line) {
        let checkbox = caps.get(1).unwrap().as_str();
        let content = caps.get(2).unwrap().as_str();
        let symbol = if checkbox == "x" || checkbox == "X" {
            "\u{2611}" // ☑
        } else {
            "\u{2610}" // ☐
        };
        return Some(("ul".to_string(), format!("{symbol} {content}")));
    }

    // Unordered list item: - or * or +
    if let Some(caps) = unordered_re.captures(line) {
        let content = caps.get(1).unwrap().as_str();
        return Some(("ul".to_string(), content.to_string()));
    }

    // Ordered list item: 1.
    if let Some(caps) = ordered_re.captures(line) {
        let content = caps.get(1).unwrap().as_str();
        return Some(("ol".to_string(), content.to_string()));
    }

    None
}

/// Handles list item nesting based on indentation.
fn handle_list_item(
    result: &mut Vec<String>,
    stack: &mut Vec<(String, usize)>,
    indent: usize,
    list_type: &str,
) {
    // Close lists that are deeper than current indent
    while let Some((_, last_indent)) = stack.last() {
        if *last_indent > indent {
            let (closed_type, _) = stack.pop().unwrap();
            result.push(format!("</{closed_type}>"));
        } else {
            break;
        }
    }

    // If we're at a deeper indent, start a new nested list
    if let Some((last_type, last_indent)) = stack.last() {
        if indent > *last_indent {
            result.push(format!("<{list_type}>"));
            stack.push((list_type.to_string(), indent));
        } else if last_type != list_type {
            // Same indent but different list type
            let (closed_type, _) = stack.pop().unwrap();
            result.push(format!("</{closed_type}>"));
            result.push(format!("<{list_type}>"));
            stack.push((list_type.to_string(), indent));
        }
    } else {
        // No list open yet
        result.push(format!("<{list_type}>"));
        stack.push((list_type.to_string(), indent));
    }
}

/// Wraps non-HTML text lines in paragraph tags.
fn convert_paragraphs(text: &str) -> String {
    let lines: Vec<&str> = text.lines().collect();
    let mut result: Vec<String> = Vec::new();
    let mut paragraph: Vec<String> = Vec::new();

    let flush_paragraph = |result: &mut Vec<String>, paragraph: &mut Vec<String>| {
        if !paragraph.is_empty() {
            let content = paragraph.join(" ");
            if !content.trim().is_empty() {
                result.push(format!("<p>{content}</p>"));
            }
            paragraph.clear();
        }
    };

    for line in lines {
        let trimmed = line.trim();

        // Skip if it's already HTML or empty
        if trimmed.starts_with('<') || trimmed.is_empty() {
            flush_paragraph(&mut result, &mut paragraph);
            result.push(line.to_string());
        } else {
            paragraph.push(trimmed.to_string());
        }
    }

    flush_paragraph(&mut result, &mut paragraph);
    result.join("\n")
}

/// Converts inline markdown elements to HTML.
fn convert_inline_elements(text: &str, refs: &HashMap<String, String>) -> String {
    let mut result = text.to_string();

    // Images ![alt](src "title")
    let img_re = Regex::new(r#"!\[([^\]]*)\]\(([^\s\)]+)(?:\s+"([^"]+)")?\)"#).unwrap();
    result = img_re
        .replace_all(&result, r#"<img alt="$1" src="$2" title="$3">"#)
        .to_string();

    // Links [text](url)
    let link_re = Regex::new(r"\[([^\]]+)\]\(([^\)]+)\)").unwrap();
    result = link_re
        .replace_all(&result, r#"<a href="$2">$1</a>"#)
        .to_string();

    // Reference links [text][ref]
    result = convert_reference_links(&result, refs);

    // Auto links (URLs) - use fancy_regex for look-behind
    let auto_link_re = FancyRegex::new(r#"(?<!["=])\b(https?://[^\s<>]+)"#).unwrap();
    result = fancy_replace_all(&auto_link_re, &result, |caps| {
        let url = caps.get(1).map(|m| m.as_str()).unwrap_or("");
        format!(r#"<a href="{url}">{url}</a>"#)
    });

    // Bold **text** or __text__
    let bold_asterisk_re = Regex::new(r"\*\*([^\*]+)\*\*").unwrap();
    result = bold_asterisk_re
        .replace_all(&result, "<strong>$1</strong>")
        .to_string();

    let bold_underscore_re = Regex::new(r"__([^_]+)__").unwrap();
    result = bold_underscore_re
        .replace_all(&result, "<strong>$1</strong>")
        .to_string();

    // Italic *text* or _text_ - use fancy_regex for look-around
    let italic_asterisk_re = FancyRegex::new(r"(?<![*\w])\*([^\*]+)\*(?![*\w])").unwrap();
    result = fancy_replace_all(&italic_asterisk_re, &result, |caps| {
        let content = caps.get(1).map(|m| m.as_str()).unwrap_or("");
        format!("<em>{content}</em>")
    });

    let italic_underscore_re = FancyRegex::new(r"(?<![_\w])_([^_]+)_(?![_\w])").unwrap();
    result = fancy_replace_all(&italic_underscore_re, &result, |caps| {
        let content = caps.get(1).map(|m| m.as_str()).unwrap_or("");
        format!("<em>{content}</em>")
    });

    // Strikethrough ~~text~~
    let strikethrough_re = Regex::new(r"~~([^~]+)~~").unwrap();
    result = strikethrough_re
        .replace_all(&result, "<del>$1</del>")
        .to_string();

    // Inline code `code`
    let inline_code_re = Regex::new(r"`([^`]+)`").unwrap();
    result = inline_code_re
        .replace_all(&result, "<code>$1</code>")
        .to_string();

    result
}

/// Helper function to replace all matches using fancy_regex.
fn fancy_replace_all<F>(re: &FancyRegex, text: &str, replacer: F) -> String
where
    F: Fn(&fancy_regex::Captures) -> String,
{
    let mut result = String::new();
    let mut last_end = 0;

    for cap in re.captures_iter(text).flatten() {
        let m = cap.get(0).unwrap();
        result.push_str(&text[last_end..m.start()]);
        result.push_str(&replacer(&cap));
        last_end = m.end();
    }

    result.push_str(&text[last_end..]);
    result
}

/// Converts reference-style links [text][ref] to HTML.
fn convert_reference_links(text: &str, refs: &HashMap<String, String>) -> String {
    let re = Regex::new(r"\[([^\]]+)\]\[([^\]]+)\]").unwrap();
    let mut result = text.to_string();

    // Collect all matches first
    let matches: Vec<_> = re
        .captures_iter(text)
        .filter_map(|cap| {
            let full = cap.get(0)?;
            let link_text = cap.get(1)?.as_str();
            let ref_key = cap.get(2)?.as_str().to_lowercase();
            refs.get(&ref_key).map(|url| {
                (
                    full.start(),
                    full.end(),
                    format!(r#"<a href="{url}">{link_text}</a>"#),
                )
            })
        })
        .collect();

    // Replace in reverse order
    for (start, end, replacement) in matches.into_iter().rev() {
        result.replace_range(start..end, &replacement);
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_headers() {
        assert!(convert("# Hello").contains("<h1>Hello</h1>"));
        assert!(convert("## World").contains("<h2>World</h2>"));
        assert!(convert("### Test").contains("<h3>Test</h3>"));
    }

    #[test]
    fn test_bold() {
        assert!(convert("**bold**").contains("<strong>bold</strong>"));
        assert!(convert("__bold__").contains("<strong>bold</strong>"));
    }

    #[test]
    fn test_italic() {
        assert!(convert("*italic*").contains("<em>italic</em>"));
        assert!(convert("_italic_").contains("<em>italic</em>"));
    }

    #[test]
    fn test_links() {
        assert!(convert("[text](url)").contains(r#"<a href="url">text</a>"#));
    }

    #[test]
    fn test_code_blocks() {
        let result = convert("```rust\nfn main() {}\n```");
        assert!(result.contains(r#"<pre><code class="language-rust">"#));
        assert!(result.contains("fn main() {}"));
    }

    #[test]
    fn test_inline_code() {
        assert!(convert("`code`").contains("<code>code</code>"));
    }

    #[test]
    fn test_horizontal_rule() {
        assert!(convert("---").contains("<hr>"));
        assert!(convert("***").contains("<hr>"));
        assert!(convert("___").contains("<hr>"));
    }

    #[test]
    fn test_blockquote() {
        assert!(convert("> quote").contains("<blockquote><p>quote</p></blockquote>"));
    }

    #[test]
    fn test_unordered_list() {
        let result = convert("- item1\n- item2");
        assert!(result.contains("<ul>"));
        assert!(result.contains("<li>item1</li>"));
        assert!(result.contains("<li>item2</li>"));
        assert!(result.contains("</ul>"));
    }

    #[test]
    fn test_ordered_list() {
        let result = convert("1. first\n2. second");
        assert!(result.contains("<ol>"));
        assert!(result.contains("<li>first</li>"));
        assert!(result.contains("<li>second</li>"));
        assert!(result.contains("</ol>"));
    }

    #[test]
    fn test_task_list() {
        let result = convert("- [ ] todo\n- [x] done");
        assert!(result.contains("\u{2610}")); // ☐
        assert!(result.contains("\u{2611}")); // ☑
    }

    #[test]
    fn test_table() {
        let result = convert("| A | B |\n|---|---|\n| 1 | 2 |");
        assert!(result.contains("<table>"));
        assert!(result.contains("<th>A</th>"));
        assert!(result.contains("<td>1</td>"));
    }

    #[test]
    fn test_table_alignment() {
        let result = convert("| L | C | R |\n|:--|:--:|--:|\n| a | b | c |");
        assert!(result.contains(r#"style="text-align:center""#));
        assert!(result.contains(r#"style="text-align:right""#));
    }

    #[test]
    fn test_strikethrough() {
        assert!(convert("~~deleted~~").contains("<del>deleted</del>"));
    }

    #[test]
    fn test_images() {
        assert!(convert("![alt](src)").contains(r#"<img alt="alt" src="src""#));
    }

    #[test]
    fn test_reference_links() {
        let md = "[text][ref]\n\n[ref]: https://example.com";
        let result = convert(md);
        assert!(result.contains(r#"<a href="https://example.com">text</a>"#));
    }
}
