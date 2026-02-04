import Cocoa
import WebKit

@main
struct MD2CB {
    static func main() {
        // Read markdown from stdin
        var markdown = ""
        while let line = readLine(strippingNewline: false) {
            markdown += line
        }

        // Convert to HTML
        let html = MarkdownToHTML.convert(markdown)

        // Use WebKit to render and copy (like a browser)
        let app = NSApplication.shared
        app.setActivationPolicy(.prohibited)

        let webView = WKWebView(frame: NSRect(x: 0, y: 0, width: 800, height: 600))
        let delegate = WebViewDelegate()
        webView.navigationDelegate = delegate

        // Load HTML with GitHub-style CSS
        let fullHTML = """
        <!DOCTYPE html>
        <html>
        <head>
        <meta charset="utf-8">
        <style>\(Styles.css)</style>
        </head>
        <body class="markdown-body">\(html)</body>
        </html>
        """

        webView.loadHTMLString(fullHTML, baseURL: nil)

        // Run until copy is complete
        let runLoop = RunLoop.current
        while !delegate.isDone {
            runLoop.run(mode: .default, before: Date(timeIntervalSinceNow: 0.1))
        }
    }
}

class WebViewDelegate: NSObject, WKNavigationDelegate {
    var isDone = false

    func webView(_ webView: WKWebView, didFinish navigation: WKNavigation!) {
        // Select all and copy via JavaScript
        let js = """
        (function() {
            const range = document.createRange();
            range.selectNodeContents(document.body);
            const selection = window.getSelection();
            selection.removeAllRanges();
            selection.addRange(range);
            document.execCommand('copy');
            return 'done';
        })()
        """

        webView.evaluateJavaScript(js) { result, error in
            if error != nil {
                fputs("Error copying: \(error!.localizedDescription)\n", stderr)
            } else {
                print("Copied to clipboard")
            }
            self.isDone = true
        }
    }
}

// MARK: - Markdown to HTML Converter

enum MarkdownToHTML {
    static func convert(_ markdown: String) -> String {
        var html = markdown

        // Extract reference links first
        let refs = extractReferenceLinks(&html)

        // Process blocks first
        html = convertCodeBlocks(html)
        html = convertBlockquotes(html)
        html = convertHeaders(html)
        html = convertHorizontalRules(html)
        html = convertTables(html)
        html = convertLists(html)
        html = convertParagraphs(html)

        // Then inline elements
        html = convertInlineElements(html, refs: refs)

        return html
    }

    static func extractReferenceLinks(_ text: inout String) -> [String: String] {
        var refs: [String: String] = [:]
        let pattern = "(?m)^\\[([^\\]]+)\\]:\\s*(.+)$"
        let regex = try! NSRegularExpression(pattern: pattern)
        let range = NSRange(text.startIndex..., in: text)

        let matches = regex.matches(in: text, range: range)
        for match in matches.reversed() {
            if let keyRange = Range(match.range(at: 1), in: text),
               let urlRange = Range(match.range(at: 2), in: text) {
                let key = String(text[keyRange]).lowercased()
                let url = String(text[urlRange]).trimmingCharacters(in: .whitespaces)
                refs[key] = url
            }
            if let fullRange = Range(match.range, in: text) {
                text.removeSubrange(fullRange)
            }
        }
        return refs
    }

    static func convertCodeBlocks(_ text: String) -> String {
        // Fenced code blocks ```lang ... ```
        let pattern = "```(\\w*)\\n([\\s\\S]*?)```"
        let regex = try! NSRegularExpression(pattern: pattern)
        let range = NSRange(text.startIndex..., in: text)

        return regex.stringByReplacingMatches(in: text, range: range, withTemplate: """
        <pre><code class="language-$1">$2</code></pre>
        """)
    }

    static func convertBlockquotes(_ text: String) -> String {
        var lines = text.components(separatedBy: "\n")
        var result: [String] = []
        var inQuote = false
        var quoteContent: [String] = []

        for line in lines {
            if line.hasPrefix(">") {
                if !inQuote {
                    inQuote = true
                    quoteContent = []
                }
                let content = String(line.dropFirst().trimmingCharacters(in: .whitespaces))
                quoteContent.append(content)
            } else {
                if inQuote {
                    result.append("<blockquote><p>\(quoteContent.joined(separator: " "))</p></blockquote>")
                    inQuote = false
                }
                result.append(line)
            }
        }
        if inQuote {
            result.append("<blockquote><p>\(quoteContent.joined(separator: " "))</p></blockquote>")
        }
        return result.joined(separator: "\n")
    }

