///! Archive format handling
///!
///! Supports ZIP, RAR, and 7z formats for comic book archives

use std::path::Path;
use crate::utils::error::{CbxError, Result};

mod utils;
mod config;
mod zip;
mod sevenz;
mod rar;
pub mod stream_reader;

// Re-export utilities for internal use only (not used in public API)
pub use config::should_sort_images;
#[allow(dead_code)] // Used by open_archive function and part of public API
pub use zip::ZipArchive;
#[allow(dead_code)] // Used by open_archive function and part of public API
pub use sevenz::SevenZipArchive;
#[allow(dead_code)] // Used by open_archive function and part of public API
pub use rar::RarArchive;
pub use stream_reader::{read_stream_to_memory, detect_archive_type_from_bytes};

/// Represents an entry in an archive
#[derive(Debug, Clone)]
pub struct ArchiveEntry {
    pub name: String,
    pub size: u64,
    #[allow(dead_code)] // Part of public API, may be used in future
    pub is_directory: bool,
}

/// Archive metadata
#[derive(Debug, Clone)]
#[allow(dead_code)] // Part of public API, may be used in future
pub struct ArchiveMetadata {
    pub total_files: usize,
    pub image_count: usize,
    pub compressed_size: u64,
    pub archive_type: ArchiveType,
}

/// Archive type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArchiveType {
    Zip,
    Rar,
    SevenZip,
}

impl ArchiveType {
    /// Detect archive type from file extension
    #[allow(dead_code)] // Part of public API, may be used in future
    pub fn from_extension(ext: &str) -> Option<Self> {
        match ext.to_lowercase().as_str() {
            "zip" | "cbz" | "epub" | "phz" => Some(Self::Zip),
            "rar" | "cbr" => Some(Self::Rar),
            "7z" | "cb7" => Some(Self::SevenZip),
            _ => None,
        }
    }

    #[allow(dead_code)] // Part of public API, may be used in future
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Zip => "ZIP",
            Self::Rar => "RAR",
            Self::SevenZip => "7-Zip",
        }
    }
}

/// Archive trait for different archive formats
#[allow(dead_code)] // Part of public API, used by archive implementations
pub trait Archive {
    /// Open an archive from a file path
    fn open(path: &Path) -> Result<Box<dyn Archive>>
    where
        Self: Sized;

    /// Find the first image in the archive (optionally sorted alphabetically)
    fn find_first_image(&self, sort: bool) -> Result<ArchiveEntry>;

    /// Extract an entry to a byte vector
    fn extract_entry(&self, entry: &ArchiveEntry) -> Result<Vec<u8>>;

    /// Get archive metadata
    fn get_metadata(&self) -> Result<ArchiveMetadata>;

    /// Get archive type
    fn archive_type(&self) -> ArchiveType;
}

/// Open an archive of any supported type from a file path
#[allow(dead_code)] // Part of public API, may be used in future
pub fn open_archive(path: &Path) -> Result<Box<dyn Archive>> {
    let extension = path
        .extension()
        .and_then(|s| s.to_str())
        .ok_or(CbxError::InvalidPath)?;

    let archive_type = ArchiveType::from_extension(extension)
        .ok_or_else(|| CbxError::UnsupportedFormat(extension.to_string()))?;

    match archive_type {
        ArchiveType::Zip => <ZipArchive as Archive>::open(path),
        ArchiveType::Rar => <RarArchive as Archive>::open(path),
        ArchiveType::SevenZip => <SevenZipArchive as Archive>::open(path),
    }
}

/// Open an archive from in-memory data (for IStream support)
///
/// This function detects the archive type from magic bytes and opens
/// the appropriate archive handler from memory.
///
/// # Arguments
/// * `data` - The complete archive data in memory
///
/// # Returns
/// * `Ok(Box<dyn Archive>)` - Opened archive handler
/// * `Err(CbxError)` - If the format is unsupported or opening fails
pub fn open_archive_from_memory(data: Vec<u8>) -> Result<Box<dyn Archive>> {
    use std::io::Cursor;

    crate::utils::debug_log::debug_log(">>>>> open_archive_from_memory STARTING <<<<<");
    crate::utils::debug_log::debug_log(&format!("Archive data size: {} bytes", data.len()));

    // Detect archive type from magic bytes
    let archive_type = detect_archive_type_from_bytes(&data)?;
    crate::utils::debug_log::debug_log(&format!("Detected archive type: {:?}", archive_type));

    match archive_type {
        ArchiveType::Zip => {
            // Create ZIP archive from memory
            let cursor = Cursor::new(data);
            let zip_reader = ::zip::ZipArchive::new(cursor)
                .map_err(|e| CbxError::Archive(format!("Failed to open ZIP from memory: {}", e)))?;

            Ok(Box::new(zip::ZipArchiveFromMemory::new(zip_reader)))
        }
        ArchiveType::SevenZip => {
            // Create 7z archive from memory
            let cursor = Cursor::new(data);
            Ok(Box::new(sevenz::SevenZipArchiveFromMemory::new(cursor)?))
        }
        ArchiveType::Rar => {
            // Create RAR archive from memory (uses temp file)
            Ok(Box::new(rar::RarArchiveFromMemory::new(data)?))
        }
    }
}
