# Roadmap

## Cross-platform & Rust Feasibility

- [x] Prototype a Rust core (parser + HTML generation).
- [ ] Evaluate per-OS rendering/clipboard stacks: macOS WebKit, Windows
      WebView2, Linux WebKitGTK or headless Chromium, and verify HTML/RTF
      clipboard parity.
- [ ] Define a platform abstraction layer (render + clipboard)

## Later

- [ ] Image embedding: inline remote images as Base64 for offline pastes.
- [ ] Math support (MathJax/KaTeX).
- [ ] Mermaid diagram rendering.
- [ ] Custom CSS via `--css <path>`.
