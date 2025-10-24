//! Integration test for WebP image decoding
//! Verifies that WebP images can be decoded and converted to thumbnails

use cbxshell::image_processor::thumbnail::create_thumbnail_with_size;
use std::fs;
use std::path::PathBuf;

/// Minimal valid WebP file (1x1 red pixel, lossy VP8 format)
/// Source: Created with libwebp, verified with dwebp
const MINIMAL_WEBP: &[u8] = include_bytes!("../test_data/minimal.webp");

#[test]
fn test_webp_decoding() {
    // Test data directory
    let test_data_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("test_data");

    // If test data doesn't exist, create minimal WebP in memory
    println!("Testing WebP decoding with minimal WebP file ({} bytes)", MINIMAL_WEBP.len());

    // Attempt to create thumbnail from WebP
    let result = create_thumbnail_with_size(MINIMAL_WEBP, 256, 256);

    match &result {
        Ok(hbitmap) => {
            println!("SUCCESS: WebP decoded and HBITMAP created: {:?}", hbitmap);
            // Clean up
            unsafe {
                use windows::Win32::Graphics::Gdi::DeleteObject;
                let _ = DeleteObject(*hbitmap);
            }
        }
        Err(e) => {
            println!("FAILED: WebP decoding error: {}", e);
            println!("Error type: {:?}", e);
        }
    }

    assert!(result.is_ok(), "WebP decoding should succeed, but got: {:?}", result.err());
}

#[test]
fn test_image_crate_webp_support() {
    use image::ImageReader;
    use std::io::Cursor;

    println!("Testing image crate WebP support directly...");

    let reader = ImageReader::new(Cursor::new(MINIMAL_WEBP))
        .with_guessed_format();

    match reader {
        Ok(reader) => {
            println!("Format detection: {:?}", reader.format());

            match reader.decode() {
                Ok(img) => {
                    println!("SUCCESS: Decoded WebP image: {}x{}", img.width(), img.height());
                    assert!(img.width() > 0 && img.height() > 0);
                }
                Err(e) => {
                    panic!("Failed to decode WebP: {}", e);
                }
            }
        }
        Err(e) => {
            panic!("Failed to create image reader: {}", e);
        }
    }
}
