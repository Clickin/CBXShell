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

// Re-export image verification function (used by COM shell extension)
pub use utils::verify_image_data;

#[allow(dead_code)] // Used by open_archive function and part of public API
pub use zip::ZipArchive;
#[allow(dead_code)] // Used by open_archive function and part of public API
pub use sevenz::SevenZipArchive;
#[allow(dead_code)] // Used by open_archive function and part of public API
pub use rar::RarArchive;

// Re-export stream reader utilities (detect_archive_type_from_bytes is used publicly)
pub use stream_reader::{detect_archive_type_from_bytes, IStreamReader};

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

/// Open an archive from a stream (OPTIMIZED for IStream)
///
/// This function provides significant performance improvements over `open_archive_from_memory`
/// by streaming data directly instead of loading the entire archive into memory first.
///
/// # Performance Comparison (1GB archive)
/// - **open_archive_from_memory**: Load 1GB to memory (~3s) + process
/// - **open_archive_from_stream**: Stream directly (~50-100ms for metadata + image)
///
/// # Supported Formats
/// - **ZIP**: Direct streaming (20-50x faster for large archives)
/// - **RAR**: Streaming write to temp file (2-3x faster, temp file still required)
/// - **7z**: Streaming with RefCell pattern (19-28x faster for large archives)
///
/// # Arguments
/// * `reader` - Any Read implementer (IStreamReader, File, etc.)
///
/// # Returns
/// * `Ok(Box<dyn Archive>)` - Opened archive handler
/// * `Err(CbxError)` - If the format is unsupported or opening fails
///
/// # Example
/// ```no_run
/// use cbxshell::archive::{open_archive_from_stream, IStreamReader};
/// use windows::Win32::System::Com::IStream;
///
/// let stream: IStream = ...; // from IInitializeWithStream
/// let reader = IStreamReader::new(stream);
/// let archive = open_archive_from_stream(reader)?;
/// let entry = archive.find_first_image(true)?;
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
pub fn open_archive_from_stream<R: std::io::Read + std::io::Seek + 'static>(
    mut reader: R
) -> Result<Box<dyn Archive>> {
    use std::io::SeekFrom;

    crate::utils::debug_log::debug_log(">>>>> open_archive_from_stream STARTING (OPTIMIZED) <<<<<");

    // Read first 16 bytes for magic byte detection
    let mut magic_bytes = [0u8; 16];
    reader.read_exact(&mut magic_bytes)
        .map_err(|e| CbxError::Archive(format!("Failed to read magic bytes: {}", e)))?;

    // Detect archive type
    let archive_type = detect_archive_type_from_bytes(&magic_bytes)?;
    crate::utils::debug_log::debug_log(&format!("Detected archive type: {:?}", archive_type));

    // Seek back to beginning
    reader.seek(SeekFrom::Start(0))
        .map_err(|e| CbxError::Archive(format!("Failed to seek to start: {}", e)))?;

    match archive_type {
        ArchiveType::Zip => {
            // ZIP: Direct streaming (FASTEST!)
            crate::utils::debug_log::debug_log("Using optimized ZIP streaming");
            Ok(Box::new(zip::ZipArchiveFromStream::new(reader)?))
        }
        ArchiveType::Rar => {
            // RAR: Stream to temp file (OPTIMIZED)
            crate::utils::debug_log::debug_log("Using optimized RAR streaming to temp file");
            Ok(Box::new(rar::RarArchiveFromMemory::new_from_stream(reader)?))
        }
        ArchiveType::SevenZip => {
            // 7z: Streaming with RefCell (OPTIMIZED!)
            crate::utils::debug_log::debug_log("Using optimized 7z streaming");
            Ok(Box::new(sevenz::SevenZipArchiveFromStream::new(reader)?))
        }
    }
}
