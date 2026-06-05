# Workspace, Command Palette, And UI Progress

Owner: Codex
Date: 2026-06-05

## Goal

Bring the real app closer to `docs/ui-options/prototype-combined.html`, while correcting the workspace concept.

## Concepts

- Library: folders and files inside the active workspace.
- Workspace: saved group of folders.
- Folder management belongs in Library.
- Workspace rail page lists saved workspaces.
- Command palette should support grouped results and prefix modes.

## Phase 1: Workspace And Library

Status: Complete

Tasks:
- [x] Rename rail mode from `workspace` to `workspaces`.
- [x] Add `WorkspaceEntry` state saved in `localStorage`.
- [x] Show saved workspace list in the Workspaces rail page.
- [x] Add New Workspace action.
- [x] Add workspace switch action.
- [x] Add workspace context menu: rename, duplicate, delete.
- [x] Move folder management into Library.
- [x] Add top-right `+` add-folder button in Library header.
- [x] Remove `Open Workspace` folder-scanning button.
- [x] Show active workspace root folders as normal top-level rows in the Library file list.
- [x] Right-click root folder rows, or use their `...` menu, to copy path or remove from Library.
- [x] Filter visible file tree by active workspace folders.

Implementation notes:
- Phase 1 is frontend-local. Workspaces are stored in `localStorage` under `minimal-reader:workspaces`.
- Switching workspace filters the Library tree only. It does not close open tabs.
- Folder removal still uses backend `remove_vault`, then workspace root IDs are normalized.
- The separate Library folder card strip was removed; root folders now live inside the file tree.

Review gate:
- [x] `npm run check` — 0 errors, 11 existing accessibility warnings.
- Manual check: create workspace, switch workspace, add folder, remove folder.

## Phase 2: Floating Reader Controls

Status: Complete

Tasks:
- [x] Add floating `Aa`, `Focus ⌘.`, `TOC ⌘J` pills inside reader frame.
- [x] `Aa` opens Settings Studio at Appearance.
- [x] `Focus ⌘.` toggles focus mode.
- [x] `TOC ⌘J` toggles TOC.
- [x] Remove duplicate topbar Focus button.
- [x] Verify focus mode still hides rail/sidebar and exits only with `Cmd+Period`.

Implementation notes:
- Floating controls are placed in `.reader-frame`, outside `.reader-topbar`.
- Floating controls are hidden while focus mode is active, so focus mode keeps the keyboard-only exit rule.
- Removed the old floating `Exit Focus` button because it allowed mouse exit from focus mode.

Review gate:
- [x] `npm run check` — 0 errors, 11 existing accessibility warnings.
- Manual check: each floating control works in normal mode; focus mode hides rail/sidebar and exits with `Cmd+Period`.

## Phase 3: Command Palette

Status: Complete

Tasks:
- [x] Add palette modes: smart, actions, files, tabs, headings, settings, workspaces.
- [x] Add prefixes:
  - no prefix: smart grouped search
  - `>` actions
  - `/` files
  - `@` open tabs
  - `#` headings
  - `?` settings
  - `:` workspaces
- [x] Add grouped rendering:
  - Open Tabs
  - Files
  - Headings
  - Actions
  - Settings
  - Workspaces
- [x] Add shortcuts:
  - `⌘K`: smart
  - `⌘P`: files
  - `⌘O`: open tabs
  - `⇧⌘K`: actions
  - `⌘⇧O`: headings
- [x] Keep:
  - `⌘F`: document search
  - `⇧⌘F`: sidebar filter

Implementation notes:
- Palette results are built as groups, but keyboard selection uses only selectable rows. Group headers are never selected.
- `Enter` runs the selected row for commands, files, open tabs, headings, settings, and workspaces.
- Settings rows open Settings Studio at the chosen section.
- Workspace rows switch the active workspace.

Review gate:
- [x] `npm run check` — 0 errors, 11 existing accessibility warnings.
- Manual check: keyboard selection skips group headers, Enter runs selected row.

## Phase 4: Settings Preview

Status: Complete

Tasks:
- [x] Add Appearance preview pane.
- [x] Preview heading, paragraph, quote, code block.
- [x] Preview uses current reader variables.
- [x] Keep system presets read-only.
- [x] Keep custom preset save/update.

Review gate:
- [x] `npm run check`
- [x] Code review: preview uses `readerStyle`, so slider-bound reader variables update preview and reader together.

## Final

Status: Complete

Tasks:
- [x] Run `npm run check`.
- [x] Run `npm run tauri build`.
- [x] Install `/Applications/Minimal Markdown Reader.app`.
- [x] Add Trove build log.

Result:
- Installed app version: `0.1.0`.
- Check result: 0 errors, 11 existing accessibility warnings.

## Current UX Fixes

Status: Complete

Tasks:
- [x] Move folder management into Library file listing; remove the extra top folder-management section.
- [x] Move floating reader controls to the bottom; keep only Focus and TOC.
- [x] Fix `Cmd+J` TOC toggle in focus mode.
- [x] Replace New Workspace prompt with inline workspace creation.
- [x] Make Settings open as a full-window studio instead of a narrow drawer.
- [x] Make tabs dynamic width with a max width.
- [x] Make Settings rail button toggle closed when clicked again.
- [x] Improve command palette group visual hierarchy.

Review gates:
- [x] Issue 1 subagent review and `npm run check`.
- [x] Issue 2 subagent review and `npm run check`.
- [x] Issue 3 subagent review and `npm run check`.
- [x] Issue 4 subagent review and `npm run check`.
- [x] Issue 5 subagent review and `npm run check`.
- [x] Issue 6 local review and `npm run check`.
- [x] Issue 7 subagent review and `npm run check`.
- [x] Final `npm run check`.
- [x] Final `npm run tauri build`.
- [x] Install updated app into `/Applications`.

## Follow-up UX Fixes

Status: Complete

Tasks:
- [x] Move floating Focus/TOC controls to the right.
- [x] Make floating controls low-visibility while reading; reveal on hover/focus/focus mode.
- [x] Hide TOC by default when entering focus mode.
- [x] Allow Default Workspace rename through inline rename UI.
- [x] New Workspace no longer auto-switches or jumps to Library.
- [x] Duplicate Workspace no longer auto-switches.
- [x] Add/Open folder now includes roots in the active workspace instead of overwriting root list.
- [x] Command palette active row scrolls into view during keyboard navigation.

Review gates:
- [x] `npm run check` — 0 errors, 11 existing warnings.
- [x] `npm run tauri build`.
- [x] Install updated app into `/Applications`.
