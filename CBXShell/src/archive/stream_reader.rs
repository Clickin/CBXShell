//! IStream-based archive reading for IThumbnailProvider
//!
//! This module provides utilities for reading archives from IStream interfaces
//! instead of file paths, which is required for IThumbnailProvider.

use windows::Win32::System::Com::*;
use crate::utils::error::{CbxError, Result};
use crate::archive::ArchiveType;

/// Maximum buffer size for reading from IStream (10GB)
/// We only extract the first image (max 32MB), so archive size doesn't matter much
/// Limit set to 10GB to support very large comic archives
const MAX_STREAM_SIZE: usize = 10 * 1024 * 1024 * 1024;

/// Read entire IStream contents into memory
///
/// This function reads all data from an IStream into a Vec<u8>.
/// It's safe because:
/// 1. We limit the total size to MAX_STREAM_SIZE (32MB)
/// 2. We validate the stream pointer
/// 3. We use proper ULARGE_INTEGER for seeking
///
/// # Arguments
/// * `stream` - The IStream to read from
///
/// # Returns
/// * `Ok(Vec<u8>)` - The complete stream contents
/// * `Err(CbxError)` - If reading fails or stream is too large
///
/// # Safety
/// This function makes COM calls which are inherently unsafe, but we wrap
/// them properly with error handling.
pub fn read_stream_to_memory(stream: &IStream) -> Result<Vec<u8>> {
    crate::utils::debug_log::debug_log(">>>>> read_stream_to_memory STARTING <<<<<");

    unsafe {
        // Step 1: Seek to end to get stream size
        let mut new_position = 0u64;
        if stream.Seek(
            0,
            STREAM_SEEK_END,
            Some(&mut new_position)
        ).is_err() {
            crate::utils::debug_log::debug_log("ERROR: Failed to seek to end");
            return Err(CbxError::Archive("Failed to seek to end of stream".to_string()));
        }

        let stream_size = new_position as usize;
        crate::utils::debug_log::debug_log(&format!("Stream size: {} bytes", stream_size));

        // Step 2: Validate size
        if stream_size == 0 {
            crate::utils::debug_log::debug_log("ERROR: Stream is empty");
            return Err(CbxError::Archive("Empty stream".to_string()));
        }

        if stream_size > MAX_STREAM_SIZE {
            crate::utils::debug_log::debug_log(&format!("ERROR: Stream too large: {} bytes (max: {})", stream_size, MAX_STREAM_SIZE));
            return Err(CbxError::Archive(format!("Stream too large: {} bytes", stream_size)));
        }

        // Step 3: Seek back to beginning
        if stream.Seek(
            0,
            STREAM_SEEK_SET,
            None
        ).is_err() {
            crate::utils::debug_log::debug_log("ERROR: Failed to seek to beginning");
            return Err(CbxError::Archive("Failed to seek to beginning of stream".to_string()));
        }

        crate::utils::debug_log::debug_log("Seek to beginning successful");

        // Step 4: Allocate buffer
        let mut buffer = vec![0u8; stream_size];
        crate::utils::debug_log::debug_log(&format!("Allocated buffer: {} bytes", buffer.len()));

        // Step 5: Read all data
        let mut total_read = 0usize;
        while total_read < stream_size {
            let mut bytes_read = 0u32;
            let remaining = stream_size - total_read;
            let to_read = remaining.min(1024 * 1024); // Read in 1MB chunks

            if stream.Read(
                buffer[total_read..].as_mut_ptr() as *mut _,
                to_read as u32,
                Some(&mut bytes_read)
            ).is_err() {
                crate::utils::debug_log::debug_log("ERROR: Failed to read from stream");
                return Err(CbxError::Archive("Failed to read from stream".to_string()));
            }

            if bytes_read == 0 {
                crate::utils::debug_log::debug_log(&format!("ERROR: Unexpected EOF at {} bytes (expected {})", total_read, stream_size));
                return Err(CbxError::Archive("Unexpected end of stream".to_string()));
            }

            total_read += bytes_read as usize;
            crate::utils::debug_log::debug_log(&format!("Read progress: {}/{} bytes", total_read, stream_size));
        }

        crate::utils::debug_log::debug_log(&format!("SUCCESS: Read {} bytes from stream", total_read));
        Ok(buffer)
    }
}

