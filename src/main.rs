mod clipboard;
mod images;
mod js_runtime;
mod parser;
mod svg_render;

use std::env;
use std::fs;
use std::io::{self, Read, Write};
use std::process::Command;

const VERSION: &str = env!("CARGO_PKG_VERSION");

fn print_help() {
    eprintln!(
        "md2cb - Convert GitHub Flavored Markdown to rich HTML clipboard content

USAGE:
    md2cb [OPTIONS] [FILE]
    cat file.md | md2cb

OPTIONS:
    -e, --edit       Open $EDITOR to edit markdown before converting
    -h, --help       Print this help message
    -V, --version    Print version information

ARGS:
    [FILE]           Input markdown file

DESCRIPTION:
    Reads Markdown from stdin, converts it to styled HTML, and copies
    the result to the system clipboard. The HTML can then be pasted
    into rich text editors like Microsoft Word, Google Docs, Slack, etc.

FEATURES:
    - GitHub Flavored Markdown (tables, task lists, strikethrough, etc.)
    - Math equations rendered as PNG images (embedded MathJax)
    - Images automatically inlined as base64 data URIs
    - GitHub-style CSS embedded for consistent styling
    - Single binary with no external dependencies

EXAMPLES:
    md2cb README.md            # Convert file directly
    cat README.md | md2cb      # Convert from stdin
    echo '# Hello' | md2cb
    md2cb < document.md
    md2cb -e                   # Open editor with empty file
    md2cb -e README.md         # Edit file before converting
    cat README.md | md2cb -e   # Edit piped content before converting"
    );
}

fn print_version() {
    eprintln!("md2cb {VERSION}");
}

/// Generate a random temp file path with .md extension
fn temp_file_path() -> std::path::PathBuf {
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let pid = std::process::id();
    std::env::temp_dir().join(format!("md2cb-{pid}-{timestamp}.md"))
}

/// Open the file in $EDITOR and return the edited content
fn edit_in_editor(initial_content: &str) -> Result<String, String> {
    let editor = env::var("EDITOR").unwrap_or_else(|_| "vi".to_string());
    let temp_path = temp_file_path();

    // Write initial content to temp file
    let mut file = fs::File::create(&temp_path)
        .map_err(|e| format!("Failed to create temp file: {e}"))?;
    file.write_all(initial_content.as_bytes())
        .map_err(|e| format!("Failed to write temp file: {e}"))?;
    drop(file);

    // Open editor
    let status = Command::new(&editor)
        .arg(&temp_path)
        .status()
        .map_err(|e| format!("Failed to open editor '{editor}': {e}"))?;

    if !status.success() {
        let _ = fs::remove_file(&temp_path);
        return Err(format!("Editor exited with status: {status}"));
    }

    // Read back the edited content
    let content = fs::read_to_string(&temp_path)
        .map_err(|e| format!("Failed to read temp file: {e}"))?;

    // Clean up temp file
    let _ = fs::remove_file(&temp_path);

    Ok(content)
}

/// Check if stdin is a terminal (no piped input)
fn stdin_is_terminal() -> bool {
    use std::io::IsTerminal;
    std::io::stdin().is_terminal()
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

    // Check for --edit flag
    let edit_mode = args.iter().any(|a| a == "--edit" || a == "-e");

    // Check for unknown flags
    for arg in &args {
        if arg.starts_with('-') && arg != "-e" && arg != "--edit" {
            eprintln!("error: unknown option '{arg}'");
            eprintln!("Usage: md2cb [OPTIONS] [FILE]");
            eprintln!("Try 'md2cb --help' for more information.");
            std::process::exit(1);
        }
    }

    // Get positional arguments (file path)
    let positional: Vec<_> = args.iter().filter(|a| !a.starts_with('-')).collect();

    // Only one file argument allowed
    if positional.len() > 1 {
        eprintln!("error: too many arguments");
        eprintln!("Usage: md2cb [OPTIONS] [FILE]");
        eprintln!("Try 'md2cb --help' for more information.");
        std::process::exit(1);
    }

    let input_file = positional.first().map(|s| s.as_str());

    // Read markdown content and track base path for relative image resolution
    let mut markdown = String::new();
    let base_path: Option<std::path::PathBuf> = if let Some(file_path) = input_file {
        // Read from file
        let path = std::path::Path::new(file_path);
        markdown = fs::read_to_string(path).unwrap_or_else(|e| {
            eprintln!("error: cannot read '{file_path}': {e}");
            std::process::exit(1);
        });
        // Use the file's parent directory for resolving relative image paths
        path.canonicalize()
            .ok()
            .and_then(|p| p.parent().map(|p| p.to_path_buf()))
    } else if edit_mode && stdin_is_terminal() {
        // No stdin input and no file, start with empty content
        None
    } else {
        // Read from stdin
        io::stdin()
            .read_to_string(&mut markdown)
            .expect("Failed to read from stdin");
        None
    };

    // If edit mode, open editor
    if edit_mode {
        match edit_in_editor(&markdown) {
            Ok(edited) => markdown = edited,
            Err(e) => {
                eprintln!("Error: {e}");
                std::process::exit(1);
            }
        }
    }

    // Convert to HTML
    let html = parser::convert(&markdown);

    // Inline images (convert URLs to base64 data URIs)
    // Use the markdown file's directory for resolving relative image paths
    let html = images::inline_images(&html, base_path.as_deref());

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
