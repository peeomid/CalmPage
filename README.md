# CalmPage

CalmPage is a macOS app for reading Markdown files with less noise and more focus.

It is made for people who:

- read notes, docs, or articles in local Markdown folders
- want a clean reading view instead of an editor
- like keyboard shortcuts
- want fast search, tabs, workspaces, and a calm layout

## What CalmPage does

- Opens one or many Markdown folders
- Shows your files in a library view
- Lets you open several files in tabs
- Offers focus mode for distraction-free reading
- Shows table of contents for long documents
- Supports command palette search for files, tabs, headings, actions, settings, and workspaces
- Lets you customize reading style, colors, spacing, and Markdown rendering

## What CalmPage is for

CalmPage is for reading, not writing.

Use it when you want:

- quick access to a vault of Markdown notes
- a simple way to jump between files
- a better reading view for long documents
- a desktop app that feels lighter than a full editor

## Quick Start

1. Open CalmPage.
2. Add a folder from the left Library panel.
3. Click a Markdown file to open it.
4. Use tabs to keep multiple files open.
5. Use Focus mode when you want a cleaner reading view.

## Keyboard Guide

Most important shortcuts:

- `Cmd+K` open command palette
- `Cmd+P` search files
- `Cmd+B` toggle left sidebar
- `Cmd+J` toggle table of contents
- `Cmd+.` toggle focus mode
- `Cmd+[` and `Cmd+]` move between tabs
- `Cmd+W` close the current tab
- `Cmd+F` search inside the current note
- `Cmd+,` open settings

If you forget a shortcut, press `?` to open the shortcut help panel.

## Library, Workspaces, and Tabs

### Library

The Library shows the folders you added and the Markdown files inside them.

- Use `+` to add a folder
- Right-click a folder to remove it or copy its path
- Search in the Library to filter files quickly

### Workspaces

Workspaces are saved sets of folders.

Use them when you want separate reading contexts, for example:

- personal notes
- work docs
- research folders
- writing projects

You can create, rename, switch, duplicate, and delete workspaces.

### Tabs

CalmPage can keep multiple Markdown files open at the same time.

This is useful when you want to compare files or move back and forth without losing your place.

## Focus Mode

Focus mode is for reading one article with fewer distractions.

It keeps the page quiet and lets you use:

- `Cmd+J` for table of contents
- `/` to search headings in focus mode
- arrow keys or `J` / `K` to move through content

## Settings

CalmPage has a reading and rendering settings area where you can tune:

- theme
- font size
- line height
- text width
- heading scale
- Markdown element styling

The goal is to let you shape the reading experience to match your own taste.

## Supported files

CalmPage scans common Markdown files, including:

- `.md`
- `.markdown`
- `.mdx`

## Development

```sh
npm install
npm run check
cargo check --manifest-path src-tauri/Cargo.toml
npm run tauri dev
```

## Build

```sh
npm run build
npm run tauri build
```

Build artifacts are written under:

```text
src-tauri/target/release/bundle/
```

## Project Docs

- [Requirements and plan](docs/requirements-and-plan.md)
- [Reading UI/UX redesign brief](docs/reading-ui-ux-redesign-brief.md)
- [Reader architecture plan](docs/reader-architecture-plan.md)
- [Workspace and command palette progress](docs/workspace-command-ui-progress.md)

