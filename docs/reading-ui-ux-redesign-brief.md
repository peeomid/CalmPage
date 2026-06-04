# Reading UI/UX Redesign Brief

## Purpose

Design a better reading-first UI for Minimal Markdown Reader.

The current app has useful features, but the interface still feels like an app shell around a document. The target experience should feel closer to a premium reading workspace: calm, fast, keyboard-friendly, and focused on text.

This document is a handoff spec for a design/build agent.

## Product Direction

Minimal Markdown Reader is a macOS desktop app for reading many local Markdown files.

Primary goal:

- Make reading feel premium, calm, and distraction-free.

Secondary goals:

- Keep navigation fast for many files.
- Keep document search and heading navigation easy.
- Keep settings powerful but not visually noisy.
- Preserve web-native Markdown rendering so the same reader can later be reused on the web.

Non-goals for this redesign:

- Do not add Markdown editing.
- Do not turn the reader into a full IDE.
- Do not make the UI visually busy just because features exist.

## Current Tech Stack

Desktop shell:

- Tauri
- Rust backend
- macOS target first

Frontend:

- Svelte
- Static Markdown reader UI
- CSS variables for themes and typography

Backend responsibilities:

- Open one or more local folders.
- Scan Markdown files.
- Watch folders for file changes.
- Search file names and paths.
- Render selected Markdown to sanitized HTML.

Markdown rendering:

- Rust `pulldown-cmark`
- Sanitized with `ammonia`
- Frontend displays static HTML with `{@html currentNote.html}`

Important performance rules:

- Do not read every Markdown body on folder open.
- Do not put all document bodies into frontend state.
- Render only the selected note.
- File explorer should stay virtualized.
- Search and indexing should not block the main UI.
- Keep expensive work in Rust/backend worker tasks where possible.

## Core UX Problem

The user often wants to just read.

Current pain:

- Sidebar, top bar, TOC, settings, and buttons compete with the document.
- The document does not yet feel like the main visual object.
- Settings are useful but exposed in a way that can make the app feel tool-like.
- Reading mode and navigation mode are not clearly separated.

Design principle:

- Reading should be the default state.
- Navigation and settings should appear quickly when needed, then get out of the way.

## Desired Experience

When a document is open, the app should feel like this:

- The document is centered and visually dominant.
- Chrome is quiet.
- Sidebar can collapse.
- TOC can collapse.
- Top bar should be minimal and not steal attention.
- Search should feel like a reading aid, not a big tool panel.
- Keyboard shortcuts should make the user feel fast.

The app should support two mental modes:

### 1. Reading Mode

For focused reading.

Expected UI:

- Center document.
- No heavy sidebar.
- No always-loud toolbar.
- Optional slim top status.
- Optional hidden/peek TOC.
- Maximum typography quality.

### 2. Navigation Mode

For finding files, headings, or text.

Expected UI:

- Sidebar/file explorer visible.
- Command palette available.
- TOC/search visible.
- Settings can be opened.

The design should make switching between these modes simple.

## Recommended Layout

### Main App Shell

Use a three-region layout:

```text
Left rail / sidebar       Reader canvas             Right rail / TOC
File explorer             Markdown document         Headings / reading tools
```

But in focused reading, collapse to:

```text
Reader canvas only
```

### Left Sidebar

Purpose:

- Folder management.
- File explorer.
- Explorer filter.
- Recent/open files later.

Requirements:

- Collapsible.
- Default width around `280-320px`.
- Can be hidden with keyboard shortcut.
- File explorer search keeps folder structure.
- Command palette remains separate from explorer search.

Suggested shortcut:

- `Cmd+B`: toggle sidebar.
- `Cmd+Option+F`: focus explorer filter.

### Reader Canvas

Purpose:

- The main experience.

Requirements:

- Centered document.
- Strong typographic rhythm.
- Reader width controlled by typography setting.
- Top spacing should feel intentional.
- No accidental whole-window scrolling.
- Only reader pane should scroll.

Visual direction:

- Editorial, refined, not generic.
- Warm paper mode should feel like a high-quality article page.
- Dark mode should be graphite/warm-black, not pure black.
- Code/documentation mode can be cooler and sharper.

### Top Bar

Current top bar is too visually present.

Recommended redesign:

- Make it a slim floating or translucent toolbar.
- Hide non-essential controls by default.
- Show document title subtly.
- Put settings/actions behind one compact button.
- Keep top bar sticky but quiet.

Top bar should include:

- Current document title.
- Small file metadata only if useful.
- Search button/shortcut hint.
- Sidebar toggle.
- Focus mode toggle.
- Settings button.

