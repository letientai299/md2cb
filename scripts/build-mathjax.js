#!/usr/bin/env node
// Bundle MathJax into a single JS file for embedding in Rust binary
// Usage: node scripts/build-mathjax.js

const esbuild = require('esbuild');
const path = require('path');

const outfile = path.join(__dirname, '..', 'assets', 'mathjax-bundle.js');

esbuild.build({
    entryPoints: [path.join(__dirname, 'mathjax-entry.js')],
    bundle: true,
    minify: true,
    format: 'iife',
    outfile: outfile,
    platform: 'browser',  // Browser platform avoids Node.js requires
    target: 'es2020',
    // Define globals to avoid dynamic requires
    define: {
        'process.env.NODE_ENV': '"production"',
        'PACKAGE_VERSION': '"3.2.1"',  // MathJax version
    },
    // Log bundle size
    metafile: true,
}).then(result => {
    // Calculate and display bundle size
    const outputs = result.metafile.outputs;
    for (const [file, info] of Object.entries(outputs)) {
        const sizeKB = (info.bytes / 1024).toFixed(1);
        console.log(`Created ${file} (${sizeKB} KB)`);
    }
}).catch((error) => {
    console.error('Build failed:', error);
    process.exit(1);
});
