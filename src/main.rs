mod clipboard;
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

    // Build full HTML document with CSS
    let css = include_str!("../assets/github-markdown.css");
    let full_html = format!(
        r#"<!DOCTYPE html>
<html>
<head>
<meta charset="utf-8">
<style>{css}</style>
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
