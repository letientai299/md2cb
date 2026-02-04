#!/usr/bin/env node
/**
 * E2E Test Suite for md2cb
 *
 * This script tests the md2cb tool by:
 * 1. Converting markdown files to clipboard HTML using the md2cb tool
 * 2. Pasting the result into a Froala rich text editor
 * 3. Taking a screenshot of the rendered output
 * 4. Comparing against browser-native copy/paste from markserv
 *
 * Prerequisites:
 *   - Dev servers running: `make dev`
 *   - md2cb binary built: `make`
 *   - Node dependencies: `pnpm install`
 *
 * Usage:
 *   node scripts/e2e-test.js              # Run all tests (headless)
 *   node scripts/e2e-test.js 01-basic     # Run tests matching "01-basic"
 *   UI=true node scripts/e2e-test.js      # Run with visible browser
 *   node scripts/e2e-test.js --help       # Show help
 *
 * Output:
 *   Screenshots are saved to e2e/screenshots/
 *   - {name}-tool.png    : Result from md2cb tool
 *   - {name}-browser.png : Result from browser copy/paste
 *
 * Environment Variables:
 *   EDITOR_PORT   - Froala editor port (default: 9090)
 *   MARKSERV_PORT - Markserv port (default: 9091)
 *   UI            - Set to "true" for visible browser (debugging)
 *
 * Modes:
 *   Headless (default): Runs in background. Reads system clipboard to paste.
 *   UI mode (UI=true):  Visible browser. Uses OS-level paste keystroke.
 *
 * Platform Support (for reading system clipboard):
 *   - macOS: Swift/NSPasteboard
 *   - Linux: xclip or xsel
 *   - Windows: PowerShell
 */

const { chromium } = require("playwright");
const { execSync, spawn, exec } = require("child_process");
const fs = require("fs");
const path = require("path");

// Configuration
const CONFIG = {
  editorPort: process.env.EDITOR_PORT || 9090,
  markservPort: process.env.MARKSERV_PORT || 9091,
  headless: process.env.UI !== "true", // Headless by default, visible when UI=true
  fixturesDir: path.join(__dirname, "..", "e2e", "fixtures"),
  screenshotsDir: path.join(__dirname, "..", "e2e", "screenshots"),
  md2cbPath: path.join(__dirname, "..", "md2cb"),
  platform: process.platform,
};

// ANSI color codes for terminal output
const colors = {
  reset: "\x1b[0m",
  bright: "\x1b[1m",
  dim: "\x1b[2m",
  red: "\x1b[31m",
  green: "\x1b[32m",
  yellow: "\x1b[33m",
  blue: "\x1b[34m",
  cyan: "\x1b[36m",
};

function log(message, color = "") {
  console.log(`${color}${message}${colors.reset}`);
}

function logStep(step, message) {
  console.log(`${colors.cyan}[${step}]${colors.reset} ${message}`);
}

function logSuccess(message) {
  console.log(`${colors.green}✓${colors.reset} ${message}`);
}

function logError(message) {
  console.log(`${colors.red}✗${colors.reset} ${message}`);
}

/**
 * Get list of test fixture files, optionally filtered by pattern
 * @param {string|null} filter - Optional filter pattern
 * @returns {string[]} Array of fixture file paths
 */
function getFixtures(filter = null) {
  const files = fs
    .readdirSync(CONFIG.fixturesDir)
    .filter((f) => f.endsWith(".md"))
    .sort();

  if (filter) {
    return files.filter((f) => f.includes(filter));
  }
  return files;
}

/**
 * Run md2cb tool and copy markdown to clipboard
 * @param {string} mdPath - Path to markdown file
 * @returns {boolean} True if successful
 */
function runMd2cb(mdPath) {
  try {
    execSync(`"${CONFIG.md2cbPath}" "${mdPath}"`, {
      stdio: ["pipe", "pipe", "pipe"],
    });
    return true;
  } catch (error) {
    logError(`md2cb failed: ${error.message}`);
    return false;
  }
}