    static func convertHeaders(_ text: String) -> String {
        var result = text
        // H1-H6
        for level in (1...6).reversed() {
            let prefix = String(repeating: "#", count: level)
            let pattern = "(?m)^\(prefix) (.+)$"
            let regex = try! NSRegularExpression(pattern: pattern)
            let range = NSRange(result.startIndex..., in: result)
            result = regex.stringByReplacingMatches(in: result, range: range, withTemplate: "<h\(level)>$1</h\(level)>")
        }
        return result
    }

    static func convertHorizontalRules(_ text: String) -> String {
        let pattern = "(?m)^(---+|\\*\\*\\*+|___+)$"
        let regex = try! NSRegularExpression(pattern: pattern)
        let range = NSRange(text.startIndex..., in: text)
        return regex.stringByReplacingMatches(in: text, range: range, withTemplate: "<hr>")
    }

    static func convertTables(_ text: String) -> String {
        let lines = text.components(separatedBy: "\n")
        var result: [String] = []
        var i = 0

        while i < lines.count {
            let line = lines[i]

            // Check if this is a table header row (has |)
            if line.contains("|") && i + 1 < lines.count {
                let nextLine = lines[i + 1]
                // Check for separator row (|---|---|)
                if nextLine.contains("|") && nextLine.contains("-") {
                    // Parse table
                    var tableHTML = "<table>\n<thead>\n<tr>\n"

                    // Parse alignments from separator
                    let alignments = parseTableAlignments(nextLine)

                    // Header row
                    let headerCells = parseTableRow(line)
                    for (idx, cell) in headerCells.enumerated() {
                        let align = idx < alignments.count ? alignments[idx] : ""
                        let style = align.isEmpty ? "" : " style=\"text-align:\(align)\""
                        tableHTML += "<th\(style)>\(cell)</th>\n"
                    }
                    tableHTML += "</tr>\n</thead>\n<tbody>\n"

                    // Skip header and separator
                    i += 2

                    // Body rows
                    while i < lines.count && lines[i].contains("|") {
                        tableHTML += "<tr>\n"
                        let cells = parseTableRow(lines[i])
                        for (idx, cell) in cells.enumerated() {
                            let align = idx < alignments.count ? alignments[idx] : ""
                            let style = align.isEmpty ? "" : " style=\"text-align:\(align)\""
                            tableHTML += "<td\(style)>\(cell)</td>\n"
                        }
                        tableHTML += "</tr>\n"
                        i += 1
                    }

                    tableHTML += "</tbody>\n</table>"
                    result.append(tableHTML)
                    continue
                }
            }

            result.append(line)
            i += 1
        }

        return result.joined(separator: "\n")
    }

    static func parseTableRow(_ row: String) -> [String] {
        row.split(separator: "|", omittingEmptySubsequences: false)
            .map { $0.trimmingCharacters(in: .whitespaces) }
            .filter { !$0.isEmpty }
    }

    static func parseTableAlignments(_ separator: String) -> [String] {
        separator.split(separator: "|")
            .map { $0.trimmingCharacters(in: .whitespaces) }
            .filter { !$0.isEmpty }
            .map { cell -> String in
                let left = cell.hasPrefix(":")
                let right = cell.hasSuffix(":")
                if left && right { return "center" }
                if right { return "right" }
                return ""
            }
    }

    static func convertLists(_ text: String) -> String {
        let lines = text.components(separatedBy: "\n")
        var result: [String] = []
        var listStack: [(type: String, indent: Int)] = []

        for line in lines {
            let trimmed = line.trimmingCharacters(in: .whitespaces)
            let indent = line.prefix(while: { $0 == " " || $0 == "\t" }).count

            // Task list item
            if let match = trimmed.range(of: "^[-*+] \\[([ xX])\\] ", options: .regularExpression) {
                let checkbox = trimmed[match].contains("x") || trimmed[match].contains("X")
                let content = String(trimmed[match.upperBound...])
                let symbol = checkbox ? "☑" : "☐"

                handleListItem(&result, &listStack, indent: indent, type: "ul")
                result.append("<li>\(symbol) \(content)</li>")
            }
            // Unordered list item
            else if let match = trimmed.range(of: "^[-*+] ", options: .regularExpression) {
                let content = String(trimmed[match.upperBound...])

                handleListItem(&result, &listStack, indent: indent, type: "ul")
                result.append("<li>\(content)</li>")
            }
            // Ordered list item
            else if let match = trimmed.range(of: "^\\d+\\. ", options: .regularExpression) {
                let content = String(trimmed[match.upperBound...])

                handleListItem(&result, &listStack, indent: indent, type: "ol")
                result.append("<li>\(content)</li>")
            }
            // Not a list item
            else {
                // Close all open lists
                while let last = listStack.popLast() {
                    result.append("</\(last.type)>")
                }
                result.append(line)
            }
        }

        // Close remaining lists
        while let last = listStack.popLast() {
            result.append("</\(last.type)>")
        }

        return result.joined(separator: "\n")
    }

