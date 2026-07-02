from __future__ import annotations

import argparse
from pathlib import Path

import cairosvg


def main() -> None:
    tools_dir = Path(__file__).resolve().parent
    resources_dir = tools_dir.parent

    parser = argparse.ArgumentParser(description="Convert an SVG file to a transparent PNG.")
    parser.add_argument(
        "input",
        nargs="?",
        type=Path,
        default=resources_dir / "jot.svg",
        help="Input SVG path. Defaults to ../jot.svg.",
    )
    parser.add_argument(
        "output",
        nargs="?",
        type=Path,
        default=resources_dir / "jot.png",
        help="Output PNG path. Defaults to ../jot.png.",
    )
    parser.add_argument("--size", type=int, default=4096, help="Output width and height in pixels.")
    args = parser.parse_args()

    cairosvg.svg2png(
        url=str(args.input),
        write_to=str(args.output),
        output_width=args.size,
        output_height=args.size,
    )

    print(f"Wrote {args.output} ({args.size}x{args.size})")


if __name__ == "__main__":
    main()
