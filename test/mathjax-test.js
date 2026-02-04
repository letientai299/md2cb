const { mathjax } = require('mathjax-full/js/mathjax.js');
const { TeX } = require('mathjax-full/js/input/tex.js');
const { SVG } = require('mathjax-full/js/output/svg.js');
const { liteAdaptor } = require('mathjax-full/js/adaptors/liteAdaptor.js');
const { RegisterHTMLHandler } = require('mathjax-full/js/handlers/html.js');
const { AllPackages } = require('mathjax-full/js/input/tex/AllPackages.js');

// Create adaptor and register handler
const adaptor = liteAdaptor();
RegisterHTMLHandler(adaptor);

// Create input and output jax
const tex = new TeX({ packages: AllPackages });
const svg = new SVG({ fontCache: 'none' });

// Create MathJax document
const html = mathjax.document('', { InputJax: tex, OutputJax: svg });

function texToSvg(latex, display = false) {
    const node = html.convert(latex, { display });
    return adaptor.outerHTML(node);
}

// Test
console.log('=== Simple inline math: a^2 ===');
console.log(texToSvg('a^2', false));
console.log('\n=== Fraction: \\frac{1}{2} ===');
console.log(texToSvg('\\frac{1}{2}', false));
console.log('\n=== Display math with split ===');
console.log(texToSvg('\\begin{split}p_n &= 1-\\frac{1}{2^r} \\\\ q_n &= \\frac{1}{2^r}\\end{split}', true));
