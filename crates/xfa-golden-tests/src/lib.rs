pub mod render;

use image::{DynamicImage, GenericImageView, Rgba};
use std::path::Path;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum GoldenTestError {
    #[error("Image load error: {0}")]
    ImageError(#[from] image::ImageError),

    #[error("Image dimensions mismatch: actual {actual_w}x{actual_h}, expected {expected_w}x{expected_h}")]
    DimensionMismatch {
        actual_w: u32,
        actual_h: u32,
        expected_w: u32,
        expected_h: u32,
    },

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}

#[derive(Debug)]
pub struct GoldenTestResult {
    pub total_pixels: u64,
    pub differing_pixels: u64,
    pub max_channel_diff: u8,
    pub diff_percentage: f64,
    pub passed: bool,
}

/// Compare two images pixel-by-pixel.
///
/// `threshold` is the maximum allowed percentage of differing pixels (0.0 - 100.0).
/// `channel_tolerance` is the per-channel difference threshold (0-255) below which
/// pixels are considered identical.
pub fn compare_images(
    actual: &DynamicImage,
    expected: &DynamicImage,
    threshold: f64,
    channel_tolerance: u8,
) -> Result<GoldenTestResult, GoldenTestError> {
    let (aw, ah) = actual.dimensions();
    let (ew, eh) = expected.dimensions();

    if aw != ew || ah != eh {
        return Err(GoldenTestError::DimensionMismatch {
            actual_w: aw,
            actual_h: ah,
            expected_w: ew,
            expected_h: eh,
        });
    }

    let total_pixels = (aw as u64) * (ah as u64);
    let mut differing_pixels = 0u64;
    let mut max_channel_diff = 0u8;

    for y in 0..ah {
        for x in 0..aw {
            let Rgba(a) = actual.get_pixel(x, y);
            let Rgba(e) = expected.get_pixel(x, y);

            let mut pixel_differs = false;
            for i in 0..4 {
                let diff = a[i].abs_diff(e[i]);
                if diff > max_channel_diff {
                    max_channel_diff = diff;
                }
                if diff > channel_tolerance {
                    pixel_differs = true;
                }
            }

            if pixel_differs {
                differing_pixels += 1;
            }
        }
    }

    let diff_percentage = if total_pixels > 0 {
        (differing_pixels as f64 / total_pixels as f64) * 100.0
    } else {
        0.0
    };

    Ok(GoldenTestResult {
        total_pixels,
        differing_pixels,
        max_channel_diff,
        diff_percentage,
        passed: diff_percentage <= threshold,
    })
}

/// Compare two image files and optionally generate a diff image.
pub fn compare_golden_files(
    actual_path: &Path,
    expected_path: &Path,
    diff_output_path: Option<&Path>,
    threshold: f64,
    channel_tolerance: u8,
) -> Result<GoldenTestResult, GoldenTestError> {
    let actual = image::open(actual_path)?;
    let expected = image::open(expected_path)?;

    let result = compare_images(&actual, &expected, threshold, channel_tolerance)?;

    if let Some(diff_path) = diff_output_path {
        let diff_img = generate_diff_image(&actual, &expected, channel_tolerance);
        diff_img.save(diff_path)?;
    }

    Ok(result)
}

/// Generate a diff image highlighting pixel differences in red.
fn generate_diff_image(
    actual: &DynamicImage,
    expected: &DynamicImage,
    channel_tolerance: u8,
) -> DynamicImage {
    let (w, h) = actual.dimensions();
    let mut diff = image::RgbaImage::new(w, h);

    for y in 0..h {
        for x in 0..w {
            let Rgba(a) = actual.get_pixel(x, y);
            let Rgba(e) = expected.get_pixel(x, y);

            let differs = (0..4).any(|i| a[i].abs_diff(e[i]) > channel_tolerance);

            if differs {
                // Red highlight for differing pixels
                diff.put_pixel(x, y, Rgba([255, 0, 0, 255]));
            } else {
                // Dimmed version of actual for context
                diff.put_pixel(x, y, Rgba([a[0] / 3, a[1] / 3, a[2] / 3, 255]));
            }
        }
    }

    DynamicImage::ImageRgba8(diff)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn identical_images_pass() {
        let img = DynamicImage::new_rgba8(10, 10);
        let result = compare_images(&img, &img, 0.0, 0).unwrap();
        assert!(result.passed);
        assert_eq!(result.differing_pixels, 0);
        assert_eq!(result.diff_percentage, 0.0);
    }

    #[test]
    fn different_images_detected() {
        let mut img_a = image::RgbaImage::new(2, 2);
        img_a.put_pixel(0, 0, Rgba([255, 0, 0, 255]));

        let img_b = image::RgbaImage::new(2, 2); // all black/transparent

        let a = DynamicImage::ImageRgba8(img_a);
        let b = DynamicImage::ImageRgba8(img_b);

        let result = compare_images(&a, &b, 0.0, 0).unwrap();
        assert!(!result.passed);
        assert_eq!(result.differing_pixels, 1);
    }

    #[test]
    fn channel_tolerance_works() {
        let mut img_a = image::RgbaImage::new(1, 1);
        img_a.put_pixel(0, 0, Rgba([100, 100, 100, 255]));

        let mut img_b = image::RgbaImage::new(1, 1);
        img_b.put_pixel(0, 0, Rgba([105, 105, 105, 255]));

        let a = DynamicImage::ImageRgba8(img_a);
        let b = DynamicImage::ImageRgba8(img_b);

        // With tolerance of 5, should pass
        let result = compare_images(&a, &b, 0.0, 5).unwrap();
        assert!(result.passed);

        // With tolerance of 4, should fail
        let result = compare_images(&a, &b, 0.0, 4).unwrap();
        assert!(!result.passed);
    }

    #[test]
    fn dimension_mismatch_error() {
        let a = DynamicImage::new_rgba8(10, 10);
        let b = DynamicImage::new_rgba8(20, 20);
        let result = compare_images(&a, &b, 0.0, 0);
        assert!(result.is_err());
    }
}
