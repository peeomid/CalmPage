# Minimal Markdown Reader

A fast, read-first Markdown desktop app built with Tauri + Svelte.

Priority:

- Premium static Markdown rendering.
- Fast file explorer and `Cmd+K` / `Cmd+P` filename search.
- Background-safe folder scanning and live watch.
- No editing in the MVP.

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

## Notes

See [docs/requirements-and-plan.md](docs/requirements-and-plan.md) for the current requirements, UI plan, and architecture notes.

See [docs/reading-ui-ux-redesign-brief.md](docs/reading-ui-ux-redesign-brief.md) for the reading-first UI/UX redesign handoff brief.
