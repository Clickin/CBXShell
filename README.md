# CBXShell-rs

Windows Shell Extension for ZIP/RAR/7z comic book archive thumbnail preview with WebP support.

> **Rust reimplementation** of the original [CBXShell](https://www.codeproject.com/Articles/2270/CBXShell-A-Thumbnail-Shell-Extension-for-Comic-Boo) C++ project by T800 Productions, modernized with Rust for improved memory safety and maintainability.

## Features

- **Multi-Format Support**: ZIP, RAR, 7z archives (.cbz, .cbr, .cb7, .epub, .phz)
- **Modern Image Formats**: JPEG, PNG, GIF, BMP, TIFF, ICO, **WebP**, **AVIF**
- **Pure Rust**: Memory-safe implementation using `windows-rs`
- **Shell Integration**: Thumbnail previews and tooltips in Windows Explorer
- **Per-User Installation**: No administrator rights required

## Building

### Prerequisites

- Rust 1.70+ (stable)
- Windows 10 SDK or later
- Visual Studio Build Tools (for MSVC linker)

### Build Commands

```bash
# Clone the repository
git clone https://github.com/Clickin/CBXShell-rs.git
cd CBXShell-rs

# Build release version
cargo build --release

# Build for both x64 and x86
cargo build --release --target x86_64-pc-windows-msvc
cargo build --release --target i686-pc-windows-msvc

# Run tests
cargo test
```

## Installation

### Option 1: Windows Store (Recommended)

Download and install CBXShell directly from the Microsoft Store for automatic updates and easy installation.

**Requirements:**
- Windows 11 21H2 or later
- Approximately 5 MB of disk space

### Option 2: NSIS Installer (GitHub Releases)

Download the latest `CBXShell-Setup-x.x.x.exe` from the [Releases](https://github.com/yourusername/cbxshell/releases) page.

**Features:**
- Automatic architecture detection (32-bit/64-bit)
- One-click installation
- Start Menu shortcuts
- Automatic COM registration
- Easy uninstallation via Add/Remove Programs

**Installation Steps:**
1. Download `CBXShell-Setup-x.x.x.exe`
2. Run the installer (requires administrator privileges)
3. Select which file formats to enable (CBZ, CBR, ZIP, RAR, 7Z)
4. Restart Windows Explorer when prompted

### Option 3: Manual Installation (Advanced Users)

For development or custom deployment:

```cmd
# Build the project
cargo build --release

# Navigate to the build output
cd target\release

# Register (run as administrator)
regsvr32 cbxshell.dll

# Unregister
regsvr32 /u cbxshell.dll
```

### Restart Windows Explorer

After installation, restart Windows Explorer to see thumbnails:

```cmd
taskkill /f /im explorer.exe
start explorer.exe
```

## Development Status

### ✅ Phase 1: COM Infrastructure (COMPLETE)
- [x] Rust workspace setup
- [x] COM DLL exports (DllGetClassObject, DllCanUnloadNow, DllRegisterServer, DllUnregisterServer)
- [x] COM class factory with reference counting
- [x] IPersistFile, IExtractImage2, IQueryInfo interface stubs
- [x] Registry management
- [x] Build system

### 🚧 Phase 2: Archive Support (IN PROGRESS)
- [ ] ZIP/CBZ extraction (`zip` crate)
- [ ] RAR/CBR extraction (`unrar` crate from https://github.com/muja/unrar.rs)
- [ ] 7z/CB7 extraction (`sevenz-rust` crate)
- [ ] Archive trait abstraction
- [ ] Alphabetical sorting with natural order

### 📋 Phase 3: Image Processing (PENDING)
- [ ] Image format detection
- [ ] WebP/AVIF support
- [ ] Thumbnail generation with high-quality resizing
- [ ] HBITMAP conversion for Windows

### 📋 Phase 4: Shell Extension Integration (PENDING)
- [ ] Implement thumbnail extraction logic
- [ ] Implement tooltip generation
- [ ] Error handling and logging
- [ ] Integration testing with Windows Explorer

### 📋 Phase 5: Configuration Manager (PENDING)
- [ ] GUI for enabling/disabling handlers
- [ ] Format selection
- [ ] Settings management

### 📋 Phase 6: Testing & Polish (PENDING)
- [ ] Comprehensive test suite
- [ ] Performance benchmarking
- [ ] Memory leak testing
- [ ] Windows 11 compatibility testing

## Project Structure

```
CBXShell/
├── Cargo.toml                   # Workspace configuration
├── cbxshell/
│   ├── Cargo.toml               # Library configuration
│   ├── build.rs                 # Build script
│   ├── src/
│   │   ├── lib.rs               # DLL entry point & COM exports
│   │   ├── com/                 # COM implementation
│   │   │   ├── mod.rs
│   │   │   ├── class_factory.rs # COM class factory
│   │   │   ├── cbxshell.rs      # Main COM object
│   │   │   ├── persist_file.rs
│   │   │   ├── extract_image.rs
│   │   │   └── query_info.rs
│   │   ├── archive/             # Archive format support
│   │   │   └── mod.rs
│   │   ├── image_processor/     # Image processing
│   │   │   └── mod.rs
│   │   ├── registry/            # Registry management
│   │   │   └── mod.rs
│   │   ├── utils/               # Utilities
│   │   │   ├── mod.rs
│   │   │   └── error.rs
│   │   └── manager/             # Configuration GUI
│   │       └── main.rs
│   └── tests/                   # Integration tests
└── README.md
```

## Architecture

### COM Implementation

CBXShell implements three Windows Shell Extension interfaces:

1. **IPersistFile**: Receives the archive file path
2. **IExtractImage2**: Extracts thumbnail bitmap
3. **IQueryInfo**: Provides tooltip information

### Archive Support

Uses a trait-based architecture for extensibility:

```rust
pub trait Archive {
    fn open(path: &Path) -> Result<Box<dyn Archive>>;
    fn find_first_image(&self, sort: bool) -> Result<ArchiveEntry>;
    fn extract_entry(&self, entry: &ArchiveEntry) -> Result<Vec<u8>>;
    fn get_metadata(&self) -> Result<ArchiveMetadata>;
}
```

Implementations:
- **ZipArchive**: Pure Rust via `zip` crate
- **RarArchive**: Via `unrar` crate (https://github.com/muja/unrar.rs)
- **SevenZipArchive**: Pure Rust via `sevenz-rust` crate

## Logging

Enable debug logging:

```cmd
set RUST_LOG=cbxshell=debug
regsvr32 cbxshell.dll
```

Logs are output to debug console (use DebugView or similar tools).

## Contributing

This is a rewrite of the original C++ CBXShell project. Contributions are welcome!

### Guidelines

- Follow Rust best practices and idioms
- Maintain memory safety (no unsafe code without justification)
- Add tests for new functionality
- Document public APIs

## Distribution

### Building MSIX Package (Windows Store)

Create an MSIX package for Windows Store submission:

```powershell
# Build MSIX package
.\build_msix.ps1 -Configuration Release -Platform x64

# Output: dist\msix\CBXShell_5.0.0.0_x64.msix
```

**Requirements:**
- Windows SDK 10.0.22621.0 or later
- Code signing certificate for Store submission

### Building NSIS Installer (GitHub Releases)

Create a standalone installer for direct distribution:

```powershell
# Build NSIS installer
.\build_nsis.ps1 -Configuration Release

# Output: dist\CBXShell-Setup-5.0.0.exe
```

**Requirements:**
- [NSIS 3.x](https://nsis.sourceforge.io/Download) or later

### Asset Requirements

For MSIX packaging, create the following assets in the `Assets/` directory:
- `StoreLogo.png` (50x50)
- `Square44x44Logo.png` (44x44)
- `Square150x150Logo.png` (150x150)
- `Wide310x150Logo.png` (310x150)
- `SplashScreen.png` (620x300)

See `Assets/README.md` for detailed specifications.

## License

This project is licensed under the Code Project Open License (CPOL) 1.02.
See `The Code Project Open License (CPOL) 1.02.md` for details.

## Credits

- Original C++ implementation by the CBXShell team
- Rust rewrite incorporating modern archive and image format support
- Uses `windows-rs` for COM interop (Microsoft)
- Uses `unrar` crate from https://github.com/muja/unrar.rs

## Roadmap

- **v5.0.0**: Initial Rust release with ZIP/RAR/7z and WebP support
- **v5.1.0**: Windows 11 specific enhancements (IThumbnailProvider, dark mode)
- **v5.2.0**: ARM64 support
- **v6.0.0**: Additional archive formats (TAR.GZ, etc.)