/**
 * Send paste keystroke using OS-level automation (cross-platform)
 * This triggers a real paste from the system clipboard.
 * Requires non-headless mode with a visible, focused window.
 */
function sendPasteKeystroke() {
  switch (CONFIG.platform) {
    case "darwin":
      // macOS: Use osascript to send Cmd+V
      execSync(`osascript -e 'tell application "System Events" to keystroke "v" using command down'`);
      break;
    case "linux":
      // Linux: Use xdotool to send Ctrl+V
      execSync("xdotool key ctrl+v");
      break;
    case "win32":
      // Windows: Use PowerShell to send Ctrl+V
      execSync(`powershell -Command "$wshell = New-Object -ComObject wscript.shell; $wshell.SendKeys('^v')"`);
      break;
    default:
      throw new Error(`Unsupported platform: ${CONFIG.platform}`);
  }
}

/**
 * Read raw HTML from system clipboard (cross-platform)
 * Returns the exact clipboard content without any processing.
 * @returns {string|null} Raw HTML or null if failed
 */
function readClipboardHtml() {
  try {
    switch (CONFIG.platform) {
      case "darwin": {
        // macOS: Use Swift to read HTML pasteboard
        const script = `import AppKit; if let h = NSPasteboard.general.string(forType: .html) { print(h) }`;
        return execSync(`swift -e '${script}'`, { encoding: "utf-8", maxBuffer: 50 * 1024 * 1024 });
      }
      case "linux": {
        // Linux: Use xclip to read HTML clipboard
        try {
          return execSync("xclip -selection clipboard -t text/html -o", { encoding: "utf-8", maxBuffer: 50 * 1024 * 1024 });
        } catch {
          return execSync("xsel --clipboard --output", { encoding: "utf-8", maxBuffer: 50 * 1024 * 1024 });
        }
      }
      case "win32": {
        // Windows: Use PowerShell to read HTML clipboard
        const ps = `Add-Type -A System.Windows.Forms; [System.Windows.Forms.Clipboard]::GetText([System.Windows.Forms.TextDataFormat]::Html)`;
        return execSync(`powershell -Command "${ps}"`, { encoding: "utf-8", maxBuffer: 50 * 1024 * 1024 });
      }
      default:
        return null;
    }
  } catch (e) {
    return null;
  }
}

/**
 * Get the platform-appropriate modifier key (Meta for macOS, Control for others)
 */
function getModifier() {
  return CONFIG.platform === "darwin" ? "Meta" : "Control";
}

/**
 * Clear the Froala editor content
 * @param {Page} page - Playwright page
 */
async function clearEditor(page) {
  // Wait for editor to be fully loaded and editable
  const editor = page.locator(".fr-element");
  await editor.waitFor({ state: "visible" });
  await page.waitForTimeout(300);

  // Click into editor and select all, then delete
  await editor.click();
  await page.keyboard.press(`${getModifier()}+a`);
  await page.keyboard.press("Backspace");
  // Wait for editor to be empty
  await page.waitForTimeout(200);
}

/**
 * Paste clipboard content into Froala editor using browser keyboard
 * @param {Page} page - Playwright page
 */
async function pasteToEditor(page) {
  const editor = page.locator(".fr-element");
  await editor.click();
  await page.keyboard.press(`${getModifier()}+v`);
  // Wait for paste to complete and images to load
  await page.waitForTimeout(1000);
  // Wait for any images to load
  await page.waitForLoadState("networkidle").catch(() => {});
}

/**
 * Paste from system clipboard into Froala editor.
 * - UI mode: Uses OS-level keystroke for real paste behavior
 * - Headless mode: Reads clipboard and writes to browser clipboard, then pastes
 * @param {Page} page - Playwright page
 */
