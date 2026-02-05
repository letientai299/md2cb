mod clipboard;
mod images;
mod js_runtime;
mod parser;
mod svg_render;

use std::env;
use std::fs;
use std::io::{self, Read, Write};
use std::process::Command;

const VERSION: &str = env!("GIT_VERSION");
const REPO_URL: &str = "https://github.com/letientai299/md2cb";

fn print_help() {
    eprintln!(
        "md2cb - Convert Markdown to rich HTML clipboard content

USAGE:
    md2cb [OPTIONS] [FILE/STDIN]
    cat file.md | md2cb

OPTIONS:
    -e, --edit       Open $EDITOR to edit before converting
    -h, --help       Print this help message
    -V, --version    Print version information

DESCRIPTION:
    Reads Markdown from stdin, converts it to styled HTML, and copies
    the result to the system clipboard. The HTML can then be pasted
    into rich text editors like Microsoft Word, Google Docs, Slack, etc.

FEATURES:
    - GitHub Flavored Markdown (tables, task lists, strikethrough, etc.)
    - Math equations rendered as PNG images (embedded MathJax)
    - Images automatically inlined as base64 data URIs

{REPO_URL}"
    );
}

fn print_version() {
    eprintln!("md2cb {VERSION}\n{REPO_URL}");
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
    let mut file =
        fs::File::create(&temp_path).map_err(|e| format!("Failed to create temp file: {e}"))?;
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
    let content =
        fs::read_to_string(&temp_path).map_err(|e| format!("Failed to read temp file: {e}"))?;

    // Clean up temp file
    let _ = fs::remove_file(&temp_path);

    Ok(content)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_args_help() {
        let args = vec!["--help".to_string()];
        let config = parse_args(&args).unwrap();
        assert!(config.show_help);
    }

    #[test]
    fn test_parse_args_version() {
        let args = vec!["-V".to_string()];
        let config = parse_args(&args).unwrap();
        assert!(config.show_version);
    }

    #[test]
    fn test_parse_args_edit() {
        let args = vec!["--edit".to_string()];
        let config = parse_args(&args).unwrap();
        assert!(config.edit_mode);
    }

    #[test]
    fn test_parse_args_file() {
        let args = vec!["readme.md".to_string()];
        let config = parse_args(&args).unwrap();
        assert_eq!(config.input_file.as_deref(), Some("readme.md"));
    }

    #[test]
    fn test_parse_args_file_and_edit() {
        let args = vec!["-e".to_string(), "readme.md".to_string()];
        let config = parse_args(&args).unwrap();
        assert!(config.edit_mode);
        assert_eq!(config.input_file.as_deref(), Some("readme.md"));
    }

    #[test]
    fn test_parse_args_unknown_option() {
        let args = vec!["--foo".to_string()];
        let err = parse_args(&args).unwrap_err();
        assert!(err.contains("unknown option"));
    }

    #[test]
    fn test_parse_args_too_many_args() {
        let args = vec!["file1".to_string(), "file2".to_string()];
        let err = parse_args(&args).unwrap_err();
        assert!(err.contains("too many arguments"));
    }

    #[test]
    fn test_temp_file_path() {
        let path = temp_file_path();
        assert!(path.extension().unwrap() == "md");
        assert!(path.to_string_lossy().contains("md2cb-"));
    }
}

/// Check if stdin is a terminal (no piped input)
fn stdin_is_terminal() -> bool {
    use std::io::IsTerminal;
    std::io::stdin().is_terminal()
}

fn parse_args(args: &[String]) -> Result<Config, String> {
    let mut config = Config::default();
    let mut positional = Vec::new();

    for arg in args {
        match arg.as_str() {
            "--help" | "-h" => config.show_help = true,
            "--version" | "-V" => config.show_version = true,
            "--edit" | "-e" => config.edit_mode = true,
            s if s.starts_with('-') => return Err(format!("unknown option '{s}'")),
            _ => positional.push(arg.clone()),
        }
    }

    if positional.len() > 1 {
        return Err("too many arguments".to_string());
    }

    config.input_file = positional.first().cloned();
    Ok(config)
}

#[derive(Default, Debug, PartialEq)]
struct Config {
    input_file: Option<String>,
    edit_mode: bool,
    show_help: bool,
    show_version: bool,
}

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();

    let config = match parse_args(&args) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("error: {e}");
            eprintln!("Usage: md2cb [OPTIONS] [FILE]");
            eprintln!("Try 'md2cb --help' for more information.");
            std::process::exit(1);
        }
    };

    // Handle --help
    if config.show_help {
        print_help();
        return;
    }

    // Handle --version
    if config.show_version {
        print_version();
        return;
    }

    let input_file = config.input_file.as_deref();
    let edit_mode = config.edit_mode;

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
