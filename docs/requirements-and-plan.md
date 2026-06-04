# Minimal Markdown Reader Requirements And Plan

## Requirements

- Build a macOS desktop Markdown reader using Tauri + Svelte.
- Prioritize premium typography and static reading quality.
- Render Markdown as static HTML for read mode; no editing in MVP.
- Open a local folder and list Markdown files.
- Open multiple local folders and list Markdown files together.
- Save opened folders and reopen them on app restart.
- Keep folder/file listing fast for many files.
- Add `Cmd+K` / `Cmd+P` command palette for quick file-name search.
- Add `Cmd+F` find-in-note for the currently opened document.
- Add `Cmd+G` / `Shift+Cmd+G` next/previous find match.
- Add heading search inside the sticky table of contents.
- Search file names/paths only in MVP; no full-content search yet.
- Index/scan in the backend so UI remains responsive.
- Watch the folder live so newly added Markdown files appear in explorer and search.
- Support dark/light reading themes.

## Architecture

```text
Tauri Rust backend
  -> vault path guard
  -> multi-root Markdown path scanner
  -> live watcher
  -> file-name search
  -> selected note Markdown -> static HTML

Svelte frontend
  -> app shell
  -> file explorer
  -> command palette
  -> reader pane
  -> theme controls
```

## Performance Rules

- Do not read every Markdown body on folder open.
- Do not store all note bodies in frontend state.
- Do not render Markdown through a rich editor for reading.
- Return small search result sets.
- Keep watcher updates scoped to file metadata.
- Render only the selected note.
- Sanitize rendered HTML before sending it to the WebView.
- Use a strict CSP instead of disabling browser protections.
- Guard async note/search responses so stale results cannot overwrite newer user actions.
- Keep virtualized rows at a fixed height so scrolling does not drift.

## UI Plan

- Left sidebar: folder picker, scan status, file explorer.
- Left sidebar: add-folder button.
- Center reader: static HTML with editorial typography.
- Top reader bar: current note title, theme toggle, command palette hint.
- Find bar: `Cmd+F`, highlighted matches, match count, next/previous.
- Palette overlay: file search input and top matching files.
- Right rail: sticky TOC and heading search.
- Empty state: invite user to open a folder.

## Typography Direction

Default preset: Editorial.

- Body: `Newsreader`, `Iowan Old Style`, Georgia, serif.
- UI: `Avenir Next`, `Inter`, system sans.
- Code: `SF Mono`, JetBrains Mono, monospace.
- Reader width: 62-78 characters depending on preset.
- Font size: 16-20px depending on preset.
- Line height: 1.58-1.72 depending on preset.
- Warm paper background in light mode.
- Deep graphite background in dark mode.

Current reader presets:

- Editorial: warm book-like prose, 18px, 1.68 line height, 66ch.
- Notebook: calmer long-note reading, 17px, 1.62 line height, 72ch.
- Technical: sharper sans layout for docs/code/tables, 16px, 1.58 line height, 78ch.
- Large: relaxed bigger reading, 20px, 1.72 line height, 62ch.

Future theme system:

- Keep typography presets separate from color themes.
- Add color presets:
  - Paper
  - Graphite
  - Polar
  - Sepia
  - Midnight
- Store selected typography preset and color theme in local storage.
- Expose custom controls later for font size, line height, width, and font family.
- Keep all render styling as CSS variables so the same renderer can be reused in a web app.

## Reading Experience Improvement Plan

### Phase 1: Reader Quality Foundation

- Add configurable typography presets.
- Add configurable color presets.
- Use CSS variables for body font, heading font, size, line height, measure, and weight.
- Improve dark mode contrast with graphite surfaces and warm text.
- Improve light mode paper feel without making text washed out.
- Improve blockquote, code, table, link, highlight, image, and selection styles.
- Keep render output static HTML for speed.

### Phase 2: Navigation For Long Notes

- Add sticky table of contents when a note has at least 3 headings.
- Generate heading links in the frontend from rendered HTML.
- Track active heading with `IntersectionObserver`.
- Add heading filter input in the TOC rail.
- Hide TOC on small screens.
- Keep scrolling inside the reader pane, not the full app shell.

### Phase 2B: Keyboard Reading Flow

- `Cmd+F`: open find bar for current note.
- `Enter`: next match.
- `Shift+Enter`: previous match.
- `Cmd+G`: next match.
- `Shift+Cmd+G`: previous match.
- `Escape`: close find or command palette.
- Show match count as current/total.
- Highlight all matches and emphasize active match.

### Phase 3: Configurable Theme System

- Add a proper settings panel with preset cards.
- Persist selected preset.
- Persist selected color preset.
- Later add manual sliders:
  - font size
  - line height
  - reader width
  - paragraph spacing
  - font family
- Later add import/export of preset JSON so typography can be tested quickly.

### Phase 4: Premium Markdown Features

- Add syntax highlighting with a high-quality theme.
- Add optional KaTeX for math notes.
- Add optional Mermaid rendering for diagrams.
- Add Obsidian-style callout blocks.
- Add backlink/wiki-link rendering only after core reading remains fast.

### Phase 5: Performance Guardrails

- Keep file scanning and Markdown rendering in Rust backend worker tasks.
- Keep file explorer virtualized.
- Do not run full-content search on every keypress in the UI thread.
- For future full-text search, use a backend search index instead of filtering all note bodies in Svelte.
- Keep command palette result count bounded.
- Add profiling before adding editor features.

## MVP Deliverables

- Tauri + Svelte project scaffold.
- Folder open.
- Virtualized Markdown folder tree.
- Multiple folder support.
- Saved folders reopen on app start.
- Folder tree collapsed by default after opening folders.
- Static Markdown rendering.
- Premium light/dark reader CSS.
- Configurable reader typography presets.
- Configurable color presets.
- Sticky table of contents for long notes.
- Heading search in sticky table of contents.
- `Cmd+F` in-note search with highlights.
- `Cmd+K` / `Cmd+P` file-name palette.
- Live watcher update with debounce.
- Incremental watcher handling for normal added/changed/deleted Markdown files.
- Sanitized Markdown HTML.
- Bounded in-memory render cache.
- Last opened folder and theme persistence.
- Build/check passing.

## Current Implementation Notes

- Backend scans file path metadata only; note bodies are read on selection.
- Filename/path search uses a precomputed lowercase search key.
- Markdown rendering uses `pulldown-cmark` and sanitizes output with `ammonia`.
- Watch events debounce before refreshing the vault snapshot.
- Normal file-level watch events update only affected paths; directory-level/fallback events trigger full refresh.
- Reader view uses static HTML/CSS, not an editor model.
- Command palette supports arrow keys and `Enter`.
- Command palette supports `Escape` while the input is focused.
- Last folder and theme are saved in local storage for a smoother app restart.
- Reader typography preset is saved in local storage.
