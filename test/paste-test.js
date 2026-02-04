const { chromium } = require('playwright');
const { execSync } = require('child_process');

async function testPaste() {
    // First, copy demo.md to clipboard using md2cb
    console.log('Copying demo.md to clipboard...');
    execSync('cat test/demo.md | ./target/release/md2cb', { stdio: 'inherit' });

    // Launch browser in headed mode to use system clipboard
    const browser = await chromium.launch({
        headless: false,  // Must be headed to access system clipboard
        slowMo: 100
    });
    const context = await browser.newContext();
    const page = await context.newPage();

    // Navigate to the editor
    console.log('Opening editor...');
    await page.goto('http://localhost:9090');
    await page.waitForTimeout(1000);

    // Find the editor area and click to focus
    const editor = await page.locator('.fr-element').first();
    await editor.click();

    // Clear existing content
    await page.keyboard.press('Meta+a');
    await page.keyboard.press('Backspace');
    await page.waitForTimeout(200);

    // Paste from system clipboard using Cmd+V
    console.log('Pasting from clipboard...');
    await page.keyboard.press('Meta+v');
    await page.waitForTimeout(1000);

    // Take screenshot
    await page.screenshot({ path: 'test/paste-result.png', fullPage: true });
    console.log('Screenshot saved to test/paste-result.png');

    // Get the pasted content
    const content = await editor.innerHTML();

    console.log('\n=== PASTED HTML (first 2000 chars) ===');
    console.log(content.substring(0, 2000));

    // Check what actually rendered
    const checks = {
        'Has any content': content.length > 100,
        'Has math images': content.includes('data:image/svg+xml') || content.includes('<img'),
        'Has math-display': content.includes('math-display'),
        'Has math-inline': content.includes('math-inline'),
        'Has math-error': content.includes('math-error'),
        'Has table': content.includes('<table') || content.includes('table'),
        'Has checkbox': content.includes('✅') || content.includes('⬜'),
    };

    console.log('\n=== CONTENT CHECKS ===');
    for (const [check, result] of Object.entries(checks)) {
        console.log(`${result ? '✓' : '✗'} ${check}`);
    }

    await browser.close();

    // Report result
    if (checks['Has math-error']) {
        console.error('\n✗ FAILED: Math errors found');
        process.exit(1);
    }
    if (!checks['Has math images']) {
        console.error('\n✗ FAILED: No math images found in pasted content');
        process.exit(1);
    }

    console.log('\n✓ Test completed - check screenshot for visual verification');
}

testPaste().catch(err => {
    console.error('Test failed:', err);
    process.exit(1);
});
