# Privacy Policy for CBXShell-rs

**Last Updated: October 25, 2024**

## Overview

CBXShell-rs is a Windows Shell Extension that provides thumbnail previews and tooltips for comic book archive files (CBZ, CBR, ZIP, and RAR formats) directly in Windows Explorer. This privacy policy explains how our application handles user data.

## What CBXShell-rs Does

CBXShell-rs integrates with Windows Explorer to:
- Display thumbnail previews of the first image in comic book archive files
- Show file information tooltips (image count, total size, etc.)
- Allow users to configure which file types display thumbnails

The application runs locally on your computer as a COM shell extension and operates entirely offline.

## Data Collection

**CBXShell-rs does NOT collect, store, transmit, or share any user data.**

Specifically, we do not:
- Collect personal information
- Track user behavior or usage statistics
- Send any data to remote servers
- Access the internet or make network connections
- Store user files or file contents
- Use analytics or telemetry services
- Share information with third parties

## How CBXShell-rs Works

1. **Local Processing Only**: When you view a folder containing comic book archives in Windows Explorer, CBXShell-rs temporarily reads the archive files on your local computer to extract thumbnail images.

2. **Temporary Memory Usage**: Image data is processed in memory only for thumbnail generation and is immediately discarded after the thumbnail is displayed.

3. **No File Storage**: The application does not save, copy, or store any of your files or their contents.

4. **User Preferences**: The only data stored is your preference settings (such as which file types should display thumbnails), which are saved locally in the Windows Registry on your computer.

## Why CBXShell-rs Requires runFullTrust Capability

The `runFullTrust` capability is required because:

1. **COM Shell Extension**: CBXShell-rs must register as a COM server and run within the Windows Explorer process to provide shell integration.

2. **File System Access**: The application needs to read compressed archive files (ZIP/RAR) from your local file system to extract thumbnail images.

3. **Desktop Bridge Architecture**: As a classic Win32 shell extension, it requires full trust to maintain compatibility with Windows Shell Extension architecture.

**This capability is NOT used to collect data, access the internet, or perform any operations beyond providing thumbnail previews for archive files.**

## User Control

You have complete control over CBXShell-rs:
- **Installation**: You can install or uninstall the application at any time through Windows Settings.
- **Configuration**: Use the included CBXShell Manager application to enable or disable thumbnail previews for specific file types.
- **File Access**: The application only accesses files that you explicitly view in Windows Explorer.

## Open Source

CBXShell-rs is open source software. You can review the complete source code at:
https://github.com/Clickin/CBXShell

The source code demonstrates that no data collection or network communication occurs.

## Changes to This Policy

We may update this privacy policy from time to time. Any changes will be posted on our GitHub repository and updated in the Microsoft Store listing.

## Contact

If you have questions about this privacy policy, please:
- Open an issue on our GitHub repository: https://github.com/Clickin/CBXShell/issues
- Contact the developer through the Microsoft Store listing

## Developer Information

- **Developer Name**: Clickin
- **Application Name**: CBXShell-rs
- **Application Type**: Windows Shell Extension (Desktop Application)
- **Platform**: Windows 10/11

---

**Summary**: CBXShell-rs is a privacy-respecting, offline-only shell extension that does not collect, transmit, or store any user data. It only processes your local archive files temporarily in memory to display thumbnails in Windows Explorer.
