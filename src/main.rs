mod clipboard;
mod images;
mod parser;

use std::io::{self, Read};

fn main() {
    // Read markdown from stdin
    let mut markdown = String::new();
    io::stdin()
        .read_to_string(&mut markdown)
        .expect("Failed to read from stdin");

    // Convert to HTML
    let html = parser::convert(&markdown);

    // Inline images (convert URLs to base64 data URIs)
    let html = images::inline_images(&html, None);

    // Build full HTML document with CSS
    let markdown_css = include_str!("../assets/github-markdown.css");
    let full_html = format!(
        r#"<!DOCTYPE html>
<html>
<head>
<meta charset="utf-8">
<style>{markdown_css}</style>
</head>
<body class="markdown-body">{html}</body>
</html>"#
    );

    // Copy to clipboard
    match clipboard::copy_html(&full_html) {
        Ok(()) => eprintln!("Copied to clipboard"),
        Err(e) => {
            eprintln!("Error copying to clipboard: {e}");
            std::process::exit(1);
        }
    }
}
