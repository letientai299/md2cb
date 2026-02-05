//! GitHub Flavored Markdown to HTML converter using comrak.

use comrak::plugins::syntect::SyntectAdapterBuilder;
use comrak::{Options, Plugins, markdown_to_html_with_plugins};
use regex::Regex;
use std::sync::LazyLock;

use crate::js_runtime;
use crate::svg_render;

// Static regex patterns - compiled once and reused
static PRE_BG_COLOR_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r#"<pre style="background-color:#[0-9a-fA-F]+;">"#).unwrap());

static CHECKED_CHECKBOX_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r#"<input[^>]*type="checkbox"[^>]*checked[^>]*/?\s*>"#).unwrap());

static UNCHECKED_CHECKBOX_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r#"<input[^>]*type="checkbox"[^>]*/?\s*>"#).unwrap());

static DISPLAY_MATH_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r#"<span data-math-style="display">([^<]*)</span>"#).unwrap());

static INLINE_MATH_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r#"<span data-math-style="inline">([^<]*)</span>"#).unwrap());

static MATH_CODE_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r#"<pre><code class="language-math" data-math-style="display">([^<]*)</code></pre>"#)
        .unwrap()
});

static MERMAID_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r#"<pre[^>]*><code class="language-mermaid">([\s\S]*?)</code></pre>"#).unwrap()
});

static SPAN_TAG_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r#"</?span[^>]*>"#).unwrap());

// Match code blocks for newline conversion (pre tag with code inside)
static CODE_BLOCK_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r#"(<pre[^>]*><code[^>]*>)([\s\S]*?)(</code></pre>)"#).unwrap());

// Match whitespace between block-level tags (should be removed, not converted to space)
static BLOCK_TAG_WHITESPACE_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r">\s+<").unwrap());

// Static comrak options - built once and reused
static COMRAK_OPTIONS: LazyLock<Options> = LazyLock::new(build_options);

// Static syntect adapter for syntax highlighting - built once and reused
static SYNTECT_ADAPTER: LazyLock<comrak::plugins::syntect::SyntectAdapter> =
    LazyLock::new(|| SyntectAdapterBuilder::new().build());

/// Converts GitHub Flavored Markdown to HTML.
pub fn convert(markdown: &str) -> String {
    // Set up plugins with the cached syntax highlighter adapter
    let mut plugins = Plugins::default();
    plugins.render.codefence_syntax_highlighter = Some(&*SYNTECT_ADAPTER);

    // Convert markdown to HTML using comrak with math support and syntax highlighting
    let html = markdown_to_html_with_plugins(markdown, &COMRAK_OPTIONS, &plugins);

    // Post-process: convert checkboxes to Unicode for compatibility
    let html = convert_checkboxes_to_unicode(&html);

    // Post-process: convert LaTeX in math spans to SVG using MathJax
    let html = convert_math_to_svg(&html);

    // Post-process: convert Mermaid code blocks to PNG images
    // Note: must run BEFORE fix_pre_background_color so the regex matches
    let html = convert_mermaid_to_png(&html);

    // Post-process: fix background-color in pre tags for proper code block styling
    // The syntect adapter adds white background which doesn't match GitHub styling
    let html = fix_pre_background_color(&html);

    // Post-process: convert newlines to <br> in code blocks for Teams compatibility
    let html = convert_code_block_newlines(&html);

    // Post-process: convert remaining newlines to spaces for rich text editor compatibility
    // (code blocks already have newlines converted to <br>, so this only affects regular content)
    normalize_whitespace(&html)
}

/// Normalizes whitespace in HTML content.
/// - Removes whitespace between HTML tags (block-level structure)
/// - Converts newlines within text content to spaces (inline text wrapping)
///
/// Code blocks are unaffected since their newlines have already been converted to `<br>` tags.
fn normalize_whitespace(html: &str) -> String {
    // First, remove whitespace between tags (preserves block structure without gaps)
    let collapsed = BLOCK_TAG_WHITESPACE_RE.replace_all(html, "><");
    // Then replace any remaining newlines (within text content) with spaces
    collapsed.replace('\n', " ")
}

