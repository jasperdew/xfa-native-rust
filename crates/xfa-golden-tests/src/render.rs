//! Simple layout-to-image renderer for golden tests.
//!
//! Renders layout output as a rasterized image for visual comparison.
//! This is a lightweight renderer for testing purposes; the production
//! renderer will use PDFium (Epic 4.4).

use image::{DynamicImage, Rgba, RgbaImage};
use xfa_layout_engine::layout::{LayoutContent, LayoutDom, LayoutNode, LayoutPage};

/// Render scale: points to pixels.
const SCALE: f64 = 1.0;

/// Render a full layout DOM to a vector of page images.
pub fn render_layout(layout: &LayoutDom) -> Vec<DynamicImage> {
    layout.pages.iter().map(render_page).collect()
}

/// Render a single layout page to an image.
fn render_page(page: &LayoutPage) -> DynamicImage {
    let w = (page.width * SCALE) as u32;
    let h = (page.height * SCALE) as u32;
    let mut img = RgbaImage::from_pixel(w, h, Rgba([255, 255, 255, 255]));

    for node in &page.nodes {
        render_node(&mut img, node, 0);
    }

    DynamicImage::ImageRgba8(img)
}

/// Render a layout node and its children recursively.
fn render_node(img: &mut RgbaImage, node: &LayoutNode, depth: usize) {
    let x = (node.rect.x * SCALE) as i32;
    let y = (node.rect.y * SCALE) as i32;
    let w = (node.rect.width * SCALE) as i32;
    let h = (node.rect.height * SCALE) as i32;

    // Choose color based on content type and depth
    let border_color = match &node.content {
        LayoutContent::Field { .. } | LayoutContent::WrappedText { .. } => {
            Rgba([0, 0, 200, 255]) // Blue for fields
        }
        LayoutContent::Text(_) => Rgba([0, 150, 0, 255]), // Green for draw text
        LayoutContent::None => {
            // Gray shades for containers, darker at deeper levels
            let gray = (200 - (depth as u8).min(5) * 20).max(100);
            Rgba([gray, gray, gray, 255])
        }
    };

    // Fill with light version of the border color
    let fill_color = Rgba([
        border_color[0].saturating_add(200).min(250),
        border_color[1].saturating_add(200).min(250),
        border_color[2].saturating_add(200).min(250),
        255,
    ]);

    // Draw filled rectangle
    draw_filled_rect(img, x, y, w, h, fill_color);

    // Draw border
    draw_rect(img, x, y, w, h, border_color);

    // Render children
    for child in &node.children {
        render_node(img, child, depth + 1);
    }
}

/// Draw a filled rectangle on the image.
fn draw_filled_rect(img: &mut RgbaImage, x: i32, y: i32, w: i32, h: i32, color: Rgba<u8>) {
    let (iw, ih) = (img.width() as i32, img.height() as i32);
    for py in y.max(0)..(y + h).min(ih) {
        for px in x.max(0)..(x + w).min(iw) {
            img.put_pixel(px as u32, py as u32, color);
        }
    }
}

/// Draw a rectangle border on the image.
fn draw_rect(img: &mut RgbaImage, x: i32, y: i32, w: i32, h: i32, color: Rgba<u8>) {
    let (iw, ih) = (img.width() as i32, img.height() as i32);

    // Top and bottom edges
    for px in x.max(0)..(x + w).min(iw) {
        if y >= 0 && y < ih {
            img.put_pixel(px as u32, y as u32, color);
        }
        let by = y + h - 1;
        if by >= 0 && by < ih {
            img.put_pixel(px as u32, by as u32, color);
        }
    }

    // Left and right edges
    for py in y.max(0)..(y + h).min(ih) {
        if x >= 0 && x < iw {
            img.put_pixel(x as u32, py as u32, color);
        }
        let rx = x + w - 1;
        if rx >= 0 && rx < iw {
            img.put_pixel(rx as u32, py as u32, color);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn render_empty_page() {
        let layout = LayoutDom {
            pages: vec![LayoutPage {
                width: 100.0,
                height: 100.0,
                nodes: vec![],
            }],
        };

        let images = render_layout(&layout);
        assert_eq!(images.len(), 1);
        assert_eq!(images[0].width(), 100);
        assert_eq!(images[0].height(), 100);
    }

    #[test]
    fn render_page_with_nodes() {
        use xfa_layout_engine::form::FormNodeId;
        use xfa_layout_engine::types::Rect;

        let layout = LayoutDom {
            pages: vec![LayoutPage {
                width: 200.0,
                height: 200.0,
                nodes: vec![LayoutNode {
                    form_node: FormNodeId(0),
                    rect: Rect::new(10.0, 10.0, 80.0, 30.0),
                    name: "Field1".to_string(),
                    content: LayoutContent::Field {
                        value: "Hello".to_string(),
                    },
                    children: vec![],
                }],
            }],
        };

        let images = render_layout(&layout);
        assert_eq!(images.len(), 1);

        // Check that the field area has non-white pixels (blue border)
        let img = &images[0];
        let pixel = img.as_rgba8().unwrap().get_pixel(10, 10);
        assert_ne!(*pixel, Rgba([255, 255, 255, 255]), "Border should not be white");
    }
}
