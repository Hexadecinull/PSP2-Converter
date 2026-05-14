# Contributing

## Prerequisites

- Rust (stable, via rustup)
- Node.js 16+
- Tauri CLI: `cargo install tauri-cli`
- On Linux: `libwebkit2gtk-4.0-dev`, `libssl-dev`, `libgtk-3-dev`, `libayatana-appindicator3-dev`

## Development

```sh
npm install
npm run tauri:dev
```

Hot-reload is active for the frontend. Rust changes require a full recompile.

## Building

```sh
npm run tauri:build
```

Outputs a platform-native installer under `src-tauri/target/release/bundle/`.

## Project Layout

```
PSP2-Converter/
├── index.html                  Entry point
├── src/
│   ├── main.js                 Frontend (vanilla JS)
│   └── style.css               Styles
└── src-tauri/
    ├── Cargo.toml
    ├── tauri.conf.json
    └── src/
        ├── main.rs             Tauri commands
        ├── error.rs            Error types
        └── converter/
            ├── mod.rs          Conversion orchestration
            ├── decompress.rs   CSO / ZSO decompressor
            ├── iso.rs          ISO metadata extractor
            ├── pbp.rs          PBP parser
            ├── sfo.rs          SFO parser + builder
            └── vpk.rs          VPK assembler
```

## Adding Format Support

Add a new match arm in `converter/mod.rs` and implement the decompressor or parser in its own file under `converter/`.

## Code Style

- No inline comments in source files
- `rustfmt` defaults for Rust
- Plain ES2021 vanilla JS for the frontend — no framework, no bundler plugins
