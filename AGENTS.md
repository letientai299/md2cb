# AGENTS.md

## Rules for AI Agents

1. **Test your own work** - Never ask the user to test for you. Run automated tests
   (unit tests, integration tests, Playwright E2E tests) to verify changes work
   correctly before reporting completion. Use Playwright to paste md2cb output
   into the Froala editor (localhost:9090) to verify clipboard output renders correctly.

2. **Use existing tools** - Don't implement custom parsers or converters when
   established crates/libraries exist. Search crates.io for existing solutions first.

3. **Verify in real environment** - For clipboard/paste functionality, actually test
   the output in the target editor (localhost:9090) using Playwright, not just unit tests.

## Project Purpose

`md2cb` is a cross-platform command-line tool that converts GitHub Flavored
Markdown (GFM) to rich HTML clipboard content. The goal is to produce clipboard
output that rich text editors can properly render with GitHub-style formatting.

This enables users to paste formatted markdown into rich text editors like
Microsoft Word, Teams, Slack, Google Docs, Apple Pages, etc., with proper
styling preserved.

## Architecture

The tool is written in Rust for cross-platform support (macOS, Windows, Linux).
**Single binary with no external runtime dependencies.**

1. **Markdown → HTML**: Uses `comrak` crate (GFM-compliant CommonMark parser)
2. **Math → PNG Images**: Uses embedded MathJax (via QuickJS) + resvg for LaTeX → SVG → PNG
3. **Mermaid → PNG Images**: Uses `mermaid-rs-renderer` (pure Rust) + resvg for diagrams → SVG → PNG
4. **Image Inlining**: Fetches images and embeds as base64 data URIs
5. **CSS Embedding**: GitHub markdown CSS embedded at compile time
6. **Clipboard**: Uses `arboard` crate for cross-platform HTML clipboard support

### Why PNG Images for Math?

Rich text editors (Froala, Word, Google Docs, etc.) sanitize pasted HTML:
- **KaTeX HTML** gets stripped (CSS classes removed, nested spans flattened)
- **MathML** is not rendered by most editors
- **SVG** data URIs are blocked by many editors
- **PNG images** are universally supported

### Source Structure

```
src/
├── main.rs        # Entry point, reads stdin, builds HTML document
├── parser.rs      # GFM to HTML converter with full test suite
├── js_runtime.rs  # Embedded QuickJS runtime for MathJax
├── svg_render.rs  # SVG to PNG conversion using resvg
├── images.rs      # Image inlining (URL to base64 data URI)
└── clipboard.rs   # Cross-platform clipboard operations

assets/
├── github-markdown.css  # GitHub's markdown CSS (embedded at compile)
└── mathjax-bundle.js    # Bundled MathJax for embedded JS engine

scripts/
└── build-mathjax.js     # Script to regenerate mathjax-bundle.js

test/
├── demo.md              # Test markdown with all GFM features
└── index.html           # Froala editor for paste testing
```

## Development

### Prerequisites

- Rust 1.70+ (install via rustup)
- Docker (for dev server)
- pnpm (for markserv)
- Node.js (only for Playwright tests and regenerating mathjax-bundle.js)

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
- Images with alt text and title (auto-inlined as base64)
- Bold, italic, strikethrough, inline code
- **Math** - inline (`$...$`) and display (`$$...$$`) rendered as PNG images via MathJax
- **Mermaid diagrams** - flowcharts, sequence diagrams, etc. rendered as PNG images
- HTML passthrough

## Image Inlining

Images are automatically converted to base64 data URIs, so pasted content
contains the actual image data. This works for:

- Remote images (HTTP/HTTPS URLs)
- Local images (relative paths)

The image data is embedded directly in the HTML, ensuring the image displays
correctly when pasted into any rich text editor.

## Math Support

LaTeX math expressions are converted to PNG images using embedded MathJax:

- **Inline math**: `$E = mc^2$` → renders as inline image
- **Display math**: `$$\int_0^\infty e^{-x^2} dx$$` → renders as centered block image
- **Math code blocks**: ` ```math ` blocks also render as display math images

The PNG approach ensures math renders correctly in all rich text editors. MathJax
supports all standard LaTeX environments including `split`, `aligned`, `matrix`,
`cases`, `pmatrix`, etc.

**No external dependencies**: MathJax runs in an embedded QuickJS JavaScript
engine, and SVG→PNG conversion uses resvg (pure Rust). Everything is bundled
into the single binary.

## Mermaid Support

Mermaid diagrams in fenced code blocks are converted to PNG images:

```markdown
\`\`\`mermaid
graph LR
    A[Start] --> B{Decision}
    B -->|Yes| C[OK]
    B -->|No| D[Cancel]
\`\`\`
```

Supported diagram types (13 total):
- **Core**: Flowchart, Sequence, Class, State
- **Data**: ER Diagram, Pie Chart, XY Chart, Quadrant Chart
- **Planning**: Gantt, Timeline, Journey
- **Other**: Mindmap, Git Graph

**No external dependencies**: Uses `mermaid-rs-renderer`, a pure Rust Mermaid
parser and renderer. SVG→PNG conversion uses resvg. Everything is bundled into
the single binary with no Node.js or browser required.

## Testing with Playwright

Use Playwright to automate comparison testing between browser copy and md2cb
output.

### Test Strategy

1. **Browser baseline**: Navigate to localhost:9091/demo.md, select all, copy
2. **md2cb output**: Run `cat test/demo.md | ./md2cb`
3. **Compare**: Paste both into the editor and compare HTML structure

## Common Issues

### Markdown not rendering correctly

The parser uses `comrak` which is GFM-compliant. Check if your markdown follows
GFM syntax. Enable `options.render.unsafe_` for raw HTML passthrough.

### Math not rendering

Math expressions use embedded MathJax (no external dependencies). Check that:
- Display math uses `$$...$$` (must be on own line for block display)
- Inline math uses `$...$` (no spaces around dollar signs)
- Alignment characters (`&`) require proper environment (`\begin{aligned}`, `\begin{split}`, etc.)
- Use Playwright to paste into Froala editor (localhost:9090) to verify math renders

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
