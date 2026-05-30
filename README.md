# Better Clipboard

Lightweight clipboard manager for Windows.

## Stack

- **Framework**: Tauri 2 (Rust + Web UI)
- **Frontend**: HTML / CSS / TypeScript (Vite)
- **Backend**: Rust
- **DB**: SQLite (rusqlite)

## Features

### Implemented

- **Global hotkey** (`Alt+C`) to toggle overlay
- **Clipboard monitoring** with polling (text, UTF-16)
- **Overlay UI** — dark theme, entry list, keyboard selection (`asdfjkl;`)
- **Paste via WM_PASTE** — sends paste command to focused control with `AttachThreadInput`
- **Deduplication** — SHA-256 content hash, duplicates move to top
- **SQLite persistence** — optional DB storage with configurable path and max entries (default 100)
- **Settings window** — hotkey config, persistence mode, DB path, max entries, font, clear operations
- **System tray** — Settings, Restart, Quit
- **Multi-language** — English / Japanese via external JSON locale files (`serde_json`)
- **Escape to close** overlay (via Win32 `GetAsyncKeyState` polling on Windows)

### Not yet implemented

- Image clipboard monitoring & thumbnail generation
- Source app name detection
- Pinning entries
- Linux support
- Application icon
- Error handling improvements

## Build

```bash
cd better-clipboard
npm install
npm run build           # frontend
cd src-tauri
cargo tauri dev         # development
cargo tauri build       # production
```

## Usage

1. App starts in the system tray (no window)
2. Press `Alt+C` to open the clipboard overlay
3. Press `a`-`;` to select an entry and paste it into the focused app
4. Press `Esc` or click outside to close the overlay
5. Right-click the tray icon for Settings / Restart / Quit

## Config

Stored at `%LOCALAPPDATA%/BetterClipboard/config.toml`.

```toml
[hotkeys]
overlay = "alt+c"
select_keys = "asdfjkl;"

persistence = "db"          # "session" | "db"

[db]
path = "..."                # default: %LOCALAPPDATA%/BetterClipboard/content.db

max_entries = 100
font_family = ""            # empty = system default
locale = ""                 # "" = auto-detect, "en", "ja"
```
