use arboard::Clipboard;
use std::error::Error;

#[cfg(all(
    unix,
    not(any(target_os = "macos", target_os = "android", target_os = "emscripten"))
))]
use arboard::SetExtLinux;

/// Copies HTML content to the clipboard.
///
/// On all platforms, this sets the HTML MIME type so rich text editors
/// can paste the formatted content.
pub fn copy_html(html: &str) -> Result<(), Box<dyn Error>> {
    let mut clipboard = Clipboard::new()?;

    // On Linux, we need to fork to keep clipboard content available after process exits
    #[cfg(all(
        unix,
        not(any(target_os = "macos", target_os = "android", target_os = "emscripten"))
    ))]
    {
        clipboard.set().wait().html(html.to_string(), None)?;
    }

    // On macOS and Windows, simple set_html works
    #[cfg(any(
        target_os = "macos",
        target_os = "windows",
        target_os = "android",
        target_os = "emscripten"
    ))]
    {
        clipboard.set_html(html, None)?;
    }

    Ok(())
}
