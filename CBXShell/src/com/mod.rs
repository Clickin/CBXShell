///! COM implementation for CBXShell Windows Shell Extension

mod class_factory;
mod cbxshell;
mod persist_file;
mod extract_image;
mod query_info;

pub use class_factory::ClassFactory;
pub use cbxshell::CBXShell;

use windows::core::GUID;

/// CLSID for CBXShell COM object
/// {9E6ECB90-5A61-42BD-B851-D3297D9C7F39}
pub const CLSID_CBXShell: GUID = GUID::from_u128(0x9E6ECB90_5A61_42BD_B851_D3297D9C7F39);