/// Replaces syntect's background-color in pre tags with GitHub's code block background.
/// Syntect uses white (#ffffff) which doesn't match GitHub styling.
/// We use GitHub's light-mode code block background (#f6f8fa) for better visibility.
/// Also adds monospace font-family for editors that strip CSS classes (e.g., Google Docs).
fn fix_pre_background_color(html: &str) -> String {
    PRE_BG_COLOR_RE.replace_all(
        html,
        r#"<pre style="background-color:#f6f8fa;padding:16px;border-radius:6px;overflow:auto;font-family:monospace;">"#,
    )
    .into_owned()
}

/// Converts newlines to `<br>` tags inside code blocks.
/// This ensures line breaks are preserved when pasting into rich text editors
/// like Microsoft Teams, which may strip plain newlines.
fn convert_code_block_newlines(html: &str) -> String {
    CODE_BLOCK_RE
        .replace_all(html, |caps: &regex::Captures| {
            let pre_open = caps.get(1).map(|m| m.as_str()).unwrap_or("");
            let content = caps.get(2).map(|m| m.as_str()).unwrap_or("");
            let close = caps.get(3).map(|m| m.as_str()).unwrap_or("");
            // Replace newlines with <br> to preserve line breaks in rich text editors
            let content_with_br = content.replace('\n', "<br>");
            // Remove trailing <br> that appears before </span></code> (from fenced code block syntax)
            let content_trimmed = content_with_br
                .strip_suffix("<br></span>")
                .map(|s| format!("{s}</span>"))
                .unwrap_or(content_with_br);
            format!("{pre_open}{content_trimmed}{close}")
        })
        .into_owned()
}

/// Converts HTML checkbox inputs to Unicode square symbols for better compatibility.
/// - Checked: ✅ (U+2705 WHITE HEAVY CHECK MARK)
/// - Unchecked: ⬜ (U+2B1C WHITE LARGE SQUARE)
fn convert_checkboxes_to_unicode(html: &str) -> String {
    // Match checked checkbox (has "checked" attribute)
    let result = CHECKED_CHECKBOX_RE.replace_all(html, "✅ ");

    // Match unchecked checkbox (no "checked" attribute)
    UNCHECKED_CHECKBOX_RE
        .replace_all(&result, "⬜ ")
        .into_owned()
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
fn decode_html_entities(s: &str) -> std::borrow::Cow<'_, str> {
    // Fast path: if no entities present, return borrowed reference
    if !s.contains('&') {
        return std::borrow::Cow::Borrowed(s);
    }
    // Slow path: decode entities
    std::borrow::Cow::Owned(
        s.replace("&amp;", "&")
            .replace("&lt;", "<")
            .replace("&gt;", ">")
            .replace("&quot;", "\"")
            .replace("&#39;", "'"),
    )
}

/// HTML-escapes a string.
fn html_escape(s: impl AsRef<str>) -> String {
    let s = s.as_ref();
    s.replace("&", "&amp;")
        .replace("<", "&lt;")
        .replace(">", "&gt;")
        .replace("\"", "&quot;")
        .replace("'", "&#39;")
}

/// Renders LaTeX to PNG image tag using embedded MathJax + resvg.
///
/// This function:
/// 1. Converts LaTeX to SVG using MathJax (via embedded QuickJS)
/// 2. Renders SVG to PNG using resvg (pure Rust)
/// 3. Returns an HTML img tag with base64-encoded PNG
fn latex_to_svg(latex: &str, display: bool) -> Result<String, String> {
    // Step 1: Convert LaTeX to SVG using embedded MathJax
    let svg = js_runtime::convert_latex_to_svg(latex, display)?;

    // Step 2: Render SVG to PNG using resvg
    let render_result = svg_render::render_svg_to_png(&svg)?;

    // Step 3: Build <img> tag with base64 PNG
    let data_uri = format!("data:image/png;base64,{}", render_result.png_base64);
    let alt = latex.replace('"', "&quot;");
    let style = if display {
        "display:block;margin:0.5em auto;"
    } else {
        "vertical-align:middle;"
    };

    Ok(format!(
        r#"<img src="{}" alt="{}" width="{}" height="{}" style="{}">"#,
        data_uri, alt, render_result.display_width, render_result.display_height, style
    ))
}

