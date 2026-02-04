/// GitHub-style CSS for rendered markdown.
public enum Styles {
    /// CSS string for styling markdown-body content.
    public static let css = """
    .markdown-body {
        font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Helvetica, Arial, sans-serif;
        font-size: 16px;
        line-height: 1.5;
        color: #1f2328;
    }
    .markdown-body h1, .markdown-body h2 {
        border-bottom: 1px solid #d0d7de;
        padding-bottom: 0.3em;
    }
    .markdown-body h1 { font-size: 2em; margin: 0.67em 0; }
    .markdown-body h2 { font-size: 1.5em; margin: 0.83em 0; }
    .markdown-body h3 { font-size: 1.25em; margin: 1em 0; }
    .markdown-body h4 { font-size: 1em; margin: 1.33em 0; }
    .markdown-body p { margin: 0 0 16px 0; }
    .markdown-body blockquote {
        margin: 0 0 16px 0;
        padding: 0 1em;
        color: #656d76;
        border-left: 4px solid #d0d7de;
    }
    .markdown-body pre {
        background: #f6f8fa;
        border-radius: 6px;
        padding: 16px;
        overflow: auto;
        font-size: 85%;
        line-height: 1.45;
    }
    .markdown-body code {
        font-family: SFMono-Regular, Consolas, "Liberation Mono", Menlo, monospace;
        font-size: 85%;
    }
    .markdown-body :not(pre) > code {
        background: rgba(175, 184, 193, 0.2);
        padding: 0.2em 0.4em;
        border-radius: 6px;
    }
    .markdown-body hr {
        height: 4px;
        padding: 0;
        margin: 24px 0;
        background: #d0d7de;
        border: 0;
    }
    .markdown-body ul, .markdown-body ol {
        padding-left: 2em;
        margin: 0 0 16px 0;
    }
    .markdown-body li { margin: 4px 0; }
    .markdown-body table {
        border-collapse: collapse;
        margin: 0 0 16px 0;
    }
    .markdown-body th, .markdown-body td {
        padding: 6px 13px;
        border: 1px solid #d0d7de;
    }
    .markdown-body th { font-weight: 600; }
    .markdown-body a { color: #0969da; text-decoration: none; }
    .markdown-body del { text-decoration: line-through; }
    .markdown-body img { max-width: 100%; }
    .markdown-body sub, .markdown-body sup { font-size: 75%; }
    """
}
