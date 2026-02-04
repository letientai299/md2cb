mod clipboard;
mod images;
mod parser;

use std::env;
use std::io::{self, Read};

const VERSION: &str = env!("CARGO_PKG_VERSION");

fn print_help() {
    eprintln!(
        "md2cb - Convert GitHub Flavored Markdown to rich HTML clipboard content

USAGE:
    md2cb [OPTIONS]
    cat file.md | md2cb

OPTIONS:
    -h, --help       Print this help message
    -V, --version    Print version information

DESCRIPTION:
    Reads Markdown from stdin, converts it to styled HTML, and copies
    the result to the system clipboard. The HTML can then be pasted
    into rich text editors like Microsoft Word, Google Docs, Slack, etc.

FEATURES:
    - GitHub Flavored Markdown (tables, task lists, strikethrough, etc.)
    - Math equations rendered as images (requires Node.js + MathJax)
    - Images automatically inlined as base64 data URIs
    - GitHub-style CSS embedded for consistent styling

EXAMPLES:
    cat README.md | md2cb
    echo '# Hello' | md2cb
    md2cb < document.md

REQUIREMENTS:
    - For math rendering: Node.js with mathjax-full and canvas packages"
    );
}

fn print_version() {
    eprintln!("md2cb {VERSION}");
}

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();

    // Handle --help
    if args.iter().any(|a| a == "--help" || a == "-h") {
        print_help();
        return;
    }

    // Handle --version
    if args.iter().any(|a| a == "--version" || a == "-V") {
        print_version();
        return;
    }

    // Check for unknown flags
    for arg in &args {
        if arg.starts_with('-') {
            eprintln!("error: unknown option '{arg}'");
            eprintln!("Usage: md2cb [OPTIONS]");
            eprintln!("Try 'md2cb --help' for more information.");
            std::process::exit(1);
        }
    }

    // Check for unexpected positional arguments
    if !args.is_empty() {
        eprintln!("error: unexpected argument '{}'", args[0]);
        eprintln!("Usage: cat file.md | md2cb");
        eprintln!("Try 'md2cb --help' for more information.");
        std::process::exit(1);
    }

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
