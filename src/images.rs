//! Image inlining - converts image URLs to base64 data URIs.

use base64::{engine::general_purpose::STANDARD, Engine};
use regex::Regex;
use std::collections::HashMap;
use std::fs;
use std::io::Read;
use std::path::Path;
use std::sync::LazyLock;

// Static regex pattern for matching img tags
static IMG_TAG_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r#"<img([^>]*)\ssrc="([^"]+)"([^>]*)>"#).unwrap()
});

/// Inlines all images in the HTML by converting URLs to base64 data URIs.
/// This ensures pasted content contains the actual image data.
pub fn inline_images(html: &str, base_path: Option<&Path>) -> String {
    let mut result = html.to_string();
    let mut cache: HashMap<String, String> = HashMap::new();

    // Collect all matches first
    let matches: Vec<_> = IMG_TAG_RE
        .captures_iter(html)
        .map(|cap| {
            let full = cap.get(0).unwrap();
            let before = cap.get(1).unwrap().as_str();
            let src = cap.get(2).unwrap().as_str();
            let after = cap.get(3).unwrap().as_str();
            (full.start(), full.end(), before.to_string(), src.to_string(), after.to_string())
        })
        .collect();

    // Process in reverse order to preserve indices
    for (start, end, before, src, after) in matches.into_iter().rev() {
        // Skip if already a data URI
        if src.starts_with("data:") {
            continue;
        }

        // Check cache first
        let data_uri = if let Some(cached) = cache.get(&src) {
            cached.clone()
        } else {
            let uri = fetch_and_encode(&src, base_path).unwrap_or_else(|| src.clone());
            cache.insert(src.clone(), uri.clone());
            uri
        };

        let replacement = format!(r#"<img{before} src="{data_uri}"{after}>"#);
        result.replace_range(start..end, &replacement);
    }

    result
}

/// Fetches an image and encodes it as a base64 data URI.
fn fetch_and_encode(src: &str, base_path: Option<&Path>) -> Option<String> {
    if src.starts_with("http://") || src.starts_with("https://") {
        fetch_remote_image(src)
    } else {
        fetch_local_image(src, base_path)
    }
}

/// Fetches a remote image via HTTP and encodes as data URI.
fn fetch_remote_image(url: &str) -> Option<String> {
    let response = ureq::get(url)
        .timeout(std::time::Duration::from_secs(10))
        .call()
        .ok()?;

    let content_type = response
        .header("Content-Type")
        .unwrap_or("image/png")
        .to_string();

    // Read response body
    let mut bytes = Vec::new();
    // Limit to 10MB to prevent memory exhaustion
    response
        .into_reader()
        .take(10 * 1024 * 1024)
        .read_to_end(&mut bytes)
        .ok()?;

    let encoded = STANDARD.encode(&bytes);
    Some(format!("data:{content_type};base64,{encoded}"))
}

/// Reads a local image file and encodes as data URI.
fn fetch_local_image(path: &str, base_path: Option<&Path>) -> Option<String> {
    let full_path = if let Some(base) = base_path {
        let full = base.join(path);
        // If path is absolute, we allow it (as per existing tests/behavior).
        // We only restrict relative paths to stay within the base directory.
        if Path::new(path).is_absolute() {
            full
        } else {
            // Prevent path traversal for relative paths
            let canonical_base = base.canonicalize().ok()?;
            let canonical_full = full.canonicalize().ok()?;
            
            if !canonical_full.starts_with(&canonical_base) {
                return None;
            }
            full
        }
    } else {
        Path::new(path).to_path_buf()
    };

    let bytes = fs::read(&full_path).ok()?;
    let content_type = guess_mime_type(&full_path);
    let encoded = STANDARD.encode(&bytes);

    Some(format!("data:{content_type};base64,{encoded}"))
}

/// Guesses MIME type from file extension.
fn guess_mime_type(path: &Path) -> &'static str {
    match path.extension().and_then(|e| e.to_str()) {
        Some("png") => "image/png",
        Some("jpg" | "jpeg") => "image/jpeg",
        Some("gif") => "image/gif",
        Some("webp") => "image/webp",
        Some("svg") => "image/svg+xml",
        Some("ico") => "image/x-icon",
        Some("bmp") => "image/bmp",
        _ => "image/png", // Default
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Minimal PNG (1x1 transparent pixel)
    const PNG_BYTES: [u8; 67] = [
        0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, 0x00, 0x00, 0x00, 0x0D,
        0x49, 0x48, 0x44, 0x52, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01,
        0x08, 0x06, 0x00, 0x00, 0x00, 0x1F, 0x15, 0xC4, 0x89, 0x00, 0x00, 0x00,
        0x0A, 0x49, 0x44, 0x41, 0x54, 0x78, 0x9C, 0x63, 0x00, 0x01, 0x00, 0x00,
        0x05, 0x00, 0x01, 0x0D, 0x0A, 0x2D, 0xB4, 0x00, 0x00, 0x00, 0x00, 0x49,
        0x45, 0x4E, 0x44, 0xAE, 0x42, 0x60, 0x82,
    ];

    #[test]
    fn test_skip_data_uri() {
        let html = r#"<img src="data:image/png;base64,abc123">"#;
        let result = inline_images(html, None);
        assert_eq!(result, html);
    }

    #[test]
    fn test_inline_local_image() {
        // Create a test image
        let test_dir = std::env::temp_dir().join("md2cb_test_local");
        fs::create_dir_all(&test_dir).unwrap();
        let img_path = test_dir.join("test.png");
        fs::write(&img_path, PNG_BYTES).unwrap();

        let html = r#"<img src="test.png">"#;
        let result = inline_images(html, Some(&test_dir));

        assert!(result.starts_with(r#"<img src="data:image/png;base64,"#));

        // Cleanup
        fs::remove_dir_all(&test_dir).ok();
    }

    #[test]
    fn test_inline_relative_path_in_subdir() {
        // Create test structure: base_dir/images/test.png
        let test_dir = std::env::temp_dir().join("md2cb_test_subdir");
        let images_dir = test_dir.join("images");
        fs::create_dir_all(&images_dir).unwrap();
        fs::write(images_dir.join("test.png"), PNG_BYTES).unwrap();

        // Relative path should resolve from base_dir
        let html = r#"<img src="images/test.png">"#;
        let result = inline_images(html, Some(&test_dir));

        assert!(
            result.starts_with(r#"<img src="data:image/png;base64,"#),
            "Relative path in subdir should be resolved. Got: {}",
            result
        );

        // Cleanup
        fs::remove_dir_all(&test_dir).ok();
    }

    #[test]
    fn test_inline_absolute_path() {
        // Create test image at absolute path
        let test_dir = std::env::temp_dir().join("md2cb_test_abs");
        fs::create_dir_all(&test_dir).unwrap();
        let img_path = test_dir.join("absolute.png");
        fs::write(&img_path, PNG_BYTES).unwrap();

        // Use absolute path in HTML
        let abs_path_str = img_path.to_string_lossy();
        let html = format!(r#"<img src="{}">"#, abs_path_str);

        // base_path shouldn't matter for absolute paths
        let other_dir = std::env::temp_dir().join("md2cb_test_other");
        fs::create_dir_all(&other_dir).unwrap();

        let result = inline_images(&html, Some(&other_dir));

        assert!(
            result.starts_with(r#"<img src="data:image/png;base64,"#),
            "Absolute path should work regardless of base_path. Got: {}",
            result
        );

        // Cleanup
        fs::remove_dir_all(&test_dir).ok();
        fs::remove_dir_all(&other_dir).ok();
    }
}
