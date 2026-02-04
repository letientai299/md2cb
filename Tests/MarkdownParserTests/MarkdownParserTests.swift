import Testing
@testable import MarkdownParser

@Suite("MarkdownParser Tests")
struct MarkdownParserTests {

    // MARK: - Headers

    @Suite("Headers")
    struct HeaderTests {
        @Test("converts H1 headers")
        func h1() {
            let input = "# Hello World"
            let result = MarkdownParser.convert(input)
            #expect(result.contains("<h1>Hello World</h1>"))
        }

        @Test("converts H2 headers")
        func h2() {
            let input = "## Section Title"
            let result = MarkdownParser.convert(input)
            #expect(result.contains("<h2>Section Title</h2>"))
        }

        @Test("converts H3 through H6")
        func h3ToH6() {
            #expect(MarkdownParser.convert("### H3").contains("<h3>H3</h3>"))
            #expect(MarkdownParser.convert("#### H4").contains("<h4>H4</h4>"))
            #expect(MarkdownParser.convert("##### H5").contains("<h5>H5</h5>"))
            #expect(MarkdownParser.convert("###### H6").contains("<h6>H6</h6>"))
        }

        @Test("does not convert text without space after hash")
        func noSpaceAfterHash() {
            let input = "#NoSpace"
            let result = MarkdownParser.convert(input)
            #expect(!result.contains("<h1>"))
        }
    }

    // MARK: - Inline Elements

    @Suite("Inline Elements")
    struct InlineTests {
        @Test("converts bold with asterisks")
        func boldAsterisks() {
            let input = "This is **bold** text"
            let result = MarkdownParser.convert(input)
            #expect(result.contains("<strong>bold</strong>"))
        }

        @Test("converts bold with underscores")
        func boldUnderscores() {
            let input = "This is __bold__ text"
            let result = MarkdownParser.convert(input)
            #expect(result.contains("<strong>bold</strong>"))
        }

        @Test("converts italic with asterisks")
        func italicAsterisks() {
            let input = "This is *italic* text"
            let result = MarkdownParser.convert(input)
            #expect(result.contains("<em>italic</em>"))
        }

        @Test("converts italic with underscores")
        func italicUnderscores() {
            let input = "This is _italic_ text"
            let result = MarkdownParser.convert(input)
            #expect(result.contains("<em>italic</em>"))
        }

        @Test("converts strikethrough")
        func strikethrough() {
            let input = "This is ~~deleted~~ text"
            let result = MarkdownParser.convert(input)
            #expect(result.contains("<del>deleted</del>"))
        }

        @Test("converts inline code")
        func inlineCode() {
            let input = "Use `print()` function"
            let result = MarkdownParser.convert(input)
            #expect(result.contains("<code>print()</code>"))
        }
    }

    // MARK: - Links

    @Suite("Links")
    struct LinkTests {
        @Test("converts inline links")
        func inlineLink() {
            let input = "[GitHub](https://github.com)"
            let result = MarkdownParser.convert(input)
            #expect(result.contains("<a href=\"https://github.com\">GitHub</a>"))
        }

        @Test("converts auto links")
        func autoLink() {
            let input = "Visit https://example.com for more"
            let result = MarkdownParser.convert(input)
            #expect(result.contains("<a href=\"https://example.com\">https://example.com</a>"))
        }

        @Test("converts images")
        func image() {
            let input = "![Alt text](image.png)"
            let result = MarkdownParser.convert(input)
            #expect(result.contains("<img alt=\"Alt text\" src=\"image.png\""))
        }
    }

    // MARK: - Code Blocks

    @Suite("Code Blocks")
    struct CodeBlockTests {
        @Test("converts fenced code blocks")
        func fencedCodeBlock() {
            let input = """
            ```swift
            let x = 1
            ```
            """
            let result = MarkdownParser.convert(input)
            #expect(result.contains("<pre><code class=\"language-swift\">"))
            #expect(result.contains("let x = 1"))
            #expect(result.contains("</code></pre>"))
        }

        @Test("converts code blocks without language")
        func codeBlockNoLanguage() {
            let input = """
            ```
            plain code
            ```
            """
            let result = MarkdownParser.convert(input)
            #expect(result.contains("<pre><code class=\"language-\">"))
        }
    }

    // MARK: - Lists

    @Suite("Lists")
    struct ListTests {
        @Test("converts unordered lists with dash")
        func unorderedDash() {
            let input = """
            - Item 1
            - Item 2
            """
            let result = MarkdownParser.convert(input)
            #expect(result.contains("<ul>"))
            #expect(result.contains("<li>Item 1</li>"))
            #expect(result.contains("<li>Item 2</li>"))
            #expect(result.contains("</ul>"))
        }

