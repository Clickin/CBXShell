#!/usr/bin/env python3
"""
Generate MSIX package logo assets from SVG icon

This script converts cbx_icon.svg to all required PNG sizes for MSIX packaging.

Requirements:
    pip install cairosvg pillow

Usage:
    python generate_logos.py
"""

import os
from pathlib import Path

try:
    import cairosvg
    from PIL import Image
    from io import BytesIO
except ImportError:
    print("Error: Required packages not installed")
    print("Please run: pip install cairosvg pillow")
    exit(1)


# Asset specifications for MSIX packaging
ASSETS = {
    "StoreLogo.png": 50,           # 50x50
    "Square44x44Logo.png": 44,     # 44x44
    "Square150x150Logo.png": 150,  # 150x150
    "Wide310x150Logo.png": (310, 150),  # 310x150 (wide)
    "SplashScreen.png": (620, 300), # 620x300 (wide)
}


def svg_to_png(svg_path: Path, output_path: Path, size: tuple[int, int]) -> None:
    """Convert SVG to PNG with specified size"""
    print(f"  Creating {output_path.name} ({size[0]}x{size[1]})...")

    # Convert SVG to PNG in memory
    png_data = cairosvg.svg2png(
        url=str(svg_path),
        output_width=size[0],
        output_height=size[1],
    )

    # Open with PIL for optional processing
    img = Image.open(BytesIO(png_data))

    # Ensure RGBA mode
    if img.mode != 'RGBA':
        img = img.convert('RGBA')

    # Save as PNG
    img.save(output_path, 'PNG', optimize=True)


def main():
    # Get script directory
    script_dir = Path(__file__).parent
    svg_path = script_dir / "cbx_icon.svg"

    if not svg_path.exists():
        print(f"Error: {svg_path} not found!")
        return 1

    print(f"Generating MSIX logos from {svg_path.name}...\n")

    # Generate each asset
    for filename, size_spec in ASSETS.items():
        output_path = script_dir / filename

        # Handle both square (int) and rectangular (tuple) sizes
        if isinstance(size_spec, int):
            size = (size_spec, size_spec)
        else:
            size = size_spec

        svg_to_png(svg_path, output_path, size)

    print(f"\n✓ Successfully generated {len(ASSETS)} logo assets!")
    print(f"  Output directory: {script_dir}")

    return 0


if __name__ == "__main__":
    exit(main())
