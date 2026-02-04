#!/usr/bin/env node
// Converts LaTeX math to PNG base64 data URI using MathJax + canvas
// Renders at high resolution but displays at text size for crisp quality
// Usage: echo "x^2" | node math-to-svg.js [--display]

const { mathjax } = require('mathjax-full/js/mathjax.js');
const { TeX } = require('mathjax-full/js/input/tex.js');
const { SVG } = require('mathjax-full/js/output/svg.js');
const { liteAdaptor } = require('mathjax-full/js/adaptors/liteAdaptor.js');
const { RegisterHTMLHandler } = require('mathjax-full/js/handlers/html.js');
const { AllPackages } = require('mathjax-full/js/input/tex/AllPackages.js');
const { createCanvas, loadImage } = require('canvas');

const adaptor = liteAdaptor();
RegisterHTMLHandler(adaptor);

const tex = new TeX({ packages: AllPackages });
const svg = new SVG({ fontCache: 'none' });
const html = mathjax.document('', { InputJax: tex, OutputJax: svg });

const display = process.argv.includes('--display');

// Render at 4x resolution for crisp display
const RENDER_SCALE = 4;
// Display at base size (1.0) - the SVG's natural ex-based dimensions converted to pixels
const DISPLAY_SCALE = 1.0;

async function svgToPng(svgString) {
    // Parse dimensions from SVG (in ex units)
    // Try width attribute first, then min-width in style (for display math with 100% width)
    let widthMatch = svgString.match(/width="([0-9.]+)ex"/);
    if (!widthMatch) {
        widthMatch = svgString.match(/min-width:\s*([0-9.]+)ex/);
    }
    const heightMatch = svgString.match(/height="([0-9.]+)ex"/);

    // ex units: roughly 0.5em, 1em ~ 16px at normal font size
    const exToPx = 8;
    const baseWidth = widthMatch ? parseFloat(widthMatch[1]) * exToPx : 100;
    const baseHeight = heightMatch ? parseFloat(heightMatch[1]) * exToPx : 50;

    // Render at high resolution
    const renderWidth = Math.ceil(baseWidth * RENDER_SCALE);
    const renderHeight = Math.ceil(baseHeight * RENDER_SCALE);

    // Display size (what the user sees)
    const displayWidth = Math.ceil(baseWidth * DISPLAY_SCALE);
    const displayHeight = Math.ceil(baseHeight * DISPLAY_SCALE);

    // Create canvas at render resolution
    const canvas = createCanvas(renderWidth, renderHeight);
    const ctx = canvas.getContext('2d');

    // Fill with white background
    ctx.fillStyle = 'white';
    ctx.fillRect(0, 0, renderWidth, renderHeight);

    // Fix SVG dimensions for canvas rendering
    // Replace width="100%" with actual pixel dimensions
    let fixedSvg = svgString;
    if (svgString.includes('width="100%"')) {
        fixedSvg = fixedSvg.replace(/width="100%"/, `width="${renderWidth}"`);
    }
    // Ensure height is also in pixels
    fixedSvg = fixedSvg.replace(/height="[0-9.]+ex"/, `height="${renderHeight}"`);
    // If width is in ex units, convert to pixels
    fixedSvg = fixedSvg.replace(/width="[0-9.]+ex"/, `width="${renderWidth}"`);

    // Convert SVG to data URI for loading
    const svgDataUri = 'data:image/svg+xml;base64,' + Buffer.from(fixedSvg).toString('base64');

    const img = await loadImage(svgDataUri);
    ctx.drawImage(img, 0, 0, renderWidth, renderHeight);

    return {
        buffer: canvas.toBuffer('image/png'),
        displayWidth,
        displayHeight
    };
}

let input = '';
process.stdin.setEncoding('utf8');
process.stdin.on('data', chunk => input += chunk);
process.stdin.on('end', async () => {
    const latex = input.trim();
    if (!latex) {
        process.exit(0);
    }
    try {
        const node = html.convert(latex, { display });
        let svgHtml = adaptor.outerHTML(node);

        // Extract the SVG
        const match = svgHtml.match(/<svg[^>]*>[\s\S]*<\/svg>/);
        if (match) {
            let svg = match[0];
            // Ensure proper xmlns
            if (!svg.includes('xmlns=')) {
                svg = svg.replace('<svg ', '<svg xmlns="http://www.w3.org/2000/svg" ');
            }
            // Set fill color to black for visibility
            svg = svg.replace(/fill="currentColor"/g, 'fill="black"');
            svg = svg.replace(/stroke="currentColor"/g, 'stroke="black"');

            try {
                // Convert to PNG
                const { buffer, displayWidth, displayHeight } = await svgToPng(svg);
                const base64 = buffer.toString('base64');
                const dataUri = `data:image/png;base64,${base64}`;

                // Use width/height attributes to scale down the high-res image
                const style = display
                    ? 'display:block;margin:0.5em auto;'
                    : 'vertical-align:middle;';
                console.log(`<img src="${dataUri}" alt="${latex.replace(/"/g, '&quot;')}" width="${displayWidth}" height="${displayHeight}" style="${style}">`);
            } catch (e) {
                // Fallback to SVG data URI
                const base64 = Buffer.from(svg).toString('base64');
                const dataUri = `data:image/svg+xml;base64,${base64}`;
                const style = display ? 'display:block;margin:0.5em auto;' : 'vertical-align:middle;';
                console.log(`<img src="${dataUri}" alt="${latex.replace(/"/g, '&quot;')}" style="${style}">`);
            }
        } else {
            console.log(svgHtml);
        }
    } catch (e) {
        console.error('MathJax error:', e.message);
        process.exit(1);
    }
});
