# Clipboard Manager — Plan

A terminal clipboard manager built with [ratatui](https://ratatui.rs). It records everything copied, sorts each item
into a category, and lets you search, pin, and re-copy past items. It uses a slate color scheme.

## Goals

- Clean structure: each module has one job and small, readable functions.
- Minimal and efficient: few dependencies, no redundant work per frame, bounded history.
- Slate design: Tailwind slate palette, rounded panels, one accent color per category.

## Architecture

```
watcher thread ──(mpsc: String)──▶ App ──▶ ui::draw ──▶ terminal
                                    │
                                    └──▶ History ──▶ history.json
```

| Module        | Responsibility                                                               |
| ------------- | ---------------------------------------------------------------------------- |
| `main.rs`     | Terminal setup and the event loop: draw, handle keys, drain clips, persist.  |
| `watcher.rs`  | Background thread that polls the system clipboard (`arboard`) for new text.  |
| `category.rs` | `Category` enum and heuristic detection (link, email, color, number, …).     |
| `history.rs`  | `Entry` and `History`: add with dedupe, 500-entry cap, pins, JSON load/save. |
| `app.rs`      | UI state: tabs, selection, search mode, key handling, filtered view.         |
| `ui.rs`       | Pure rendering: search bar, category sidebar, entry table, preview, footer.  |
| `theme.rs`    | Slate palette and per-category colors.                                       |

## Build steps

1. **Scaffold**: Cargo project with `ratatui`, `arboard`, `serde`, `serde_json`, and `dirs`.
2. **Categories**: implement `Category::detect`, ordered from most to least specific, and unit-test it.
3. **History**: entries stored newest first; re-copying moves an item to the top; the oldest unpinned
   entries are evicted past the limit; the history is saved to the OS data dir.
4. **Watcher**: poll the clipboard every 400 ms and send only changed, non-empty text over a channel.
5. **App state**: `Tab` filters (All, Pinned, each category), `Normal`/`Search` modes, and a cached
   `visible` index list that is recomputed only after a key press or a new clip.
6. **UI**: three-column layout with a search bar and a key-hint footer; snippet and preview lengths are capped so
   huge clips stay cheap to draw.
7. **Loop**: 250 ms tick so ages stay fresh; handle only key *press* events (Windows also reports releases);
   save only when the history changed.

## Conventions

- `cargo fmt` (compact settings in `rustfmt.toml`) and `cargo clippy --all-targets` must be clean.
- Rendering only updates the table's scroll state; all data changes go through `App`.
- Colors live only in `theme.rs`.

## Possible next steps

- Image clipboard support (arboard `image-data` feature).
- Event-driven clipboard listening instead of polling.
- Configurable history limit and keybindings.
- Render tests using ratatui's `TestBackend`.