async function pasteFromSystemClipboard(page) {
  const editor = page.locator(".fr-element");
  await editor.click();
  await page.waitForTimeout(200);

  if (!CONFIG.headless) {
    // UI mode: Use OS-level paste for real clipboard behavior
    await page.bringToFront();
    sendPasteKeystroke();
  } else {
    // Headless mode: Read system clipboard and write to browser clipboard
    const html = readClipboardHtml();
    if (html) {
      await page.evaluate(async (content) => {
        const blob = new Blob([content], { type: "text/html" });
        await navigator.clipboard.write([new ClipboardItem({ "text/html": blob })]);
      }, html);
    }
    // Trigger paste
    await page.keyboard.press(`${getModifier()}+v`);
  }

  // Wait for paste to complete and images to load
  await page.waitForTimeout(1500);
  await page.waitForLoadState("networkidle").catch(() => {});
}

/**
 * Take a screenshot of the editor content
 * @param {Page} page - Playwright page
 * @param {string} name - Screenshot name (without extension)
 */
async function takeEditorScreenshot(page, name) {
  const editor = page.locator(".fr-element");
  const screenshotPath = path.join(CONFIG.screenshotsDir, `${name}.png`);
  await editor.screenshot({ path: screenshotPath });
  return screenshotPath;
}

/**
 * Navigate to markserv and copy rendered markdown
 * @param {Page} page - Playwright page
 * @param {string} mdFilename - Markdown filename (relative to fixtures)
 */
async function copyFromMarkserv(page, mdFilename) {
  // markserv serves files from e2e/fixtures directory
  const url = `http://localhost:${CONFIG.markservPort}/${mdFilename}`;
  await page.goto(url);
  await page.waitForLoadState("networkidle");

  // Select all content and copy
  const mod = getModifier();
  await page.keyboard.press(`${mod}+a`);
  await page.keyboard.press(`${mod}+c`);
  await page.waitForTimeout(200);
}

/**
 * Run a single test for a markdown file
 * @param {Browser} browser - Playwright browser
 * @param {string} mdFilename - Markdown filename
 * @returns {Object} Test result
 */
async function runTest(browser, mdFilename) {
  const testName = mdFilename.replace(".md", "");
  logStep("TEST", `${colors.bright}${testName}${colors.reset}`);

  const mdPath = path.join(CONFIG.fixturesDir, mdFilename);

  // Grant clipboard permissions for headless mode
  const context = await browser.newContext({
    permissions: ["clipboard-read", "clipboard-write"],
  });
  const page = await context.newPage();

  const results = {
    name: testName,
    toolScreenshot: null,
    browserScreenshot: null,
    toolSuccess: false,
    browserSuccess: false,
  };

  try {
    // ===== Test 1: md2cb tool output =====
    logStep("1/4", "Running md2cb tool...");
    if (!runMd2cb(mdPath)) {
      throw new Error("md2cb tool failed");
    }

    logStep("2/4", "Pasting tool output to editor...");
    await page.goto(`http://localhost:${CONFIG.editorPort}`);
    await page.waitForSelector(".fr-element", { state: "visible" });
    await clearEditor(page);
    await pasteFromSystemClipboard(page);

    const toolScreenshot = await takeEditorScreenshot(page, `${testName}-tool`);
    results.toolScreenshot = toolScreenshot;
    results.toolSuccess = true;
    logSuccess(`Tool screenshot: ${path.basename(toolScreenshot)}`);

    // ===== Test 2: Browser copy for comparison =====
    logStep("3/4", "Copying from markserv...");
    await copyFromMarkserv(page, mdFilename);

    logStep("4/4", "Pasting browser copy to editor...");
    await page.goto(`http://localhost:${CONFIG.editorPort}`);
    await page.waitForSelector(".fr-element", { state: "visible" });
    await clearEditor(page);
    await pasteToEditor(page);

    const browserScreenshot = await takeEditorScreenshot(
      page,
      `${testName}-browser`
    );
    results.browserScreenshot = browserScreenshot;
    results.browserSuccess = true;
    logSuccess(`Browser screenshot: ${path.basename(browserScreenshot)}`);
  } catch (error) {
    logError(`Test failed: ${error.message}`);
  } finally {
    await context.close();
  }

  return results;
}

