# md2cb

Markdown to clipboard converter for macOS. Converts GFM markdown to rich HTML
and copies to clipboard for pasting into Word, Google Docs, Pages, etc.

Uses WebKit to render markdown with GitHub-style CSS, producing clipboard
content identical to browser copy behavior.

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
md2cb-swift/
├── Sources/md2cb/
│   └── md2cb.swift    # Main source file
├── test/
│   ├── demo.md        # Test markdown file
│   └── index.html     # Rich text editor for paste testing
├── Makefile
└── Package.swift
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

- macOS 13+
- Swift 5.9+
- Docker (for dev server)
- pnpm (for markserv)