Avoid:

- Large toolbar height.
- Many equal-weight buttons.
- Strong background that fights the document.

### Right TOC Rail

Purpose:

- Heading navigation.
- Heading search.
- Reading progress later.

Requirements:

- Sticky.
- Collapsible.
- Hidden in focus mode unless user asks.
- Search headings with `Cmd+Shift+O` and `/`.
- Active heading should be visible but subtle.

TOC should not feel like a second sidebar. It should be lighter than the file explorer.

Suggested design:

- Thin rail.
- Low-contrast heading list.
- Active heading with left border or soft background.
- Search field appears on focus, not always loud.

## Focus Mode

Add a true focus mode.

Purpose:

- Let the user read without file/sidebar/settings noise.

Behavior:

- Hide left sidebar.
- Hide right TOC rail.
- Make top bar minimal or auto-hide.
- Keep document centered.
- Keep `Cmd+K`, `Cmd+F`, `/`, and `Esc` usable.
- Exit focus mode with `Esc` or a visible small control.

Suggested shortcuts:

- `Cmd+.` or `Cmd+Shift+F`: toggle focus mode.
- `Esc`: close transient UI first; if nothing open, optionally exit focus mode.

Important:

- Focus mode should not block navigation. It should hide UI but keep keyboard access.

## Search UX

### In-Document Find

Current shortcut:

- `Cmd+F`

Expected behavior:

- Opens small find bar.
- Highlights all matches.
- Shows current match count, like `3/18`.
- `Enter`: next match.
- `Shift+Enter`: previous match.
- `Cmd+G`: next match.
- `Shift+Cmd+G`: previous match.
- `Esc`: close find.

Design recommendation:

- Use a compact floating find popover near the top right of reader, not a full-width bar.
- It should not push the document down.
- It should not move the top bar.
- It should scroll only the reader pane.

### Heading Search

Current shortcuts:

- `Cmd+Shift+O`
- `/` when not typing

Expected behavior:

- Focus TOC search.
- Filter headings only.
- Keep visible heading hierarchy if possible.
- Press `Enter` jumps to first/current heading match.

Design recommendation:

- Heading search can live in the TOC rail.
- In focus mode, it can appear as a small overlay command box.

### File Search

There should be two file search experiences:

#### Command Palette

Shortcut:

- `Cmd+K` / `Cmd+P`

Purpose:

- Fast flat jump to any file.

Behavior:

- Flat list.
- Search file title/path/root.
- Arrow keys and Enter.

#### Explorer Filter

Shortcut:

- `Cmd+Option+F`

Purpose:

- Filter the file tree but preserve folder structure.

Behavior:

- Parent folders remain visible if they contain matching files.
- Matching branches auto-expand.
- Clearing filter restores previous collapsed state.

## Settings UX

Settings should not feel like app clutter.

Current controls are useful, but they should move into a dedicated settings panel/sheet.

Recommended structure:

```text
Reader Settings
  Presets
  Typography
  Colors
  Markdown Elements
  Layout
  Shortcuts
```

### Presets

Keep quick presets:

- Editorial
- Notebook
- Technical
- Large
- Custom

Add future presets:

- Academic
- Compact
- Manuscript
- Night Journal
- Code Notes

Preset behavior:

- Clicking a preset updates all related settings.
- Changing a slider marks preset as `Custom`.
- Presets should be previewable quickly.

### Typography Controls

Current useful controls:

- Body size
- Line height
- Reader width
- H1 scale
- H2 scale
- H3 scale
- Paragraph gap
- Code size

Recommended additions:

- Body font family.
- Heading font family.
- UI font family.
- Font weight.
- Heading weight.
- Heading spacing before.
- Heading spacing after.
- List spacing.
- Blockquote size/spacing.
- Table font size.

Important design point:

- H1 should not be hardcoded huge.
- Use scale relative to body size.
- Default H1 should feel premium but not overpower long notes.

Recommended default ranges:

- Body size: `16-21px`.
- Line height: `1.45-1.75`.
- Reader width: `58-76ch`.
- H1 scale: `1.7-2.4`.
- H2 scale: `1.3-1.7`.
- H3 scale: `1.1-1.35`.

### Color Controls

Color should be semantic. That means colors are named by job, not by hex.

Current color presets:

- Paper
- Graphite
- Polar
- Sepia
- Midnight

Keep them, but improve token depth.

Color tokens should include:

