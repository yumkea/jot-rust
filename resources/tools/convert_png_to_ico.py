from __future__ import annotations

import argparse
from pathlib import Path

from PIL import Image


DEFAULT_SIZES = (16, 24, 32, 48, 64, 128, 256)


def parse_sizes(value: str) -> list[tuple[int, int]]:
    sizes: list[tuple[int, int]] = []
    for item in value.split(","):
        size = int(item.strip())
        if size <= 0:
            raise argparse.ArgumentTypeError("Icon sizes must be positive integers.")
        sizes.append((size, size))
    return sizes


def main() -> None:
    tools_dir = Path(__file__).resolve().parent
    resources_dir = tools_dir.parent

    parser = argparse.ArgumentParser(description="Convert a PNG file to a multi-size ICO.")
    parser.add_argument(
        "input",
        nargs="?",
        type=Path,
        default=resources_dir / "jot.png",
        help="Input PNG path. Defaults to ../jot.png.",
    )
    parser.add_argument(
        "output",
        nargs="?",
        type=Path,
        default=resources_dir / "jot.ico",
        help="Output ICO path. Defaults to ../jot.ico.",
    )
    parser.add_argument(
        "--sizes",
        type=parse_sizes,
        default=[(size, size) for size in DEFAULT_SIZES],
        help="Comma-separated square icon sizes. Defaults to 16,24,32,48,64,128,256.",
    )
    args = parser.parse_args()

    with Image.open(args.input) as image:
        image = image.convert("RGBA")
        image.save(args.output, format="ICO", sizes=args.sizes)

    size_list = ", ".join(f"{width}x{height}" for width, height in args.sizes)
    print(f"Wrote {args.output} ({size_list})")


if __name__ == "__main__":
    main()
