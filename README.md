# md2cb

Markdown to clipboard converter. Converts GitHub Flavored Markdown (GFM) to rich
HTML and copies it to the system clipboard for pasting into Word, Google Docs,
Pages, etc.

## Usage

```bash
cat file.md | md2cb
# or
md2cb < file.md
```

## Features

- **Headers** (h1-h6) with border-bottom for h1/h2
- **Tables** with column alignment (left/center/right)
- **Task lists** → ☑/☐ Unicode symbols
- **Nested lists** (ordered and unordered)
- **Fenced code blocks** with language class
- **Blockquotes** with left border
- **Horizontal rules**
- **Links** (inline, reference-style, auto-links)
- **Images**
- **Bold**, _italic_, ~~strikethrough~~, `inline code`
- **HTML passthrough** (`<sub>`, `<sup>`, `<u>`, etc.)

## Project Structure

```
src/
├── main.rs        # Entry point, reads stdin, builds HTML document
├── parser.rs      # GFM to HTML converter with tests
└── clipboard.rs   # Cross-platform clipboard operations

assets/
└── github-markdown.css  # Embedded GitHub CSS
```

## Build

```bash
make              # Build release binary → ./md2cb
make clean        # Clean build artifacts
```

## Development

```bash
make dev          # Start test servers and open in browser
make dev-stop     # Stop test servers
```

- Rich text editor: http://localhost:9090
- Markdown preview: http://localhost:9091/demo.md

## Requirements

- Rust 1.70+
- Docker (for dev server)
- pnpm (for markserv)
