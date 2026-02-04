import Cocoa
import MarkdownParser
import WebKit

@main
struct MD2CB {
    static func main() {
        // Read markdown from stdin efficiently
        let markdown = readStdin()

        // Convert to HTML
        let html = MarkdownParser.convert(markdown)

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

    /// Reads all input from stdin efficiently.
    private static func readStdin() -> String {
        var lines: [String] = []
        while let line = readLine(strippingNewline: false) {
            lines.append(line)
        }
        return lines.joined()
    }
}

final class WebViewDelegate: NSObject, WKNavigationDelegate, @unchecked Sendable {
    var isDone = false

    // swiftlint:disable:next implicitly_unwrapped_optional
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

        webView.evaluateJavaScript(js) { [weak self] _, error in
            if let error {
                fputs("Error copying: \(error.localizedDescription)\n", stderr)
            } else {
                print("Copied to clipboard")
            }
            self?.isDone = true
        }
    }
}
