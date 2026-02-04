// Playwright visual comparison test
const { chromium } = require('playwright');
const { execSync } = require('child_process');
const fs = require('fs');
const path = require('path');

const EDITOR_URL = 'http://localhost:9090';
const MARKSERV_URL = 'http://localhost:9091';
const SCREENSHOT_DIR = 'test/screenshots';

async function sleep(ms) {
  return new Promise(resolve => setTimeout(resolve, ms));
}

async function clearEditor(page) {
  await page.evaluate(() => {
    const editor = document.querySelector('.fr-element');
    if (editor) editor.innerHTML = '';
  });
}

async function getEditorHtml(page) {
  return await page.evaluate(() => {
    const editor = document.querySelector('.fr-element');
    return editor ? editor.innerHTML : '';
  });
}

async function testFile(browser, mdFile, index) {
  const context = await browser.newContext({
    permissions: ['clipboard-read', 'clipboard-write']
  });
  const page = await context.newPage();

  const baseName = path.basename(mdFile, '.md');
  const results = {
    file: mdFile,
    browserHtml: '',
    md2cbHtml: '',
    issues: []
  };

  try {
    // --- Test 1: Browser clipboard (copy from rendered markdown) ---
    console.log(`  [1/2] Testing browser clipboard...`);
    await page.goto(`${MARKSERV_URL}/${mdFile}`);
    await sleep(1000);

    // Select all and copy
    await page.keyboard.press('Meta+a');
    await sleep(200);
    await page.keyboard.press('Meta+c');
    await sleep(500);

    // Paste to editor
    await page.goto(EDITOR_URL);
    await page.waitForSelector('.fr-element', { timeout: 10000 });
    await sleep(500);
    await clearEditor(page);

    await page.click('.fr-element');
    await page.keyboard.press('Meta+v');
    await sleep(500);

    results.browserHtml = await getEditorHtml(page);

    // Take screenshot
    await page.screenshot({
      path: `${SCREENSHOT_DIR}/${String(index).padStart(2, '0')}-${baseName}-browser.png`,
      fullPage: false
    });

    // --- Test 2: md2cb output ---
    console.log(`  [2/2] Testing md2cb output...`);

    const fullPath = path.join(process.cwd(), mdFile);
    execSync(`cat "${fullPath}" | ./md2cb`, { stdio: 'pipe' });
    await sleep(200);

    await clearEditor(page);
    await page.click('.fr-element');
    await page.keyboard.press('Meta+v');
    await sleep(500);

    results.md2cbHtml = await getEditorHtml(page);

    // Take screenshot
    await page.screenshot({
      path: `${SCREENSHOT_DIR}/${String(index).padStart(2, '0')}-${baseName}-md2cb.png`,
      fullPage: false
    });

    // --- Compare and find issues ---
    analyzeResults(results, mdFile);

  } catch (error) {
    results.issues.push(`Error: ${error.message}`);
  } finally {
    await context.close();
  }

  return results;
}

