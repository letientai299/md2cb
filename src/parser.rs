//! GitHub Flavored Markdown to HTML converter using comrak.

use comrak::plugins::syntect::SyntectAdapterBuilder;
use comrak::{markdown_to_html_with_plugins, Options, Plugins};
use regex::Regex;

use crate::js_runtime;
use crate::svg_render;

/// Converts GitHub Flavored Markdown to HTML.
pub fn convert(markdown: &str) -> String {
    // Build syntect adapter for syntax highlighting with inline styles
    // Using the default "InspiredGitHub" theme which matches our GitHub-style output
    let adapter = SyntectAdapterBuilder::new().build();

    // Set up plugins with the syntax highlighter
    let mut plugins = Plugins::default();
    plugins.render.codefence_syntax_highlighter = Some(&adapter);

    // Convert markdown to HTML using comrak with math support and syntax highlighting
    let options = build_options();
    let html = markdown_to_html_with_plugins(markdown, &options, &plugins);

    // Post-process: convert checkboxes to Unicode for compatibility
    let html = convert_checkboxes_to_unicode(&html);

    // Post-process: convert LaTeX in math spans to SVG using MathJax
    let html = convert_math_to_svg(&html);

    // Post-process: convert Mermaid code blocks to PNG images
    // Note: must run BEFORE fix_pre_background_color so the regex matches
    let html = convert_mermaid_to_png(&html);

    // Post-process: fix background-color in pre tags for proper code block styling
    // The syntect adapter adds white background which doesn't match GitHub styling
    fix_pre_background_color(&html)
}

/// Replaces syntect's background-color in pre tags with GitHub's code block background.
/// Syntect uses white (#ffffff) which doesn't match GitHub styling.
/// We use GitHub's light-mode code block background (#f6f8fa) for better visibility.
fn fix_pre_background_color(html: &str) -> String {
    let re = Regex::new(r#"<pre style="background-color:#[0-9a-fA-F]+;">"#).unwrap();
    re.replace_all(
        html,
        r#"<pre style="background-color:#f6f8fa;padding:16px;border-radius:6px;overflow:auto;">"#,
    )
    .to_string()
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
        .map_err(|e| format!("Mermaid rendering error: {}", e))?;

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
    let span_re = Regex::new(r#"</?span[^>]*>"#).unwrap();
    span_re.replace_all(html, "").to_string()
}

/// Converts Mermaid code blocks to PNG images.
///
/// Comrak with syntect outputs mermaid code blocks as:
/// `<pre style="..."><code class="language-mermaid"><span>...</span></code></pre>`
///
/// This function converts the Mermaid content to PNG images.
fn convert_mermaid_to_png(html: &str) -> String {
    // Match pre tags with optional style attributes (from syntect)
    // The content may contain span tags from syntax highlighting
    let mermaid_re = Regex::new(
        r#"<pre[^>]*><code class="language-mermaid">([\s\S]*?)</code></pre>"#,
    )
    .unwrap();

    mermaid_re
        .replace_all(html, |caps: &regex::Captures| {
            let definition_raw = caps.get(1).map(|m| m.as_str()).unwrap_or("");
            // Strip span tags added by syntect syntax highlighting
            let definition_stripped = strip_span_tags(definition_raw);
            let definition = decode_html_entities(&definition_stripped);
            match mermaid_to_png(&definition) {
                Ok(img) => format!(r#"<div class="mermaid-diagram">{img}</div>"#),
                Err(e) => {
                    eprintln!("Mermaid rendering error: {}", e);
                    format!(
                        r#"<pre class="mermaid-error"><code>{}</code></pre>"#,
                        definition
                    )
                }
            }
        })
        .to_string()
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
        let result = convert("```mermaid\ngraph TD\n    A[Start] --> B{Decision}\n    B -->|Yes| C[OK]\n    B -->|No| D[Cancel]\n```");
        assert!(result.contains("mermaid-diagram"));
        assert!(result.contains("<img"));
        // Should NOT have error
        assert!(!result.contains("mermaid-error"));
    }
}