/**
 * Check if dev servers are running
 * @returns {Object} Server status
 */
async function checkServers() {
  const results = { editor: false, markserv: false };

  try {
    const response = await fetch(
      `http://localhost:${CONFIG.editorPort}`
    );
    results.editor = response.ok;
  } catch {}

  try {
    const response = await fetch(
      `http://localhost:${CONFIG.markservPort}`
    );
    results.markserv = response.ok;
  } catch {}

  return results;
}

/**
 * Start markserv for e2e fixtures directory
 * @returns {Object} Object with kill method to stop the server
 */
function startMarkserv() {
  // Run markserv as a background process using exec
  // Use relative path from project root since markserv has issues with absolute paths
  // Use --no-browser to prevent opening system browser
  const cmd = `pnpx markserv --no-browser -p ${CONFIG.markservPort} ./e2e/fixtures`;
  const proc = exec(cmd, { cwd: path.join(__dirname, "..") });

  // Log critical errors only (filter out deprecation warnings and ENOENT)
  proc.stderr?.on("data", (data) => {
    const msg = data.toString();
    if (!msg.includes("DeprecationWarning") && !msg.includes("ENOENT")) {
      console.error(`markserv: ${msg}`);
    }
  });

  return {
    kill: () => {
      try {
        proc.kill();
        execSync(`pkill -f "markserv.*${CONFIG.markservPort}"`, { stdio: "ignore" });
      } catch {}
    }
  };
}

/**
 * Wait for a server to be ready
 * @param {string} url - URL to check
 * @param {number} maxWait - Maximum wait time in ms
 * @returns {boolean} True if server is ready
 */
async function waitForServer(url, maxWait = 15000) {
  const startTime = Date.now();
  let attempts = 0;
  while (Date.now() - startTime < maxWait) {
    attempts++;
    try {
      const response = await fetch(url);
      if (response.ok || response.status === 404) {
        return true;
      }
    } catch (e) {
      // Connection refused is expected while server is starting
    }
    await new Promise((resolve) => setTimeout(resolve, 500));
  }
  return false;
}

/**
 * Print usage help
 */
function printHelp() {
  console.log(`
${colors.bright}md2cb E2E Test Suite${colors.reset}

${colors.cyan}Usage:${colors.reset}
  node scripts/e2e-test.js              Run all tests (headless)
  node scripts/e2e-test.js <filter>     Run tests matching filter
  UI=true node scripts/e2e-test.js      Run with visible browser
  node scripts/e2e-test.js --help       Show this help

${colors.cyan}Examples:${colors.reset}
  node scripts/e2e-test.js              # Run all tests headless
  node scripts/e2e-test.js 01-basic     # Run only 01-basic-text.md
  UI=true node scripts/e2e-test.js      # Debug with visible browser

${colors.cyan}Environment Variables:${colors.reset}
  EDITOR_PORT=9090      Froala editor port
  MARKSERV_PORT=9091    Markserv preview port
  UI=true               Show browser window (for debugging)

${colors.cyan}Prerequisites:${colors.reset}
  1. Build the tool:     make
  2. Start dev servers:  make dev
  3. Install deps:       pnpm install

${colors.cyan}Platform Requirements:${colors.reset}
  macOS:   Swift (for clipboard reading)
  Linux:   xclip or xsel
  Windows: PowerShell

${colors.cyan}Output:${colors.reset}
  Screenshots saved to e2e/screenshots/
  - {name}-tool.png    : md2cb tool output
  - {name}-browser.png : Browser copy/paste result
`);
}

/**
 * Main entry point
 */
