# AGENTS.md

## Rules for AI Agents

1. **Test your own work** - Never ask the user to test for you. Run automated tests
   (unit tests, integration tests, Playwright E2E tests) to verify changes work
   correctly before reporting completion. Use `node test/paste-test.js` to verify
   clipboard output renders correctly in the Froala editor.

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

The tool is written in Rust for cross-platform support (macOS, Windows, Linux):

1. **Markdown → HTML**: Uses `comrak` crate (GFM-compliant CommonMark parser)
2. **Math → PNG Images**: Uses MathJax (Node.js) to render LaTeX to PNG images embedded as base64 data URIs
3. **Image Inlining**: Fetches images and embeds as base64 data URIs
4. **CSS Embedding**: GitHub markdown CSS embedded at compile time
5. **Clipboard**: Uses `arboard` crate for cross-platform HTML clipboard support

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
├── images.rs      # Image inlining (URL to base64 data URI)
└── clipboard.rs   # Cross-platform clipboard operations

scripts/
└── math-to-svg.js # MathJax-based LaTeX to PNG converter (called by parser.rs)

assets/
└── github-markdown.css  # GitHub's markdown CSS (embedded at compile)

test/
├── demo.md              # Test markdown with all GFM features
└── paste-test.js        # Playwright E2E test for paste verification
```

## Development

### Prerequisites

- Rust 1.70+ (install via rustup)
- Node.js 18+ (for MathJax math rendering)
- npm packages: `npm install mathjax-full canvas` (for math rendering)
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
- Images with alt text and title (auto-inlined as base64)
- Bold, italic, strikethrough, inline code
- **Math** - inline (`$...$`) and display (`$$...$$`) rendered as PNG images via MathJax
- HTML passthrough

## Image Inlining

Images are automatically converted to base64 data URIs, so pasted content
contains the actual image data. This works for:

- Remote images (HTTP/HTTPS URLs)
- Local images (relative paths)

The image data is embedded directly in the HTML, ensuring the image displays
correctly when pasted into any rich text editor.

## Math Support

LaTeX math expressions are converted to PNG images using MathJax:

- **Inline math**: `$E = mc^2$` → renders as inline image
- **Display math**: `$$\int_0^\infty e^{-x^2} dx$$` → renders as centered block image
- **Math code blocks**: ` ```math ` blocks also render as display math images

The PNG approach ensures math renders correctly in all rich text editors. MathJax
supports all standard LaTeX environments including `split`, `aligned`, `matrix`,
`cases`, `pmatrix`, etc.

**Prerequisites**: Node.js and npm packages (`mathjax-full`, `canvas`) must be
installed for math rendering.

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

Math expressions use MathJax via Node.js. Check that:
- Node.js is installed and `scripts/math-to-svg.js` is accessible
- npm packages are installed: `npm install mathjax-full canvas`
- Display math uses `$$...$$` (must be on own line for block display)
- Inline math uses `$...$` (no spaces around dollar signs)
- Alignment characters (`&`) require proper environment (`\begin{aligned}`, `\begin{split}`, etc.)
- Run `node test/paste-test.js` to verify math renders in editor

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
