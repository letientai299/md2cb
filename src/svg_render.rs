//! SVG to PNG rendering using resvg.
//!
//! Converts SVG strings to PNG images with high-resolution rendering
//! for crisp output when displayed at smaller sizes.

use base64::{engine::general_purpose::STANDARD, Engine};
use resvg::tiny_skia::{Pixmap, Transform};
use resvg::usvg::{fontdb, Options, Tree};
use std::sync::{Arc, OnceLock};

/// Render scale factor for crisp output (4x like the original Node.js implementation)
const RENDER_SCALE: f32 = 4.0;

/// Global font database - loaded once and reused
static FONT_DB: OnceLock<Arc<fontdb::Database>> = OnceLock::new();

/// Get or initialize the font database with system fonts
fn get_font_db() -> Arc<fontdb::Database> {
    FONT_DB
        .get_or_init(|| {
            let mut db = fontdb::Database::new();
            db.load_system_fonts();
            Arc::new(db)
        })
        .clone()
}

/// Result of SVG to PNG conversion
pub struct SvgRenderResult {
    /// Base64-encoded PNG data
    pub png_base64: String,
    /// Display width in pixels (the size it should be shown at)
    pub display_width: u32,
    /// Display height in pixels
    pub display_height: u32,
}

/// Renders an SVG string to PNG and returns base64-encoded result.
///
/// The SVG is rendered at 4x resolution for crispness, but the returned
/// display dimensions are the original size.
pub fn render_svg_to_png(svg_content: &str) -> Result<SvgRenderResult, String> {
    // Parse SVG with font database for text rendering
    let mut opts = Options::default();
    opts.fontdb = get_font_db();
    let tree = Tree::from_str(svg_content, &opts)
        .map_err(|e| format!("SVG parse error: {}", e))?;

    // Get original size from the SVG
    let size = tree.size();
    let base_width = size.width();
    let base_height = size.height();

    // Calculate render dimensions (scaled up for crispness)
    let render_width = (base_width * RENDER_SCALE).ceil() as u32;
    let render_height = (base_height * RENDER_SCALE).ceil() as u32;

    // Display dimensions (what the user sees)
    let display_width = base_width.ceil() as u32;
    let display_height = base_height.ceil() as u32;

    // Create pixmap for rendering
    let mut pixmap = Pixmap::new(render_width, render_height)
        .ok_or("Failed to create pixmap - dimensions may be too large or zero")?;

    // Fill with white background (matching the Node.js implementation)
    pixmap.fill(resvg::tiny_skia::Color::WHITE);

    // Render with scale transform
    let transform = Transform::from_scale(RENDER_SCALE, RENDER_SCALE);
    resvg::render(&tree, transform, &mut pixmap.as_mut());

    // Encode to PNG
    let png_data = pixmap
        .encode_png()
        .map_err(|e| format!("PNG encode error: {}", e))?;

    // Base64 encode
    let png_base64 = STANDARD.encode(&png_data);

    Ok(SvgRenderResult {
        png_base64,
        display_width,
        display_height,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_svg_render() {
        let svg = r#"<svg xmlns="http://www.w3.org/2000/svg" width="100" height="50">
            <rect width="100" height="50" fill="red"/>
        </svg>"#;
        let result = render_svg_to_png(svg).unwrap();
        assert!(!result.png_base64.is_empty());
        assert_eq!(result.display_width, 100);
        assert_eq!(result.display_height, 50);
    }

    #[test]
    fn test_svg_with_viewbox() {
        let svg = r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 200 100" width="200" height="100">
            <circle cx="100" cy="50" r="40" fill="blue"/>
        </svg>"#;
        let result = render_svg_to_png(svg).unwrap();
        assert!(!result.png_base64.is_empty());
    }

    #[test]
    fn test_invalid_svg() {
        let svg = "not valid svg";
        let result = render_svg_to_png(svg);
        assert!(result.is_err());
    }
}