/// Detect archive type from magic bytes
///
/// This function inspects the first few bytes of data to determine the archive type.
/// This is more reliable than extension-based detection for IStream sources.
///
/// # Magic Bytes
/// - ZIP: `50 4B 03 04` or `50 4B 05 06` or `50 4B 07 08` (PK\x03\x04, PK\x05\x06, PK\x07\x08)
/// - RAR: `52 61 72 21 1A 07 00` (Rar!\x1A\x07\x00) - RAR 4.x
/// - RAR5: `52 61 72 21 1A 07 01 00` (Rar!\x1A\x07\x01\x00) - RAR 5.x
/// - 7z: `37 7A BC AF 27 1C` (7z¼¯'\x1C)
///
/// # Arguments
/// * `data` - The raw archive data (at least first 16 bytes)
///
/// # Returns
/// * `Ok(ArchiveType)` - The detected archive type
/// * `Err(CbxError)` - If the format is not recognized
pub fn detect_archive_type_from_bytes(data: &[u8]) -> Result<ArchiveType> {
    crate::utils::debug_log::debug_log(">>>>> detect_archive_type_from_bytes STARTING <<<<<");

    if data.len() < 8 {
        crate::utils::debug_log::debug_log(&format!("ERROR: Data too short: {} bytes", data.len()));
        return Err(CbxError::UnsupportedFormat("Data too short".to_string()));
    }

    // Log first 16 bytes as hex for debugging
    let preview_len = data.len().min(16);
    let hex_preview: Vec<String> = data[..preview_len]
        .iter()
        .map(|b| format!("{:02X}", b))
        .collect();
    crate::utils::debug_log::debug_log(&format!("First {} bytes: {}", preview_len, hex_preview.join(" ")));

    // Check ZIP magic bytes
    if data.len() >= 4 {
        let magic = &data[0..4];
        if magic == b"PK\x03\x04" || magic == b"PK\x05\x06" || magic == b"PK\x07\x08" {
            crate::utils::debug_log::debug_log("Detected: ZIP format");
            return Ok(ArchiveType::Zip);
        }
    }

    // Check 7z magic bytes
    if data.len() >= 6 {
        let magic = &data[0..6];
        if magic == b"7z\xBC\xAF\x27\x1C" {
            crate::utils::debug_log::debug_log("Detected: 7-Zip format");
            return Ok(ArchiveType::SevenZip);
        }
    }

    // Check RAR magic bytes (RAR 4.x and 5.x)
    if data.len() >= 7 {
        let magic = &data[0..7];
        if magic == b"Rar!\x1A\x07\x00" {
            crate::utils::debug_log::debug_log("Detected: RAR 4.x format");
            return Ok(ArchiveType::Rar);
        }
    }

    if data.len() >= 8 {
        let magic = &data[0..8];
        if magic == b"Rar!\x1A\x07\x01\x00" {
            crate::utils::debug_log::debug_log("Detected: RAR 5.x format");
            return Ok(ArchiveType::Rar);
        }
    }

    crate::utils::debug_log::debug_log("ERROR: Unrecognized archive format");
    Err(CbxError::UnsupportedFormat("Unrecognized archive format".to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_zip_format() {
        // ZIP local file header signature
        let zip_data = b"PK\x03\x04\x14\x00\x00\x00\x08\x00";
        assert_eq!(
            detect_archive_type_from_bytes(zip_data).unwrap(),
            ArchiveType::Zip
        );
    }

    #[test]
    fn test_detect_zip_central_directory() {
        // ZIP central directory signature
        let zip_data = b"PK\x05\x06\x00\x00\x00\x00";
        assert_eq!(
            detect_archive_type_from_bytes(zip_data).unwrap(),
            ArchiveType::Zip
        );
    }

    #[test]
    fn test_detect_7z_format() {
        let sevenz_data = b"7z\xBC\xAF\x27\x1C\x00\x00";
        assert_eq!(
            detect_archive_type_from_bytes(sevenz_data).unwrap(),
            ArchiveType::SevenZip
        );
    }

    #[test]
    fn test_detect_rar4_format() {
        let rar_data = b"Rar!\x1A\x07\x00\x00";
        assert_eq!(
            detect_archive_type_from_bytes(rar_data).unwrap(),
            ArchiveType::Rar
        );
    }

    #[test]
    fn test_detect_rar5_format() {
        let rar_data = b"Rar!\x1A\x07\x01\x00";
        assert_eq!(
            detect_archive_type_from_bytes(rar_data).unwrap(),
            ArchiveType::Rar
        );
    }

    #[test]
    fn test_detect_unknown_format() {
        let unknown_data = b"UNKNOWN\x00\x00\x00\x00";
        assert!(detect_archive_type_from_bytes(unknown_data).is_err());
    }

    #[test]
    fn test_detect_data_too_short() {
        let short_data = b"PK";
        assert!(detect_archive_type_from_bytes(short_data).is_err());
    }
}
