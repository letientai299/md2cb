# AGENTS.md

## Project Purpose

`md2cb` is a macOS command-line tool that converts GitHub Flavored Markdown
(GFM) to rich HTML clipboard content. The goal is to produce clipboard output
identical to what a browser produces when copying rendered markdown—with full
computed inline styles.

This enables users to paste formatted markdown into rich text editors like
Microsoft Word, Teams, Slack, Google Docs, Apple Pages, etc., with proper
styling preserved.

## Architecture

The tool uses a WebKit-based approach:

1. **Markdown → HTML**: Custom parser converts GFM to HTML
2. **HTML → Rendered**: WebKit renders HTML with GitHub-style CSS
3. **Rendered → Clipboard**: WebKit's native copy
   (`document.execCommand('copy')`) captures computed styles

This approach ensures the clipboard content matches browser behavior exactly.

## Development

### Prerequisites

- macOS 13+
- Swift 5.9+
- Docker
- pnpm

### Make targets

```bash
make          # Build release binary
make dev      # Start dev servers and open in browser
make dev-stop # Stop servers
```

- **http://localhost:9090** - Rich text editor (Froala) for paste testing
- **http://localhost:9091/demo.md** - Markdown preview with MarkServ

### Workflow

1. Edit `Sources/md2cb/md2cb.swift`
2. Run `make` to rebuild
3. Test with `cat test/demo.md | ./md2cb`
4. Paste into the editor at localhost:9090 to verify output
5. Double verify with Playwright MCP

## Testing with Playwright

Use Playwright to automate comparison testing between browser copy and md2cb
output.

### Test Strategy

1. **Browser baseline**: Navigate to localhost:9091/demo.md, select all, copy
2. **md2cb output**: Run `cat test/demo.md | ./md2cb`
3. **Compare**: Paste both into the editor and compare HTML structure

### Example Playwright Test

```javascript
import { test, expect } from "@playwright/test";

test("md2cb matches browser clipboard", async ({ page, context }) => {
  // Grant clipboard permissions
  await context.grantPermissions(["clipboard-read", "clipboard-write"]);

  // Get browser clipboard content
  await page.goto("http://localhost:9091/demo.md");
  await page.keyboard.press("Meta+a");
  await page.keyboard.press("Meta+c");
  const browserClipboard = await page.evaluate(() =>
    navigator.clipboard.readText(),
  );

  // Get md2cb clipboard content (run externally, then read)
  // ... execute: cat test/demo.md | ./md2cb

  // Paste into editor and verify
  await page.goto("http://localhost:9090");
  await page.click(".fr-element"); // Froala editor
  await page.keyboard.press("Meta+v");

  // Assert expected elements are present
  await expect(page.locator("table")).toBeVisible();
  await expect(page.locator("h1")).toContainText("GFM Feature Demos");
});
```

## Common Issues

### Nested lists not working

Check `handleListItem()` indent tracking logic.

### HTML tags stripped

Ensure they're not being escaped in `convertParagraphs()` or
`convertInlineElements()`.

### Styles not applied

Verify CSS in `Styles.css` and that WebKit is fully rendering before copy.
