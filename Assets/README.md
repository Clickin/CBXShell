# MSIX Package Assets

This directory contains the required assets for the MSIX package. These images are used in the Windows Store and Start Menu.

## Required Assets

### Store Logo
- **StoreLogo.png**: 50x50 pixels
  - Used in the Windows Store listing

### Application Tiles
- **Square44x44Logo.png**: 44x44 pixels
  - App icon in Start Menu and taskbar
- **Square150x150Logo.png**: 150x150 pixels
  - Medium tile in Start Menu
- **Wide310x150Logo.png**: 310x150 pixels
  - Wide tile in Start Menu
- **SplashScreen.png**: 620x300 pixels
  - Shown when app launches

## Design Guidelines

- Use transparent backgrounds for logos
- Maintain consistent branding across all assets
- Follow Windows 11 design principles (rounded corners, modern look)
- Recommended: Create vector source (SVG) and export to PNG at required sizes

## Creating Assets

You can create these assets using:
- Adobe Illustrator / Photoshop
- Figma
- Inkscape (free)
- GIMP (free)

Alternatively, use the Windows App Studio Asset Generator or Visual Studio's built-in asset generator.

## Placeholder Note

Currently, this directory contains placeholder assets. Replace these with proper branded images before publishing to Windows Store.
