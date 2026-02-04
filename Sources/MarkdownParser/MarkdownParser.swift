import Foundation

/// A pure GFM (GitHub Flavored Markdown) to HTML converter.
public enum MarkdownParser {
    /// Converts GitHub Flavored Markdown to HTML.
    /// - Parameter markdown: The markdown string to convert.
    /// - Returns: HTML string.
    public static func convert(_ markdown: String) -> String {
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
}

// MARK: - Reference Links

extension MarkdownParser {
    /// Extracts reference-style link definitions from markdown.
    /// - Parameter text: The markdown text (modified in place to remove definitions).
    /// - Returns: Dictionary mapping reference keys to URLs.
    static func extractReferenceLinks(_ text: inout String) -> [String: String] {
        var refs: [String: String] = [:]
        guard let regex = try? NSRegularExpression(pattern: "(?m)^\\[([^\\]]+)\\]:\\s*(.+)$") else {
            return refs
        }

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
}

// MARK: - Code Blocks

extension MarkdownParser {
    /// Converts fenced code blocks (```lang ... ```) to HTML.
    static func convertCodeBlocks(_ text: String) -> String {
        guard let regex = try? NSRegularExpression(pattern: "```(\\w*)\\n([\\s\\S]*?)```") else {
            return text
        }
        let range = NSRange(text.startIndex..., in: text)
        return regex.stringByReplacingMatches(
            in: text,
            range: range,
            withTemplate: "<pre><code class=\"language-$1\">$2</code></pre>"
        )
    }
}

// MARK: - Blockquotes

extension MarkdownParser {
    /// Converts blockquote lines (> text) to HTML.
    static func convertBlockquotes(_ text: String) -> String {
        let lines = text.components(separatedBy: "\n")
        var result: [String] = []
        var inQuote = false
        var quoteContent: [String] = []

        for line in lines {
            if line.hasPrefix(">") {
                if !inQuote {
                    inQuote = true
                    quoteContent = []
                }
                let content = String(line.dropFirst()).trimmingCharacters(in: .whitespaces)
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
}

// MARK: - Headers

extension MarkdownParser {
    /// Converts ATX-style headers (# H1, ## H2, etc.) to HTML.
    static func convertHeaders(_ text: String) -> String {
        var result = text
        // Process H6 first to H1 to avoid partial matches
        for level in (1...6).reversed() {
            let prefix = String(repeating: "#", count: level)
            guard let regex = try? NSRegularExpression(pattern: "(?m)^\(prefix) (.+)$") else {
                continue
            }
            let range = NSRange(result.startIndex..., in: result)
            result = regex.stringByReplacingMatches(
                in: result,
                range: range,
                withTemplate: "<h\(level)>$1</h\(level)>"
            )
        }
        return result
    }
}

// MARK: - Horizontal Rules

extension MarkdownParser {
    /// Converts horizontal rules (---, ***, ___) to HTML.
    static func convertHorizontalRules(_ text: String) -> String {
        guard let regex = try? NSRegularExpression(pattern: "(?m)^(---+|\\*\\*\\*+|___+)$") else {
            return text
        }
        let range = NSRange(text.startIndex..., in: text)
        return regex.stringByReplacingMatches(in: text, range: range, withTemplate: "<hr>")
    }
}

// MARK: - Tables

extension MarkdownParser {
    /// Converts GFM tables to HTML with alignment support.
    static func convertTables(_ text: String) -> String {
        let lines = text.components(separatedBy: "\n")
        var result: [String] = []
        var index = 0

        while index < lines.count {
            let line = lines[index]

            // Check if this is a table header row (has |)
            if line.contains("|"), index + 1 < lines.count {
                let nextLine = lines[index + 1]
                // Check for separator row (|---|---|)
                if nextLine.contains("|"), nextLine.contains("-") {
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
                    index += 2

                    // Body rows
                    while index < lines.count, lines[index].contains("|") {
                        tableHTML += "<tr>\n"
                        let cells = parseTableRow(lines[index])
                        for (idx, cell) in cells.enumerated() {
                            let align = idx < alignments.count ? alignments[idx] : ""
                            let style = align.isEmpty ? "" : " style=\"text-align:\(align)\""
                            tableHTML += "<td\(style)>\(cell)</td>\n"
                        }
                        tableHTML += "</tr>\n"
                        index += 1
                    }

                    tableHTML += "</tbody>\n</table>"
                    result.append(tableHTML)
                    continue
                }
            }

            result.append(line)
            index += 1
        }

        return result.joined(separator: "\n")
    }

    /// Parses a table row into cell contents.
    public static func parseTableRow(_ row: String) -> [String] {
        row.split(separator: "|", omittingEmptySubsequences: false)
            .map { $0.trimmingCharacters(in: .whitespaces) }
            .filter { !$0.isEmpty }
    }

    /// Parses alignment markers from a table separator row.
    public static func parseTableAlignments(_ separator: String) -> [String] {
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
}

// MARK: - Lists

extension MarkdownParser {
    /// Converts markdown lists (ordered, unordered, task) to HTML.
    static func convertLists(_ text: String) -> String {
        let lines = text.components(separatedBy: "\n")
        var result: [String] = []
        var listStack: [(type: String, indent: Int)] = []

        for line in lines {
            let trimmed = line.trimmingCharacters(in: .whitespaces)
            let indent = line.prefix(while: { $0 == " " || $0 == "\t" }).count

            if let listItem = parseListItem(trimmed, indent: indent) {
                handleListItem(&result, &listStack, indent: indent, type: listItem.type)
                result.append("<li>\(listItem.content)</li>")
            } else {
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

    private struct ListItem {
        let type: String  // "ul" or "ol"
        let content: String
    }

    private static func parseListItem(_ line: String, indent: Int) -> ListItem? {
        // Task list item: - [ ] or - [x]
        if let match = line.range(of: "^[-*+] \\[([ xX])\\] ", options: .regularExpression) {
            let checkbox = line[match].contains("x") || line[match].contains("X")
            let content = String(line[match.upperBound...])
            let symbol = checkbox ? "☑" : "☐"
            return ListItem(type: "ul", content: "\(symbol) \(content)")
        }

        // Unordered list item: - or * or +
        if let match = line.range(of: "^[-*+] ", options: .regularExpression) {
            let content = String(line[match.upperBound...])
            return ListItem(type: "ul", content: content)
        }

        // Ordered list item: 1.
        if let match = line.range(of: "^\\d+\\. ", options: .regularExpression) {
            let content = String(line[match.upperBound...])
            return ListItem(type: "ol", content: content)
        }

        return nil
    }

    private static func handleListItem(
        _ result: inout [String],
        _ stack: inout [(type: String, indent: Int)],
        indent: Int,
        type: String
    ) {
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
}

// MARK: - Paragraphs

extension MarkdownParser {
    /// Wraps non-HTML text lines in paragraph tags.
    static func convertParagraphs(_ text: String) -> String {
        let lines = text.components(separatedBy: "\n")
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

            // Skip if it's already HTML or empty
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
}

// MARK: - Inline Elements

extension MarkdownParser {
    /// Converts inline markdown elements to HTML.
    /// - Parameters:
    ///   - text: The text to process.
    ///   - refs: Reference link definitions.
    /// - Returns: HTML with inline elements converted.
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
        result = convertReferenceLinks(result, refs: refs)

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

    private static func convertReferenceLinks(_ text: String, refs: [String: String]) -> String {
        guard let regex = try? NSRegularExpression(pattern: "\\[([^\\]]+)\\]\\[([^\\]]+)\\]") else {
            return text
        }

        var result = text
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
        return result
    }
}
