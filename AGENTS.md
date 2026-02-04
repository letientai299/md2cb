# AGENTS.md

## Project Purpose

`md2cb` is a cross-platform command-line tool that converts GitHub Flavored
Markdown (GFM) to rich HTML clipboard content. The goal is to produce clipboard
output that rich text editors can properly render with GitHub-style formatting.

This enables users to paste formatted markdown into rich text editors like
Microsoft Word, Teams, Slack, Google Docs, Apple Pages, etc., with proper
styling preserved.

## Architecture

The tool is written in Rust for cross-platform support (macOS, Windows, Linux):

1. **Markdown → HTML**: Custom regex-based parser converts GFM to HTML
2. **CSS Embedding**: GitHub markdown CSS is embedded at compile time
3. **Clipboard**: Uses `arboard` crate for cross-platform HTML clipboard support

### Source Structure

```
src/
├── main.rs        # Entry point, reads stdin, builds HTML document
├── parser.rs      # GFM to HTML converter with full test suite
└── clipboard.rs   # Cross-platform clipboard operations

assets/
└── github-markdown.css  # GitHub's official markdown CSS (embedded at compile)
```

## Development

### Prerequisites

- Rust 1.70+ (install via rustup)
- Docker (for dev server)
- pnpm (for markserv)

### Make targets

```bash
make          # Build release binary
make test     # Run tests
make dev      # Start dev servers and open in browser
make dev-stop # Stop servers
make install  # Install to /usr/local/bin
```

- **http://localhost:9090** - Rich text editor (Froala) for paste testing
- **http://localhost:9091/demo.md** - Markdown preview with MarkServ

### Workflow

1. Edit `src/parser.rs` or other source files
2. Run `make` to rebuild
3. Test with `cat test/demo.md | ./md2cb`
4. Paste into the editor at localhost:9090 to verify output
5. Run `make test` to ensure all tests pass

## Supported Markdown Features

- Headers (H1-H6)
- Tables with column alignment (left/center/right)
- Task lists (checkboxes)
- Nested lists (ordered and unordered)
- Fenced code blocks with language classes
- Blockquotes
- Horizontal rules
- Links (inline, reference-style, auto-links)
- Images with alt text and title
- Bold, italic, strikethrough, inline code
- HTML passthrough

## Testing with Playwright

Use Playwright to automate comparison testing between browser copy and md2cb
output.

### Test Strategy

1. **Browser baseline**: Navigate to localhost:9091/demo.md, select all, copy
2. **md2cb output**: Run `cat test/demo.md | ./md2cb`
3. **Compare**: Paste both into the editor and compare HTML structure

## Common Issues

### Nested lists not working

Check `handle_list_item()` indent tracking logic in `src/parser.rs`.

### HTML tags stripped

Ensure they're not being escaped in `convert_paragraphs()` or
`convert_inline_elements()`.

### Styles not applied

Verify CSS in `assets/github-markdown.css` is being properly embedded. The CSS
uses CSS variables for theming - ensure the target editor supports them.

## Custom CSS

The CSS file at `assets/github-markdown.css` is from the `github-markdown-css`
npm package. To update or customize:

1. Download new CSS from https://github.com/sindresorhus/github-markdown-css
2. Replace `assets/github-markdown.css`
3. Rebuild with `make`

The CSS is embedded at compile time using `include_str!()`.