/// Converts LaTeX content in comrak's math spans to SVG using MathJax.
///
/// Comrak outputs math as:
/// - Inline: `<span data-math-style="inline">latex</span>`
/// - Display: `<span data-math-style="display">latex</span>`
///
/// This function converts the LaTeX content to inline SVG.
fn convert_math_to_svg(html: &str) -> String {
    // Match display math spans
    let result = DISPLAY_MATH_RE.replace_all(html, |caps: &regex::Captures| {
        let latex_raw = caps.get(1).map(|m| m.as_str()).unwrap_or("");
        let latex = decode_html_entities(latex_raw);
        match latex_to_svg(&latex, true) {
            Ok(svg) => format!(r#"<div class="math math-display">{svg}</div>"#),
            Err(_) => format!(
                r#"<div class="math math-display math-error">$${}$$</div>"#,
                html_escape(latex)
            ),
        }
    });

    // Match inline math spans
    let result = INLINE_MATH_RE.replace_all(&result, |caps: &regex::Captures| {
        let latex_raw = caps.get(1).map(|m| m.as_str()).unwrap_or("");
        let latex = decode_html_entities(latex_raw);
        match latex_to_svg(&latex, false) {
            Ok(svg) => format!(r#"<span class="math math-inline">{svg}</span>"#),
            Err(_) => format!(
                r#"<span class="math math-inline math-error">${}$</span>"#,
                html_escape(latex)
            ),
        }
    });

    // Also handle math code blocks (```math)
    MATH_CODE_RE
        .replace_all(&result, |caps: &regex::Captures| {
            let latex_raw = caps.get(1).map(|m| m.as_str()).unwrap_or("").trim();
            let latex = decode_html_entities(latex_raw);
            match latex_to_svg(&latex, true) {
                Ok(svg) => format!(r#"<div class="math math-display">{svg}</div>"#),
                Err(_) => format!(
                    r#"<div class="math math-display math-error">$${}$$</div>"#,
                    html_escape(latex)
                ),
            }
        })
        .into_owned()
}

/// Sanitizes SVG font-family attributes that contain unescaped quotes.
/// mermaid-rs-renderer generates invalid SVG with unescaped quotes in font-family:
/// font-family="Inter, ui-sans-serif, system-ui, -apple-system, "Segoe UI", sans-serif"
/// This function replaces the entire font-family value with a clean version.
fn sanitize_mermaid_svg(svg: &str) -> String {
    // The problematic pattern is font-family="...stuff with "quotes" inside..."
    // Rather than trying to parse the malformed XML, we'll do a simple string replacement
    // of the known problematic font-family value
    svg.replace(
        r#"font-family="Inter, ui-sans-serif, system-ui, -apple-system, "Segoe UI", sans-serif""#,
        r#"font-family="Inter, ui-sans-serif, system-ui, -apple-system, 'Segoe UI', sans-serif""#,
    )
}

/// Renders Mermaid diagram to PNG image tag.
///
/// This function:
/// 1. Converts Mermaid definition to SVG using mermaid-rs-renderer (pure Rust)
/// 2. Renders SVG to PNG using resvg (pure Rust)
/// 3. Returns an HTML img tag with base64-encoded PNG
fn mermaid_to_png(definition: &str) -> Result<String, String> {
    // Step 1: Convert Mermaid definition to SVG using native Rust library
    let svg = mermaid_rs_renderer::render(definition)
        .map_err(|e| format!("Mermaid rendering error: {e}"))?;

    // Step 1.5: Sanitize the SVG (fix invalid font-family attributes)
    let svg = sanitize_mermaid_svg(&svg);

    // Step 2: Render SVG to PNG using resvg
    let render_result = svg_render::render_svg_to_png(&svg)?;

    // Step 3: Build <img> tag with base64 PNG
    let data_uri = format!("data:image/png;base64,{}", render_result.png_base64);
    let alt = "Mermaid diagram";

    Ok(format!(
        r#"<img src="{}" alt="{}" width="{}" height="{}" style="display:block;margin:0.5em auto;">"#,
        data_uri, alt, render_result.display_width, render_result.display_height
    ))
}

/// Strips HTML span tags from content, preserving the text inside.
/// Used to clean up syntect's syntax highlighting spans from mermaid content.
fn strip_span_tags(html: &str) -> String {
    SPAN_TAG_RE.replace_all(html, "").into_owned()
}

/// Converts Mermaid code blocks to PNG images.
///
/// Comrak with syntect outputs mermaid code blocks as:
/// `<pre style="..."><code class="language-mermaid"><span>...</span></code></pre>`
///
/// This function converts the Mermaid content to PNG images.
fn convert_mermaid_to_png(html: &str) -> String {
    MERMAID_RE
        .replace_all(html, |caps: &regex::Captures| {
            let definition_raw = caps.get(1).map(|m| m.as_str()).unwrap_or("");
            // Strip span tags added by syntect syntax highlighting
            let definition_stripped = strip_span_tags(definition_raw);
            let definition = decode_html_entities(&definition_stripped);
            match mermaid_to_png(&definition) {
                Ok(img) => format!(r#"<div class="mermaid-diagram">{img}</div>"#),
                Err(e) => {
                    eprintln!("Mermaid rendering error: {e}");
                    format!(
                        r#"<pre class="mermaid-error"><code>{}</code></pre>"#,
                        html_escape(definition)
                    )
                }
            }
        })
        .into_owned()
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
        assert!(result.contains("<pre"));
        assert!(result.contains("<code"));
        // Syntax highlighting splits tokens across spans, so check for individual parts
        assert!(result.contains("fn "));
        assert!(result.contains("main"));
        // Check for syntax highlighting (inline styles)
        assert!(result.contains("style="));
        // Check for GitHub-style background color
        assert!(result.contains("background-color:#f6f8fa"));
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
        assert!(
            result.contains("data:image/png;base64")
                || result.contains("data:image/svg+xml;base64")
        );
    }

    #[test]
    fn test_inline_math() {
        let result = convert("The equation $E = mc^2$ is famous.");
        assert!(result.contains("math-inline"));
        assert!(result.contains("<img"));
        assert!(
            result.contains("data:image/png;base64")
                || result.contains("data:image/svg+xml;base64")
        );
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
        let result = convert(
            r#"$$
\begin{split}
p_n &= 1-\frac{1}{2^r} \\
q_n &= \frac{1}{2^r}
\end{split}
$$"#,
        );
        assert!(result.contains("math-display"));
        assert!(result.contains("<img"));
        // Should NOT have parse error
        assert!(!result.contains("math-error"));
    }

    #[test]
    fn test_mermaid_flowchart() {
        let result = convert("```mermaid\ngraph LR\n    A --> B\n```");
        assert!(result.contains("mermaid-diagram"));
        assert!(result.contains("<img"));
        assert!(result.contains("data:image/png;base64"));
    }

    #[test]
    fn test_mermaid_sequence_diagram() {
        let result = convert("```mermaid\nsequenceDiagram\n    Alice->>Bob: Hello\n```");
        assert!(result.contains("mermaid-diagram"));
        assert!(result.contains("<img"));
    }

    #[test]
    fn test_mermaid_complex_flowchart() {
        let result = convert(
            "```mermaid\ngraph TD\n    A[Start] --> B{Decision}\n    B -->|Yes| C[OK]\n    B -->|No| D[Cancel]\n```",
        );
        assert!(result.contains("mermaid-diagram"));
        assert!(result.contains("<img"));
        // Should NOT have error
        assert!(!result.contains("mermaid-error"));
    }

    #[test]
    fn test_code_block_newlines_converted_to_br() {
        let result = convert("```rust\nfn main() {\n    println!(\"Hello\");\n}\n```");
        assert!(result.contains("<pre"));
        assert!(result.contains("<code"));
        // Should have <br> tags instead of plain newlines inside code block
        assert!(result.contains("<br>"));
        // The code content should still be present
        assert!(result.contains("main"));
        assert!(result.contains("println"));
    }

    #[test]
    fn test_code_block_multiline_preserves_structure() {
        let result = convert("```python\ndef hello():\n    print('world')\n\nhello()\n```");
        // Count <br> occurrences - should have at least 3 (one per newline in code)
        let br_count = result.matches("<br>").count();
        assert!(
            br_count >= 3,
            "Expected at least 3 <br> tags, got {br_count}"
        );
    }

    #[test]
    fn test_code_block_no_trailing_br() {
        let result = convert("```rust\nfn main() {}\n```");
        // Should NOT have trailing <br> before </span></code>
        assert!(
            !result.contains("<br></span></code>"),
            "Should not have trailing <br> before closing tags"
        );
        // But should still have the code content
        assert!(result.contains("main"));
    }

    #[test]
    fn test_normalize_whitespace_removes_between_tags() {
        // Whitespace between closing and opening tags should be removed
        let html = "<ul>\n<li>item</li>\n</ul>";
        let result = normalize_whitespace(html);
        assert_eq!(result, "<ul><li>item</li></ul>");
    }

    #[test]
    fn test_normalize_whitespace_preserves_inline_content() {
        // Newlines within text content should become spaces
        let html = "<li>hello\nworld</li>";
        let result = normalize_whitespace(html);
        assert_eq!(result, "<li>hello world</li>");
    }

    #[test]
    fn test_normalize_whitespace_mixed() {
        // Mixed: remove between tags, preserve in content
        let html = "<ul>\n<li>hello\nworld</li>\n</ul>";
        let result = normalize_whitespace(html);
        assert_eq!(result, "<ul><li>hello world</li></ul>");
    }

    #[test]
    fn test_list_no_extra_whitespace_between_items() {
        let result = convert("- item1\n- item2\n- item3");
        // Should not have whitespace between </li> and <li>
        assert!(!result.contains("</li> <li>"));
        assert!(!result.contains("</li>\n<li>"));
        // Should have direct adjacency
        assert!(result.contains("</li><li>"));
    }

    #[test]
    fn test_nested_list_no_extra_whitespace() {
        let result = convert("- parent\n  - child1\n  - child2");
        // Should not have whitespace between list tags
        assert!(!result.contains("</li> <li>"));
        assert!(!result.contains("<ul> <li>"));
        assert!(!result.contains("</ul> </li>"));
    }

    #[test]
    fn test_inline_html_spacing_preserved() {
        // HTML tags split across lines should preserve spacing
        let result = convert("- <u>underline</u>, <span>orange</span>,\n  <strong>strong</strong>");
        // Should have space before <strong> (from the newline in content)
        assert!(result.contains("</span>, <strong>"));
    }

    #[test]
    fn test_paragraph_structure_preserved() {
        let result = convert("First paragraph.\n\nSecond paragraph.");
        // Should have separate <p> tags (semantic structure preserved)
        assert!(result.contains("<p>First paragraph.</p>"));
        assert!(result.contains("<p>Second paragraph.</p>"));
        // Tags should be adjacent (no whitespace between)
        assert!(result.contains("</p><p>"));
    }
}
