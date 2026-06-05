# Reader Architecture Plan

## Goal

Build Scratch around a fast reader-first Markdown surface.

Editing stays available, but TipTap should not be the default reading surface.
Focus mode should never depend on TipTap.

## Why Change

The current `Editor` component uses TipTap for both reading and editing. That
means reading inherits editor behavior: editable DOM, editor keyboard handling,
selection plugins, format toolbar state, and ProseMirror decorations.

For reading, this is too much. It makes simple features like `Esc` to exit focus
mode harder than they should be.

## Target Modes

### Reader Mode

Default when opening an existing note.

- Markdown renders to sanitized HTML.
- No `contenteditable`.
- Search uses normal DOM marks.
- TOC is built from rendered headings.
- Focus mode uses this mode only.
- Fast path for large files.

### Editor Mode

Only when the user explicitly edits.

- TipTap remains here.
- Slash commands, wikilinks, tables, image paste, math editing, formatting bar,
  and auto-save stay in this mode.
- Switching back to reader mode re-renders from saved/current markdown.

### Source Mode

Plain textarea markdown editing can remain as a separate mode.

## Reusable Renderer Package

Create a DOM-independent renderer module:

```ts
renderMarkdownToHtml(markdown, config) => {
  html,
  headings,
  text,
}
```

The module must not import React, Tauri, or browser-only APIs. This keeps it
extractable later for a CLI:

```bash
scratch-render input.md --theme graphite --font newsreader --out output.html
```

### Renderer Responsibilities

- Convert Markdown to HTML.
- Sanitize unsafe HTML/scripts.
- Generate stable heading IDs.
- Return heading metadata:

```ts
{
  id: string,
  level: 1 | 2 | 3 | 4 | 5 | 6,
  text: string,
}
```

- Return plain text for search indexing.
- Support config values for theme/typography, but keep CSS generation separate
  where possible.

### Renderer Non-Goals

- No app state.
- No folder scanning.
- No search UI.
- No React components.

## ReaderView Component

Create a React component around the renderer:

```tsx
<ReaderView
  markdown={currentNote.content}
  config={readerConfig}
  focusMode={focusMode}
  onExitFocusMode={exitFocusMode}
/>
```

### ReaderView Responsibilities

- Render sanitized HTML.
- Show transparent floating TOC only when headings exist.
- Hide TOC if no document is open or document has no headings.
- Search current document with `Cmd+F` or `/` in focus mode.
- Highlight matches with `<mark>`.
- Move active match with `Enter` / `Shift+Enter`.
- Keep `Esc` behavior predictable:
  - if search open: close search
  - else if focus mode: exit focus mode
- Provide a visible focus mode exit control.

## Layout Rules

- Focus mode must own layout state.
- While focus mode is active, left sidebar and TOC toggles should be ignored or
  queued until focus mode exits.
- Do not let sidebar toggle mutate layout in focus mode.
- TOC should not reserve fixed right-column space by default. It should be a
  floating rail over transparent background.

## Migration Steps

1. Add renderer core module.
2. Add `ReaderView` component.
3. Integrate `ReaderView` for existing-note read mode.
4. Keep new/empty notes opening in editor mode.
5. Add explicit Edit button/shortcut to enter TipTap.
6. Move focus mode to always use `ReaderView`.
7. Remove TipTap search from reader path.
8. Add CLI wrapper around renderer later.

## Previous Bugs To Fix In Migration

- Focus mode cannot be exited.
- `/` search does not work in focus mode.
- Sidebar toggle during focus mode breaks layout.
- Empty right TOC panel appears before opening a note.
- Bullet/task-list rendering shows unwanted checkbox/box artifacts.

## Acceptance Checks

- Existing note opens in reader mode.
- `Esc` exits focus mode from any focused element except an open search box,
  where first `Esc` closes search and second `Esc` exits focus.
- `Command+.` toggles focus mode on and off.
- `/` in focus mode opens in-document search.
- No right TOC UI when no note is open.
- No right TOC UI when note has no headings.
- TOC overlays/floats and does not reserve permanent width.
- Sidebar toggle has no layout effect during focus mode.
- `npm run build` passes.
- `cargo test` passes.
- Built `.app` launches.