function analyzeResults(results, mdFile) {
  const source = fs.readFileSync(mdFile, 'utf-8');
  const browserHtml = results.browserHtml;
  const md2cbHtml = results.md2cbHtml;

  // Check for tables
  if (source.includes('|') && source.includes('---')) {
    if (!/<table/i.test(md2cbHtml)) {
      results.issues.push('Tables not rendered');
    }
  }

  // Check for code blocks
  if (source.includes('```') || source.includes('    ')) {
    if (!/<pre/i.test(md2cbHtml) && !/<code/i.test(md2cbHtml)) {
      results.issues.push('Code blocks not rendered');
    }
  }

  // Check for task lists
  if (source.includes('- [x]') || source.includes('- [ ]')) {
    if (!/[✅⬜☑☐]/.test(md2cbHtml) && !/checkbox/i.test(md2cbHtml)) {
      results.issues.push('Task lists not rendered correctly');
    }
  }

  // Check for math
  if (/\$[^$]+\$/.test(source) || /\$\$[^$]+\$\$/.test(source)) {
    if (/math-error/.test(md2cbHtml)) {
      results.issues.push('Math rendering error');
    }
    // Check if math was converted to images
    if (!/<img[^>]*>/.test(md2cbHtml) && /<span[^>]*data-math/.test(md2cbHtml)) {
      results.issues.push('Math not converted to images');
    }
  }

  // Check for links
  if (/\[.+\]\(.+\)/.test(source)) {
    if (!/<a /i.test(md2cbHtml)) {
      results.issues.push('Links not rendered');
    }
  }

  // Check for images
  if (/!\[/.test(source)) {
    if (!/<img/i.test(md2cbHtml)) {
      results.issues.push('Images not rendered');
    }
  }

  // Check for blockquotes
  if (source.includes('>')) {
    if (!/<blockquote/i.test(md2cbHtml) && browserHtml && /<blockquote/i.test(browserHtml)) {
      results.issues.push('Blockquotes not rendered');
    }
  }

  // Check for strikethrough
  if (source.includes('~~')) {
    if (!/<del|<s|<strike/i.test(md2cbHtml)) {
      results.issues.push('Strikethrough not rendered');
    }
  }

  // Compare element counts
  const browserTables = (browserHtml.match(/<table/gi) || []).length;
  const md2cbTables = (md2cbHtml.match(/<table/gi) || []).length;
  if (browserTables > 0 && md2cbTables !== browserTables) {
    results.issues.push(`Table count mismatch: browser=${browserTables}, md2cb=${md2cbTables}`);
  }

  const browserLists = (browserHtml.match(/<[uo]l/gi) || []).length;
  const md2cbLists = (md2cbHtml.match(/<[uo]l/gi) || []).length;
  if (browserLists > 0 && md2cbLists !== browserLists) {
    results.issues.push(`List count mismatch: browser=${browserLists}, md2cb=${md2cbLists}`);
  }
}

async function main() {
  // Create screenshot directory
  if (!fs.existsSync(SCREENSHOT_DIR)) {
    fs.mkdirSync(SCREENSHOT_DIR, { recursive: true });
  }

  console.log('Starting visual comparison tests...\n');

  const browser = await chromium.launch({
    headless: false,
    args: ['--disable-web-security']
  });

  const testFiles = [
    'test/demo.md',
    ...fs.readdirSync('test/edge-cases')
      .filter(f => f.endsWith('.md'))
      .sort()
      .map(f => `test/edge-cases/${f}`)
  ];

  const allResults = [];

  for (let i = 0; i < testFiles.length; i++) {
    const file = testFiles[i];
    console.log(`[${i + 1}/${testFiles.length}] Testing: ${file}`);

    const result = await testFile(browser, file, i);
    allResults.push(result);

    if (result.issues.length > 0) {
      result.issues.forEach(issue => {
        console.log(`  ⚠️  ${issue}`);
      });
    } else {
      console.log(`  ✓ No issues detected`);
    }
    console.log('');
  }

  await browser.close();

  // Print summary
  console.log('\n=== SUMMARY ===\n');

  const filesWithIssues = allResults.filter(r => r.issues.length > 0);
  if (filesWithIssues.length === 0) {
    console.log('✓ All tests passed with no issues detected!\n');
  } else {
    console.log(`Found issues in ${filesWithIssues.length} file(s):\n`);
    filesWithIssues.forEach(r => {
      console.log(`${r.file}:`);
      r.issues.forEach(issue => console.log(`  - ${issue}`));
      console.log('');
    });
  }

  // Save detailed report
  const report = {
    timestamp: new Date().toISOString(),
    totalFiles: testFiles.length,
    filesWithIssues: filesWithIssues.length,
    results: allResults.map(r => ({
      file: r.file,
      issues: r.issues,
      browserHtmlLength: r.browserHtml.length,
      md2cbHtmlLength: r.md2cbHtml.length
    }))
  };

  fs.writeFileSync('test/comparison-report.json', JSON.stringify(report, null, 2));
  console.log(`Screenshots saved to: ${SCREENSHOT_DIR}/`);
  console.log('Detailed report saved to: test/comparison-report.json');
}

main().catch(console.error);
