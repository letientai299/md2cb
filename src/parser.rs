//! GitHub Flavored Markdown to HTML converter using comrak.

use comrak::{markdown_to_html, Options};
use regex::Regex;
use std::process::{Command, Stdio};
use std::io::Write;

/// Converts GitHub Flavored Markdown to HTML.
pub fn convert(markdown: &str) -> String {
    // Convert markdown to HTML using comrak with math support
    let options = build_options();
    let html = markdown_to_html(markdown, &options);

    // Post-process: convert checkboxes to Unicode for compatibility
    let html = convert_checkboxes_to_unicode(&html);

    // Post-process: convert LaTeX in math spans to SVG using MathJax
    convert_math_to_svg(&html)
}

/// Converts HTML checkbox inputs to Unicode square symbols for better compatibility.
/// - Checked: ✅ (U+2705 WHITE HEAVY CHECK MARK)
/// - Unchecked: ⬜ (U+2B1C WHITE LARGE SQUARE)
fn convert_checkboxes_to_unicode(html: &str) -> String {
    // Match checked checkbox (has "checked" attribute)
    let checked_re = Regex::new(r#"<input[^>]*type="checkbox"[^>]*checked[^>]*/?\s*>"#).unwrap();
    let result = checked_re.replace_all(html, "✅ ");

    // Match unchecked checkbox (no "checked" attribute)
    let unchecked_re = Regex::new(r#"<input[^>]*type="checkbox"[^>]*/?\s*>"#).unwrap();
    unchecked_re.replace_all(&result, "⬜ ").to_string()
}

/// Builds comrak options with GFM extensions enabled.
fn build_options() -> Options {
    let mut options = Options::default();

    // Enable GFM extensions
    options.extension.strikethrough = true;
    options.extension.table = true;
    options.extension.autolink = true;
    options.extension.tasklist = true;
    options.extension.superscript = true;
    options.extension.footnotes = true;
    options.extension.description_lists = true;

    // Enable math with dollar syntax ($...$ and $$...$$)
    options.extension.math_dollars = true;
    options.extension.math_code = true;

    // Render options
    options.render.unsafe_ = true; // Allow raw HTML passthrough

    options
}

/// Decodes HTML entities in LaTeX content.
/// Comrak HTML-encodes content, but MathJax needs raw LaTeX.
fn decode_html_entities(s: &str) -> String {
    s.replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&#39;", "'")
}

/// Renders LaTeX to SVG using MathJax via Node.js script.
fn latex_to_svg(latex: &str, display: bool) -> Result<String, String> {
    // Find the script relative to the executable or in common locations
    let script_paths = [
        "scripts/math-to-svg.js",
        "./scripts/math-to-svg.js",
        concat!(env!("CARGO_MANIFEST_DIR"), "/scripts/math-to-svg.js"),
    ];

    let script_path = script_paths
        .iter()
        .find(|p| std::path::Path::new(p).exists())
        .ok_or_else(|| "math-to-svg.js script not found".to_string())?;

    let mut cmd = Command::new("node");
    cmd.arg(script_path);
    if display {
        cmd.arg("--display");
    }
    cmd.stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    let mut child = cmd.spawn().map_err(|e| format!("Failed to spawn node: {}", e))?;

    if let Some(stdin) = child.stdin.as_mut() {
        stdin
            .write_all(latex.as_bytes())
            .map_err(|e| format!("Failed to write to stdin: {}", e))?;
    }

    let output = child
        .wait_with_output()
        .map_err(|e| format!("Failed to wait for node: {}", e))?;

    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    } else {
        Err(String::from_utf8_lossy(&output.stderr).to_string())
    }
}

/// Converts LaTeX content in comrak's math spans to SVG using MathJax.
///
/// Comrak outputs math as:
/// - Inline: `<span data-math-style="inline">latex</span>`
/// - Display: `<span data-math-style="display">latex</span>`
///
/// This function converts the LaTeX content to inline SVG.
fn convert_math_to_svg(html: &str) -> String {
    let mut result = html.to_string();

    // Match display math spans
    let display_re =
        Regex::new(r#"<span data-math-style="display">([^<]*)</span>"#).unwrap();
    result = display_re
        .replace_all(&result, |caps: &regex::Captures| {
            let latex_raw = caps.get(1).map(|m| m.as_str()).unwrap_or("");
            let latex = decode_html_entities(latex_raw);
            match latex_to_svg(&latex, true) {
                Ok(svg) => format!(r#"<div class="math math-display">{svg}</div>"#),
                Err(_) => format!(
                    r#"<div class="math math-display math-error">$${}$$</div>"#,
                    latex
                ),
            }
        })
        .to_string();

    // Match inline math spans
    let inline_re =
        Regex::new(r#"<span data-math-style="inline">([^<]*)</span>"#).unwrap();
    result = inline_re
        .replace_all(&result, |caps: &regex::Captures| {
            let latex_raw = caps.get(1).map(|m| m.as_str()).unwrap_or("");
            let latex = decode_html_entities(latex_raw);
            match latex_to_svg(&latex, false) {
                Ok(svg) => format!(r#"<span class="math math-inline">{svg}</span>"#),
                Err(_) => format!(
                    r#"<span class="math math-inline math-error">${}$</span>"#,
                    latex
                ),
            }
        })
        .to_string();

    // Also handle math code blocks (```math)
    let code_re = Regex::new(
        r#"<pre><code class="language-math" data-math-style="display">([^<]*)</code></pre>"#,
    )
    .unwrap();
    result = code_re
        .replace_all(&result, |caps: &regex::Captures| {
            let latex_raw = caps.get(1).map(|m| m.as_str()).unwrap_or("").trim();
            let latex = decode_html_entities(latex_raw);
            match latex_to_svg(&latex, true) {
                Ok(svg) => format!(r#"<div class="math math-display">{svg}</div>"#),
                Err(_) => format!(
                    r#"<div class="math math-display math-error">$${}$$</div>"#,
                    latex
                ),
            }
        })
        .to_string();

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_headers() {
        assert!(convert("# Hello").contains("<h1>"));
        assert!(convert("## World").contains("<h2>"));
        assert!(convert("### Test").contains("<h3>"));
    }

    #[test]
    fn test_bold() {
        assert!(convert("**bold**").contains("<strong>"));
        assert!(convert("__bold__").contains("<strong>"));
    }

    #[test]
    fn test_italic() {
        assert!(convert("*italic*").contains("<em>"));
        assert!(convert("_italic_").contains("<em>"));
    }

    #[test]
    fn test_links() {
        let result = convert("[text](http://example.com)");
        assert!(result.contains("<a href="));
        assert!(result.contains("example.com"));
    }

    #[test]
    fn test_code_blocks() {
        let result = convert("```rust\nfn main() {}\n```");
        assert!(result.contains("<pre>"));
        assert!(result.contains("<code"));
        assert!(result.contains("fn main()"));
    }

    #[test]
    fn test_inline_code() {
        assert!(convert("`code`").contains("<code>"));
    }

    #[test]
    fn test_horizontal_rule() {
        assert!(convert("---").contains("<hr"));
    }

    #[test]
    fn test_blockquote() {
        assert!(convert("> quote").contains("<blockquote>"));
    }

    #[test]
    fn test_unordered_list() {
        let result = convert("- item1\n- item2");
        assert!(result.contains("<ul>"));
        assert!(result.contains("<li>"));
    }

    #[test]
    fn test_ordered_list() {
        let result = convert("1. first\n2. second");
        assert!(result.contains("<ol>"));
        assert!(result.contains("<li>"));
    }

    #[test]
    fn test_task_list() {
        let result = convert("- [ ] todo\n- [x] done");
        // Should use Unicode symbols instead of HTML checkboxes
        assert!(result.contains("⬜")); // Unchecked: WHITE LARGE SQUARE
        assert!(result.contains("✅")); // Checked: WHITE HEAVY CHECK MARK
        assert!(!result.contains(r#"type="checkbox""#)); // No HTML checkbox
    }

    #[test]
    fn test_table() {
        let result = convert("| A | B |\n|---|---|\n| 1 | 2 |");
        assert!(result.contains("<table>"));
        assert!(result.contains("<th>"));
        assert!(result.contains("<td>"));
    }

    #[test]
    fn test_strikethrough() {
        assert!(convert("~~deleted~~").contains("<del>"));
    }

    #[test]
    fn test_images() {
        let result = convert("![alt](http://example.com/img.png)");
        assert!(result.contains("<img"));
        assert!(result.contains("src="));
    }

    #[test]
    fn test_autolink() {
        let result = convert("Visit https://example.com for more.");
        assert!(result.contains("<a href="));
    }

    #[test]
    fn test_display_math() {
        let result = convert("$$x^2 + y^2 = z^2$$");
        assert!(result.contains("math-display"));
        assert!(result.contains("<img"));
        assert!(result.contains("data:image/png;base64") || result.contains("data:image/svg+xml;base64"));
    }

    #[test]
    fn test_inline_math() {
        let result = convert("The equation $E = mc^2$ is famous.");
        assert!(result.contains("math-inline"));
        assert!(result.contains("<img"));
        assert!(result.contains("data:image/png;base64") || result.contains("data:image/svg+xml;base64"));
    }

    #[test]
    fn test_math_does_not_match_double_dollar() {
        let result = convert("$$x^2$$");
        assert!(result.contains("math-display"));
        assert!(!result.contains("math-inline"));
    }

    #[test]
    fn test_math_code_block() {
        let result = convert("```math\nx^2 + y^2\n```");
        assert!(result.contains("math-display"));
        assert!(result.contains("<img"));
    }

    #[test]
    fn test_math_split_environment() {
        // Test complex LaTeX that requires display mode
        let result = convert(r#"$$
\begin{split}
p_n &= 1-\frac{1}{2^r} \\
q_n &= \frac{1}{2^r}
\end{split}
$$"#);
        assert!(result.contains("math-display"));
        assert!(result.contains("<img"));
        // Should NOT have parse error
        assert!(!result.contains("math-error"));
    }

}