async function main() {
  const args = process.argv.slice(2);

  // Handle help flag
  if (args.includes("--help") || args.includes("-h")) {
    printHelp();
    process.exit(0);
  }

  // Get filter from args
  const filter = args[0] || null;

  log(`\n${colors.bright}md2cb E2E Test Suite${colors.reset}\n`);

  // Check prerequisites
  if (!fs.existsSync(CONFIG.md2cbPath)) {
    logError(`md2cb binary not found at ${CONFIG.md2cbPath}`);
    logError("Run 'make' to build the tool first.");
    process.exit(1);
  }

  // Create screenshots directory
  if (!fs.existsSync(CONFIG.screenshotsDir)) {
    fs.mkdirSync(CONFIG.screenshotsDir, { recursive: true });
  }

  // Check servers
  log("Checking dev servers...");
  const servers = await checkServers();

  if (!servers.editor) {
    logError(`Editor server not running on port ${CONFIG.editorPort}`);
    logError("Run 'make dev' to start dev servers.");
    process.exit(1);
  }

  // For markserv, we need it to serve e2e/fixtures, not test/
  // We'll start our own instance
  let markservProc = null;
  if (!servers.markserv) {
    log("Starting markserv for e2e fixtures...");
    markservProc = startMarkserv();
    // Wait for it to be ready
    const markservUrl = `http://localhost:${CONFIG.markservPort}`;
    log(`Waiting for ${markservUrl}...`);
    const ready = await waitForServer(markservUrl, 20000);
    if (!ready) {
      logError(`Markserv failed to start on port ${CONFIG.markservPort}`);
      logError("Try running: pnpx markserv -p 9091 ./e2e/fixtures");
      if (markservProc) markservProc.kill();
      process.exit(1);
    }
  } else {
    log(
      `${colors.yellow}Note: Using existing markserv. Make sure it serves e2e/fixtures.${colors.reset}`
    );
  }

  logSuccess(
    `Editor: http://localhost:${CONFIG.editorPort}`
  );
  logSuccess(
    `Markserv: http://localhost:${CONFIG.markservPort}`
  );

  // Get fixtures to test
  const fixtures = getFixtures(filter);
  if (fixtures.length === 0) {
    logError(`No fixtures found${filter ? ` matching "${filter}"` : ""}`);
    if (markservProc) markservProc.kill();
    process.exit(1);
  }

  const mode = CONFIG.headless
    ? `${colors.dim}(headless)${colors.reset}`
    : `${colors.green}(UI mode)${colors.reset}`;
  log(`\nRunning ${fixtures.length} test(s) ${mode}\n`);

  // Launch browser
  const browser = await chromium.launch({
    headless: CONFIG.headless,
  });

  // Run tests
  const results = [];
  for (const fixture of fixtures) {
    const result = await runTest(browser, fixture);
    results.push(result);
    console.log(""); // Blank line between tests
  }

  // Cleanup
  await browser.close();
  if (markservProc) {
    markservProc.kill();
  }

  // Print summary
  log(`${colors.bright}Summary${colors.reset}`);
  log("─".repeat(50));

  let passed = 0;
  let failed = 0;

  for (const result of results) {
    const toolStatus = result.toolSuccess
      ? `${colors.green}✓${colors.reset}`
      : `${colors.red}✗${colors.reset}`;
    const browserStatus = result.browserSuccess
      ? `${colors.green}✓${colors.reset}`
      : `${colors.red}✗${colors.reset}`;

    console.log(`${result.name}`);
    console.log(`  Tool: ${toolStatus}  Browser: ${browserStatus}`);

    if (result.toolSuccess && result.browserSuccess) {
      passed++;
    } else {
      failed++;
    }
  }

  log("─".repeat(50));
  log(
    `${colors.green}Passed: ${passed}${colors.reset}  ${colors.red}Failed: ${failed}${colors.reset}`
  );

  log(`\nScreenshots saved to: ${CONFIG.screenshotsDir}`);

  process.exit(failed > 0 ? 1 : 0);
}

main().catch((error) => {
  logError(`Unexpected error: ${error.message}`);
  console.error(error);
  process.exit(1);
});