- App background.
- Sidebar background.
- Reader background.
- Top bar background.
- Main text.
- Muted text.
- Faint text.
- Border.
- Accent.
- Link.
- Link hover.
- Selection.
- Find highlight.
- Active find highlight.
- Inline code text.
- Inline code background.
- Code block background.
- Code border.
- Blockquote background.
- Blockquote border.
- Table border.
- TOC active.
- File selected.
- File hover.
- Error.

Design point:

- Do not only change the whole app color.
- Small Markdown elements need their own subtle colors.
- Inline code should be readable and distinct without looking like an error badge.
- Search highlight should be obvious but not ugly.

### Markdown Element Styling

The reader should make Markdown look intentionally designed.

Elements to style carefully:

- H1, H2, H3.
- Paragraphs.
- Lists.
- Links.
- Inline code.
- Code blocks.
- Blockquotes.
- Tables.
- Images.
- Horizontal rules.
- Task lists.
- Callouts later.

Future optional features:

- Syntax highlighting.
- Obsidian-style callouts.
- KaTeX math.
- Mermaid diagrams.

## Keyboard Shortcuts

Current/recommended shortcuts:

```text
Cmd+K / Cmd+P       Command palette
Cmd+F               Find in current note
Cmd+G               Next find match
Shift+Cmd+G         Previous find match
Cmd+Shift+O         Search headings
/                   Focus heading search when not typing
Cmd+Option+F        Focus explorer filter
Cmd+B               Toggle sidebar
Cmd+Option+T        Toggle TOC rail
Esc                 Close current overlay/panel
```

Shortcut rules:

- Do not trigger app shortcuts while user is typing in an input, except intentional standard shortcuts like `Cmd+F`.
- More specific shortcuts should be checked before broad shortcuts.
- Example: `Cmd+Option+F` must not also trigger `Cmd+F`.

## Visual Design Direction

The app should not look like a default web dashboard.

Recommended visual language:

- Editorial.
- Quiet.
- Premium.
- Warm but sharp.
- Strong typography.
- Low visual noise.

Avoid:

- Generic SaaS dashboard look.
- Heavy cards everywhere.
- Purple gradient default AI style.
- Large button rows in the reading area.
- Too much dark contrast that makes text glow.

Good inspiration categories:

- High-quality longform article pages.
- Premium note apps.
- Technical documentation with great typography.
- Minimal writing apps, but with better file navigation.

## Design Requirements For Another Agent

When redesigning, the agent should produce:

1. A new app layout proposal.
2. A focus reading mode.
3. A collapsed/expanded sidebar design.
4. A quieter top bar.
5. A better find UI that does not push layout.
6. A better TOC rail.
7. A proper settings sheet/panel.
8. Refined typography presets.
9. Refined color presets with semantic tokens.
10. CSS implementation that preserves performance.

## Implementation Constraints

Keep:

- Tauri + Svelte.
- Static HTML Markdown rendering.
- Rust backend rendering/scanning.
- CSS variables for themes.
- Virtualized explorer rows.
- Keyboard-first navigation.

Do not:

- Add React.
- Add a heavy editor component.
- Add large UI libraries unless strongly justified.
- Move Markdown rendering to a slow frontend-only pipeline.
- Break folder watching or multi-folder support.

## Acceptance Criteria

Reading experience:

- User can hide side UI and read with minimal distraction.
- Reader text feels like the main object.
- Top bar does not compete with document.
- Typography looks good for long documents.
- H1 is not too large by default.

Navigation:

- User can open file quickly with `Cmd+K`.
- User can filter explorer with `Cmd+Option+F`.
- User can search current note with `Cmd+F`.
- User can search headings with `Cmd+Shift+O` or `/`.

Settings:

- User can quickly switch presets.
- User can tune typography details.
- User can tune color theme at preset level.
- Manual typography changes show as `Custom`.

Performance:

- Opening large folders should not freeze the UI.
- Typing in search inputs should feel immediate.
- Reading scroll should remain smooth.
- UI overlays should not cause layout jumps.

## Suggested Redesign Phases

### Phase 1: Reading Focus

- Add focus mode.
- Collapse sidebars.
- Quiet top bar.
- Floating find UI.

### Phase 2: Navigation Polish

- Better command palette.
- Better explorer filter.
- Better TOC rail and heading search.

### Phase 3: Settings System

- Dedicated settings panel.
- Typography controls.
- Color tokens.
- Preset preview.

### Phase 4: Premium Markdown Styling

- Improve Markdown element styling.
- Add callouts.
- Add syntax highlighting.
- Improve tables and images.

### Phase 5: Visual QA

- Test with long prose note.
- Test with technical note.
- Test with many headings.
- Test light/dark/sepia.
- Test small window.
- Test keyboard-only flow.
