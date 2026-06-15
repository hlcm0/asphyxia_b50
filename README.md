# SDVX B50 Tool

Standalone SDVX 7 B50 PNG exporter for Asphyxia savedata and compatible private servers.

## Features

- Reads SDVX 7 records from `sdvx@asphyxia.db`.
- Can fetch SDVX 7 scores directly from compatible Eamuse/XRPC private servers.
- Scans `contents/data/others/music_db.xml` and `contents/data/music/**/jk_*.png`.
- Lists SDVX 7 player profiles and deduplicates profiles by `__refid`.
- Aggregates duplicate records for the same song and difficulty before ranking.
- Sorts B50 entries by version 7 `volforce`.
- Uses local jacket images as data URLs, with an internal placeholder when no jacket exists.
- Supports custom B50 background images for both preview and exported PNG.
- Export layout uses 5 cards per row by default, or chooses a row count from the selected background image aspect ratio.

## Usage

1. Select the game data folder, usually `contents/data`.
2. Choose a score source.
3. For local savedata, select the Asphyxia savedata folder, scan players, then select a player.
4. For cloud server scores, enter the server URL, card ID, optional password, and optional PCBID, then fetch B50.
5. Optionally select a background image, preview the B50, then export PNG.

The app stores selected paths in `sdvx-b50-tool.settings.json` next to the executable or `.app` bundle. On startup, if saved paths are empty, it also tries to auto-detect:

- Game data: `./data`, then `./contents/data`
- Savedata: `./savedata`, then `./asphyxia/savedata`

On Windows, exported PNGs default to the executable directory. On macOS, they default next to the `.app` bundle, not inside the bundle.

## Development

```bash
npm install
npm run tauri:dev
```

## Portable Build

Build the native executable for the current platform:

```bash
npm run tauri:build:portable
```

On macOS, the raw executable is generated under `src-tauri/target/release/`.

Build a macOS `.app` bundle:

```bash
npm run tauri:bundle:mac
```

The macOS app bundle is generated under:

```text
src-tauri/target/release/bundle/macos/SDVX B50 Tool.app
```

Build a Windows x64 portable `.exe` from macOS:

```bash
npm run tauri:build:win
```

The Windows executable is generated under:

```text
src-tauri/target/x86_64-pc-windows-msvc/release/sdvx-b50-tool.exe
```

Release builds are generated automatically by GitHub Actions when a version tag is pushed.
The workflow uploads platform artifacts for macOS and Windows.

Tag names should use the `vX.Y.Z` format.

Rust/Cargo is required to build. End users only need the generated executable and the system WebView2 Runtime, which is normally present on Windows 10/11.

### Windows WebView2 data directory error

If Windows shows "Microsoft Edge cannot read and write to its data directory"
for a path like:

```text
C:\Users\<user>\AppData\Local\net.local.sdvx-b50-tool\EBWebView
```

close all running copies of the app and rename or delete:

```text
C:\Users\<user>\AppData\Local\net.local.sdvx-b50-tool
```

This folder only contains WebView2 cache/storage, not game data or exported B50
images. Newer builds probe the WebView2 data directory before creating the
window, and fall back to a writable temporary directory when the normal local
app data path is unavailable.

## Supported Data

- Game data folder: `contents/data`
- Savedata folder: `asphyxia/savedata`
- Database file: `sdvx@asphyxia.db`
- Cloud score source: SDVX 7 Eamuse/XRPC private servers compatible with `game.sv7_load_m`
- Version: SDVX 7 only

The app does not calculate or export SDVX 6 B50.

## B50 Rules

- Only records with `collection: "profile"` or `collection: "music"` and `version: 7` are used.
- For duplicate records with the same song and difficulty, the app keeps the best clear lamp, best score, and best grade, then recomputes the VF used for ranking.
- Different difficulties of the same song can appear independently.
- Jacket selection uses `target = music.type + 1`; from available `jk_<mid>_<n>.png` files, it chooses the largest `n <= target`, otherwise the smallest available jacket for the song.
- If no jacket exists, the app uses an embedded placeholder instead of a missing image URL.
