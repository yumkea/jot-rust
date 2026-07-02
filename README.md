# Jot

<p align="center">
  <img src="./resources/jot.svg" width="72" height="72" alt="Jot logo">
</p>

<p align="center">
  A lightweight Tauri desktop app for quickly capturing notes, ideas, tasks, and Markdown-friendly writing.
</p>

<p align="center">
  <a href="https://github.com/yumkea/jot-rust/releases">Releases</a>
  ·
  <a href="https://github.com/yumkea/jot-rust/issues">Issues</a>
  ·
  <a href="./LICENSE">License</a>
</p>

<p align="center">
  <img src="https://img.shields.io/badge/Tauri-2.x-24c8db?style=flat-square" alt="Tauri 2">
  <img src="https://img.shields.io/badge/Vue-3-42b883?style=flat-square" alt="Vue 3">
  <img src="https://img.shields.io/badge/TypeScript-5-3178c6?style=flat-square" alt="TypeScript 5">
  <img src="https://img.shields.io/badge/License-GPL--3.0-4f46e5?style=flat-square" alt="GPL-3.0">
</p>

## Overview

Jot is a compact desktop note-taking app built with Tauri, Rust, Vue, and TipTap. It is designed for quick capture: open the floating window with a shortcut, write immediately, and let the app save notes automatically.

It focuses on local-first writing, a small interface, and practical editor features instead of heavy workspace management.

<p align="center">
  <img src="./assets/main_window.png" width="42%" alt="Jot dark main window">
  &nbsp;
  <img src="./assets/main_window_white.png" width="42%" alt="Jot light main window">
</p>

## Features

- Floating, borderless desktop window with transparent styling.
- Global shortcut support, defaulting to `Ctrl+J`.
- Auto-save editor with Markdown serialization.
- Markdown-friendly rich editing powered by TipTap.
- Headings, bold, italic, inline code, code blocks, lists, task lists, tables, and math.
- Inline math with `$...$` and block math with `$$...$$`.
- Search panel with title/content modes.
- History panel grouped by `updated_at` from Markdown frontmatter.
- Outline panel generated from note headings.
- Word count and selected-word count in the footer.
- Dark/light theme, accent color, bilingual UI, configurable shortcuts.
- System tray integration and optional auto-launch.
- Export to Markdown or standalone HTML.
- Local Markdown note storage with YAML-style frontmatter.

<p align="center">
  <img src="./assets/task_table_latex.png" width="42%" alt="Task list, table, and LaTeX in dark mode">
  &nbsp;
  <img src="./assets/outline.png" width="42%" alt="Outline panel in dark mode">
</p>

## Storage

Notes are stored locally as Markdown files. Each note includes frontmatter metadata:

```markdown
---
id: "..."
title: "..."
created_at: 2026-07-02 10:00:00
updated_at: 2026-07-02 10:30:00
---
Your note content...
```

The history view uses `updated_at` from frontmatter rather than filesystem modified time, so copied or restored files keep their original note timeline.

The storage directory can be changed in the app settings.

## Requirements

- Node.js 20.19 or newer
- npm
- Rust stable toolchain
- Platform-specific Tauri build dependencies

For Tauri system dependencies, see the official setup guide:

https://v2.tauri.app/start/prerequisites/

## Development

Clone the repository:

```bash
git clone git@github.com:yumkea/jot-rust.git
cd jot-rust
```

Install dependencies:

```bash
npm install
```

Run the desktop app in development mode:

```bash
npm run tauri dev
```

Run only the Vite frontend:

```bash
npm run dev
```

The frontend dev server uses:

```text
http://localhost:1420
```

## Scripts

```bash
npm run dev          # Start Vite
npm run build        # Type-check and build frontend
npm test             # Run Vitest tests
npm run tauri dev    # Start Tauri development app
npm run tauri build  # Build desktop bundles
```

## Icons And Assets

The source icon is:

```text
resources/jot.svg
```

Generated assets include:

- `resources/jot.png`
- `resources/jot.ico`
- `public/favicon.ico`
- `src-tauri/icons/*`

Helper scripts live in `resources/tools`:

```bash
cd resources/tools
uv sync
uv run python convert_svg_to_png.py
uv run python convert_png_to_ico.py
```

The local virtual environment is ignored by Git:

```text
resources/tools/.venv/
```

## Testing

Run the test suite:

```bash
npm test
```

The current tests cover Markdown conversion/editor behavior and a regression check that the sidebar logo uses the shared `resources/jot.svg` asset.

## Build

Build the production frontend:

```bash
npm run build
```

Build desktop installers/packages with Tauri:

```bash
npm run tauri build
```

Configured bundle targets currently include:

- Windows NSIS
- Windows MSI
- macOS app bundle

Additional Linux/macOS packaging depends on the host platform and installed Tauri prerequisites.

## Tech Stack

- Tauri 2
- Rust
- Vue 3
- TypeScript
- Vite
- TipTap / ProseMirror
- Markdown-it
- KaTeX
- Vitest

## Roadmap

- Release downloadable builds through GitHub Releases.
- Improve cross-platform packaging notes.
- Add more regression tests for note storage and frontmatter parsing.
- Add screenshots or short videos for the main workflows.

## Contributing

Issues and pull requests are welcome.

Before opening a pull request, please run:

```bash
npm test
npm run build
```

For larger changes, include a short explanation of the behavior being changed and any manual verification you performed.

## License

This project is licensed under the [GNU General Public License v3.0](./LICENSE).
