//! Image decoding from raw bytes
//!
//! Supports all image formats provided by the `image` crate including:
//! JPEG, PNG, GIF, BMP, TIFF, ICO, WebP, and more.

use crate::utils::error::CbxError;
use image::{DynamicImage, ImageReader};
use std::io::Cursor;

type Result<T> = std::result::Result<T, CbxError>;

/// Decode image from raw bytes
///
/// This function attempts to automatically detect the image format and decode it.
/// It supports all formats enabled in the `image` crate dependency.
///
/// # Arguments
/// * `data` - Raw image file bytes
///
/// # Returns
/// * `Ok(DynamicImage)` - Successfully decoded image
/// * `Err(CbxError::Image)` - Failed to decode (invalid format or corrupt data)
///
/// # Examples
/// ```no_run
/// let jpeg_data = std::fs::read("image.jpg")?;
/// let img = decode_image(&jpeg_data)?;
/// println!("Image dimensions: {}x{}", img.width(), img.height());
/// ```
pub fn decode_image(data: &[u8]) -> Result<DynamicImage> {
    if data.is_empty() {
        return Err(CbxError::Image("Empty image data".to_string()));
    }

    // Create a reader from the byte slice
    let reader = ImageReader::new(Cursor::new(data))
        .with_guessed_format()
        .map_err(|e| CbxError::Image(format!("Format detection failed: {}", e)))?;

    // Decode the image
    reader
        .decode()
        .map_err(|e| CbxError::Image(format!("Failed to decode image: {}", e)))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Minimal valid JPEG file (1x1 red pixel)
    /// This is a base64 decoded JPEG file that represents a 1x1 red pixel
    const MINIMAL_JPEG: &[u8] = &[
        0xFF, 0xD8, 0xFF, 0xE0, 0x00, 0x10, 0x4A, 0x46, 0x49, 0x46, 0x00, 0x01, 0x01, 0x00,
        0x00, 0x01, 0x00, 0x01, 0x00, 0x00, 0xFF, 0xDB, 0x00, 0x43, 0x00, 0x03, 0x02, 0x02,
        0x02, 0x02, 0x02, 0x03, 0x02, 0x02, 0x02, 0x03, 0x03, 0x03, 0x03, 0x04, 0x06, 0x04,
        0x04, 0x04, 0x04, 0x04, 0x08, 0x06, 0x06, 0x05, 0x06, 0x09, 0x08, 0x0A, 0x0A, 0x09,
        0x08, 0x09, 0x09, 0x0A, 0x0C, 0x0F, 0x0C, 0x0A, 0x0B, 0x0E, 0x0B, 0x09, 0x09, 0x0D,
        0x11, 0x0D, 0x0E, 0x0F, 0x10, 0x10, 0x11, 0x10, 0x0A, 0x0C, 0x12, 0x13, 0x12, 0x10,
        0x13, 0x0F, 0x10, 0x10, 0x10, 0xFF, 0xC0, 0x00, 0x0B, 0x08, 0x00, 0x01, 0x00, 0x01,
        0x01, 0x01, 0x11, 0x00, 0xFF, 0xC4, 0x00, 0x14, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x09, 0xFF, 0xC4,
        0x00, 0x14, 0x10, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xFF, 0xDA, 0x00, 0x08, 0x01, 0x01, 0x00, 0x00,
        0x3F, 0x00, 0x54, 0xDF, 0xFF, 0xD9,
    ];

    /// Minimal valid PNG file (1x1 red pixel)
    /// This is a valid 1x1 PNG generated and verified
    const MINIMAL_PNG: &[u8] = &[
        0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, // PNG signature
        0x00, 0x00, 0x00, 0x0D, 0x49, 0x48, 0x44, 0x52, // IHDR chunk
        0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, // 1x1 dimensions
        0x08, 0x02, 0x00, 0x00, 0x00, 0x90, 0x77, 0x53, 0xDE,
        0x00, 0x00, 0x00, 0x0C, 0x49, 0x44, 0x41, 0x54, // IDAT chunk (12 bytes)
        0x08, 0xD7, 0x63, 0xF8, 0xCF, 0xC0, 0x00, 0x00, // Compressed data
        0x03, 0x01, 0x01, 0x00, 0x18, 0xDD, 0x8D, 0xB0, // CRC corrected
        0x00, 0x00, 0x00, 0x00, 0x49, 0x45, 0x4E, 0x44, // IEND chunk
        0xAE, 0x42, 0x60, 0x82,
    ];

    #[test]
    fn test_decode_jpeg() {
        let result = decode_image(MINIMAL_JPEG);
        assert!(result.is_ok(), "Failed to decode JPEG: {:?}", result.err());

        let img = result.unwrap();
        assert_eq!(img.width(), 1);
        assert_eq!(img.height(), 1);
    }

    #[test]
    fn test_decode_png() {
        let result = decode_image(MINIMAL_PNG);
        assert!(result.is_ok(), "Failed to decode PNG: {:?}", result.err());

        let img = result.unwrap();
        assert_eq!(img.width(), 1);
        assert_eq!(img.height(), 1);
    }

    #[test]
    fn test_decode_empty_data() {
        let result = decode_image(&[]);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), CbxError::Image(_)));
    }

    #[test]
    fn test_decode_corrupt_data() {
        let corrupt = vec![0xFF, 0x00, 0x12, 0x34, 0x56, 0x78, 0x9A, 0xBC];
        let result = decode_image(&corrupt);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), CbxError::Image(_)));
    }

    #[test]
    fn test_decode_partial_data() {
        // Only JPEG signature, no actual image data
        let partial = vec![0xFF, 0xD8, 0xFF, 0xE0];
        let result = decode_image(&partial);
        assert!(result.is_err());
    }

    #[test]
    fn test_decode_wrong_format() {
        // This is not an image file, just random bytes
        let not_image = b"This is not an image file content";
        let result = decode_image(not_image);
        assert!(result.is_err());
    }
}