    static func handleListItem(_ result: inout [String], _ stack: inout [(type: String, indent: Int)], indent: Int, type: String) {
        // Close lists that are deeper than current indent
        while let last = stack.last, last.indent > indent {
            stack.removeLast()
            result.append("</\(last.type)>")
        }

        // If we're at a deeper indent, start a new nested list
        if let last = stack.last {
            if indent > last.indent {
                result.append("<\(type)>")
                stack.append((type, indent))
            } else if last.type != type {
                // Same indent but different list type
                result.append("</\(last.type)>")
                stack.removeLast()
                result.append("<\(type)>")
                stack.append((type, indent))
            }
        } else {
            // No list open yet
            result.append("<\(type)>")
            stack.append((type, indent))
        }
    }

    static func convertParagraphs(_ text: String) -> String {
        var lines = text.components(separatedBy: "\n")
        var result: [String] = []
        var paragraph: [String] = []

        func flushParagraph() {
            if !paragraph.isEmpty {
                let content = paragraph.joined(separator: " ")
                if !content.trimmingCharacters(in: .whitespaces).isEmpty {
                    result.append("<p>\(content)</p>")
                }
                paragraph = []
            }
        }

        for line in lines {
            let trimmed = line.trimmingCharacters(in: .whitespaces)

            // Skip if it's already HTML
            if trimmed.hasPrefix("<") || trimmed.isEmpty {
                flushParagraph()
                result.append(line)
            } else {
                paragraph.append(trimmed)
            }
        }
        flushParagraph()

        return result.joined(separator: "\n")
    }

    static func convertInlineElements(_ text: String, refs: [String: String]) -> String {
        var result = text

        // Images ![alt](src "title")
        result = result.replacingOccurrences(
            of: "!\\[([^\\]]*)\\]\\(([^\\s\\)]+)(?:\\s+\"([^\"]+)\")?\\)",
            with: "<img alt=\"$1\" src=\"$2\" title=\"$3\">",
            options: .regularExpression
        )

        // Links [text](url)
        result = result.replacingOccurrences(
            of: "\\[([^\\]]+)\\]\\(([^\\)]+)\\)",
            with: "<a href=\"$2\">$1</a>",
            options: .regularExpression
        )

        // Reference links [text][ref]
        let refLinkPattern = "\\[([^\\]]+)\\]\\[([^\\]]+)\\]"
        if let regex = try? NSRegularExpression(pattern: refLinkPattern) {
            let range = NSRange(result.startIndex..., in: result)
            let matches = regex.matches(in: result, range: range)
            for match in matches.reversed() {
                if let fullRange = Range(match.range, in: result),
                   let textRange = Range(match.range(at: 1), in: result),
                   let refRange = Range(match.range(at: 2), in: result) {
                    let linkText = String(result[textRange])
                    let refKey = String(result[refRange]).lowercased()
                    if let url = refs[refKey] {
                        result.replaceSubrange(fullRange, with: "<a href=\"\(url)\">\(linkText)</a>")
                    }
                }
            }
        }

        // Auto links (URLs)
        result = result.replacingOccurrences(
            of: "(?<![\"=])\\b(https?://[^\\s<>]+)",
            with: "<a href=\"$1\">$1</a>",
            options: .regularExpression
        )

        // Bold **text** or __text__
        result = result.replacingOccurrences(
            of: "\\*\\*([^\\*]+)\\*\\*",
            with: "<strong>$1</strong>",
            options: .regularExpression
        )
        result = result.replacingOccurrences(
            of: "__([^_]+)__",
            with: "<strong>$1</strong>",
            options: .regularExpression
        )

        // Italic *text* or _text_
        result = result.replacingOccurrences(
            of: "(?<![\\*\\w])\\*([^\\*]+)\\*(?![\\*\\w])",
            with: "<em>$1</em>",
            options: .regularExpression
        )
        result = result.replacingOccurrences(
            of: "(?<![_\\w])_([^_]+)_(?![_\\w])",
            with: "<em>$1</em>",
            options: .regularExpression
        )

        // Strikethrough ~~text~~
        result = result.replacingOccurrences(
            of: "~~([^~]+)~~",
            with: "<del>$1</del>",
            options: .regularExpression
        )

        // Inline code `code`
        result = result.replacingOccurrences(
            of: "`([^`]+)`",
            with: "<code>$1</code>",
            options: .regularExpression
        )

        return result
    }
}

// MARK: - GitHub-style CSS

enum Styles {
    static let css = """
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