        @Test("converts unordered lists with asterisk")
        func unorderedAsterisk() {
            let input = """
            * Item 1
            * Item 2
            """
            let result = MarkdownParser.convert(input)
            #expect(result.contains("<ul>"))
            #expect(result.contains("<li>Item 1</li>"))
        }

        @Test("converts ordered lists")
        func orderedList() {
            let input = """
            1. First
            2. Second
            """
            let result = MarkdownParser.convert(input)
            #expect(result.contains("<ol>"))
            #expect(result.contains("<li>First</li>"))
            #expect(result.contains("<li>Second</li>"))
            #expect(result.contains("</ol>"))
        }

        @Test("converts task lists")
        func taskList() {
            let input = """
            - [ ] Unchecked
            - [x] Checked
            """
            let result = MarkdownParser.convert(input)
            #expect(result.contains("☐ Unchecked"))
            #expect(result.contains("☑ Checked"))
        }
    }

    // MARK: - Blockquotes

    @Suite("Blockquotes")
    struct BlockquoteTests {
        @Test("converts single line blockquote")
        func singleLine() {
            let input = "> This is a quote"
            let result = MarkdownParser.convert(input)
            #expect(result.contains("<blockquote><p>This is a quote</p></blockquote>"))
        }

        @Test("converts multi-line blockquote")
        func multiLine() {
            let input = """
            > Line 1
            > Line 2
            """
            let result = MarkdownParser.convert(input)
            #expect(result.contains("<blockquote>"))
            #expect(result.contains("Line 1"))
            #expect(result.contains("Line 2"))
        }
    }

    // MARK: - Tables

    @Suite("Tables")
    struct TableTests {
        @Test("converts basic table")
        func basicTable() {
            let input = """
            | Header 1 | Header 2 |
            |----------|----------|
            | Cell 1   | Cell 2   |
            """
            let result = MarkdownParser.convert(input)
            #expect(result.contains("<table>"))
            #expect(result.contains("<th>Header 1</th>"))
            #expect(result.contains("<td>Cell 1</td>"))
            #expect(result.contains("</table>"))
        }

        @Test("parses table row cells")
        func parseTableRow() {
            let row = "| A | B | C |"
            let cells = MarkdownParser.parseTableRow(row)
            #expect(cells == ["A", "B", "C"])
        }

        @Test("parses table alignments")
        func parseAlignments() {
            let separator = "|:---|:---:|---:|"
            let alignments = MarkdownParser.parseTableAlignments(separator)
            #expect(alignments == ["", "center", "right"])
        }

        @Test("applies table alignment styles")
        func tableAlignment() {
            let input = """
            | Left | Center | Right |
            |:-----|:------:|------:|
            | L    | C      | R     |
            """
            let result = MarkdownParser.convert(input)
            #expect(result.contains("style=\"text-align:center\""))
            #expect(result.contains("style=\"text-align:right\""))
        }
    }

    // MARK: - Horizontal Rules

    @Suite("Horizontal Rules")
    struct HorizontalRuleTests {
        @Test("converts dashes to hr")
        func dashes() {
            let input = "---"
            let result = MarkdownParser.convert(input)
            #expect(result.contains("<hr>"))
        }

        @Test("converts asterisks to hr")
        func asterisks() {
            let input = "***"
            let result = MarkdownParser.convert(input)
            #expect(result.contains("<hr>"))
        }

        @Test("converts underscores to hr")
        func underscores() {
            let input = "___"
            let result = MarkdownParser.convert(input)
            #expect(result.contains("<hr>"))
        }
    }

    // MARK: - Paragraphs

    @Suite("Paragraphs")
    struct ParagraphTests {
        @Test("wraps plain text in paragraphs")
        func plainText() {
            let input = "Hello world"
            let result = MarkdownParser.convert(input)
            #expect(result.contains("<p>Hello world</p>"))
        }

        @Test("does not double-wrap HTML")
        func noDoubleWrap() {
            let input = "<div>Already HTML</div>"
            let result = MarkdownParser.convert(input)
            #expect(!result.contains("<p><div>"))
        }
    }

    // MARK: - Integration

    @Suite("Integration")
    struct IntegrationTests {
        @Test("converts complex document")
        func complexDocument() {
            let input = """
            # Title

            This is **bold** and *italic* text.

            - List item 1
            - List item 2

            ```swift
            let x = 1
            ```
            """
            let result = MarkdownParser.convert(input)
            #expect(result.contains("<h1>Title</h1>"))
            #expect(result.contains("<strong>bold</strong>"))
            #expect(result.contains("<em>italic</em>"))
            #expect(result.contains("<ul>"))
            #expect(result.contains("<pre><code"))
        }
    }
}
