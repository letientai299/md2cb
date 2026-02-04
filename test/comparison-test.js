// Playwright test to compare browser clipboard with md2cb output
const { chromium } = require('playwright');
const { execSync } = require('child_process');
const fs = require('fs');
const path = require('path');

const EDITOR_URL = 'http://localhost:9090';
const MARKSERV_URL = 'http://localhost:9091';

async function sleep(ms) {
  return new Promise(resolve => setTimeout(resolve, ms));
}

async function getEditorHtml(page) {
  return await page.evaluate(() => {
    const editor = document.querySelector('.fr-element');
    return editor ? editor.innerHTML : '';
  });
}

async function clearEditor(page) {
  await page.evaluate(() => {
    const editor = document.querySelector('.fr-element');
    if (editor) editor.innerHTML = '';
  });
}

async function pasteToEditor(page) {
  await page.click('.fr-element');
  await page.keyboard.press('Meta+v');
  await sleep(500); // Wait for paste to complete
}

// Extract and normalize HTML for comparison
function normalizeHtml(html) {
  return html
    // Remove data URIs for comparison (they differ in implementation)
    .replace(/data:[^"]+/g, 'DATA_URI')
    // Normalize whitespace
    .replace(/\s+/g, ' ')
    // Remove style attributes (browsers add them)
    .replace(/\s*style="[^"]*"/g, '')
    // Remove class attributes for cleaner comparison
    .replace(/\s*class="[^"]*"/g, '')
    // Normalize self-closing tags
    .replace(/<br\s*\/?>/gi, '<br>')
    .replace(/<hr\s*\/?>/gi, '<hr>')
    .replace(/<img([^>]*)\/>/gi, '<img$1>')
    .trim();
}

async function testFile(browser, mdFile) {
  const context = await browser.newContext({
    permissions: ['clipboard-read', 'clipboard-write']
  });
  const page = await context.newPage();

  const results = {
    file: mdFile,
    browserHtml: '',
    md2cbHtml: '',
    browserErrors: [],
    md2cbErrors: []
  };

  try {
    // Step 1: Get browser-rendered HTML
    const mdPath = path.basename(mdFile);
    const mdDir = path.dirname(mdFile);

    // Navigate to markserv for browser clipboard
    await page.goto(`${MARKSERV_URL}/${mdFile}`);
    await sleep(1000);

    // Select all and copy
    await page.keyboard.press('Meta+a');
    await page.keyboard.press('Meta+c');
    await sleep(300);

    // Paste to editor
    await page.goto(EDITOR_URL);
    await page.waitForSelector('.fr-element', { timeout: 10000 });
    await sleep(500);
    await clearEditor(page);
    await pasteToEditor(page);

    results.browserHtml = await getEditorHtml(page);

    // Step 2: Get md2cb output
    const fullPath = path.join(process.cwd(), mdFile);
    try {
      execSync(`cat "${fullPath}" | ./md2cb`, { stdio: ['inherit', 'pipe', 'pipe'] });
    } catch (e) {
      results.md2cbErrors.push(e.message);
    }

    // Paste md2cb output to editor
    await clearEditor(page);
    await pasteToEditor(page);

    results.md2cbHtml = await getEditorHtml(page);

  } catch (error) {
    results.browserErrors.push(error.message);
  } finally {
    await context.close();
  }

  return results;
}

async function runComparisonTests() {
  console.log('Starting comparison tests...\n');

  const browser = await chromium.launch({
    headless: false,
    args: ['--disable-web-security']
  });

  const testFiles = [
    'test/demo.md',
    ...fs.readdirSync('test/edge-cases')
      .filter(f => f.endsWith('.md'))
      .map(f => `test/edge-cases/${f}`)
  ];

  const results = [];

  for (const file of testFiles) {
    console.log(`Testing: ${file}`);
    const result = await testFile(browser, file);
    results.push(result);

    // Report findings
    const browserNorm = normalizeHtml(result.browserHtml);
    const md2cbNorm = normalizeHtml(result.md2cbHtml);

    if (result.browserErrors.length > 0) {
      console.log(`  ❌ Browser errors: ${result.browserErrors.join(', ')}`);
    }
    if (result.md2cbErrors.length > 0) {
      console.log(`  ❌ md2cb errors: ${result.md2cbErrors.join(', ')}`);
    }

    // Check for key elements
    const checks = [
      { name: 'Tables', pattern: /<table/i },
      { name: 'Lists', pattern: /<[ou]l/i },
      { name: 'Code', pattern: /<code/i },
      { name: 'Headers', pattern: /<h[1-6]/i },
      { name: 'Links', pattern: /<a /i },
      { name: 'Blockquotes', pattern: /<blockquote/i },
      { name: 'Images', pattern: /<img/i },
      { name: 'Bold', pattern: /<strong/i },
      { name: 'Italic', pattern: /<em/i }
    ];

    for (const check of checks) {
      const inBrowser = check.pattern.test(result.browserHtml);
      const inMd2cb = check.pattern.test(result.md2cbHtml);

      if (inBrowser && !inMd2cb) {
        console.log(`  ⚠️  ${check.name}: Present in browser, missing in md2cb`);
      } else if (!inBrowser && inMd2cb) {
        console.log(`  📝 ${check.name}: Present in md2cb, missing in browser`);
      }
    }

    // Check for math errors
    if (result.md2cbHtml.includes('math-error')) {
      console.log(`  ❌ Math rendering error detected`);
    }

    // Check for checkboxes (task lists)
    const md2cbHasCheckboxes = /[✅⬜☑☐]|checkbox/.test(result.md2cbHtml);
    const sourceHasTaskList = fs.readFileSync(file, 'utf-8').includes('- [');
    if (sourceHasTaskList && !md2cbHasCheckboxes) {
      console.log(`  ⚠️  Task list checkboxes may not be rendering correctly`);
    }

    console.log(`  ✓ Completed\n`);
  }

  await browser.close();

  // Summary
  console.log('\n=== Summary ===');
  console.log(`Tested ${results.length} files`);

  // Save detailed results
  const reportPath = 'test/comparison-report.json';
  fs.writeFileSync(reportPath, JSON.stringify(results, null, 2));
  console.log(`Detailed results saved to: ${reportPath}`);
}

runComparisonTests().catch(console.error);
