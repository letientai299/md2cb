// MathJax entry point for bundling
// This file is bundled into a single JS file for embedding in the Rust binary

const { mathjax } = require('mathjax-full/js/mathjax.js');
const { TeX } = require('mathjax-full/js/input/tex.js');
const { SVG } = require('mathjax-full/js/output/svg.js');
const { liteAdaptor } = require('mathjax-full/js/adaptors/liteAdaptor.js');
const { RegisterHTMLHandler } = require('mathjax-full/js/handlers/html.js');
const { AllPackages } = require('mathjax-full/js/input/tex/AllPackages.js');

// Initialize MathJax with liteAdaptor (no DOM needed)
const adaptor = liteAdaptor();
RegisterHTMLHandler(adaptor);

const tex = new TeX({ packages: AllPackages });
const svg = new SVG({ fontCache: 'none' });  // No font caching, self-contained SVG
const html = mathjax.document('', { InputJax: tex, OutputJax: svg });

// Expose global function for Rust to call
globalThis.convertLatexToSvg = function(latex, display) {
    try {
        const node = html.convert(latex, { display: display });
        let svgHtml = adaptor.outerHTML(node);

        // Extract the SVG element
        const match = svgHtml.match(/<svg[^>]*>[\s\S]*<\/svg>/);
        if (match) {
            let svgStr = match[0];

            // Ensure proper xmlns namespace
            if (!svgStr.includes('xmlns=')) {
                svgStr = svgStr.replace('<svg ', '<svg xmlns="http://www.w3.org/2000/svg" ');
            }

            // Convert currentColor to black for visibility
            svgStr = svgStr.replace(/fill="currentColor"/g, 'fill="black"');
            svgStr = svgStr.replace(/stroke="currentColor"/g, 'stroke="black"');

            return JSON.stringify({ success: true, svg: svgStr });
        }

        // Fallback: return the full output
        return JSON.stringify({ success: true, svg: svgHtml });
    } catch (e) {
        return JSON.stringify({ success: false, error: e.message || String(e) });
    }
};
