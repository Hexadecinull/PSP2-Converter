# PSP2 Converter

Convert PSP games into installable VPK files for PS Vita with HENkaku or Ensō custom firmware.

Supports ISO, CSO, ZSO, and EBOOT.PBP input formats. Games run through the Vita's native PSP hardware emulation layer — no Adrenaline required.

> **Notice:** Only convert games you own. This tool is for personal backup use only.

## Requirements

- PS Vita with HENkaku or Ensō custom firmware
- VitaShell or similar to install VPK files

## Usage

1. Open PSP2 Converter
2. Drop your PSP game file (ISO, CSO, ZSO, or EBOOT.PBP) onto the app, or click Browse
3. Optionally override the title or Title ID
4. Select an output directory
5. Click **Convert to VPK**
6. Transfer the generated `.vpk` to your Vita and install via VitaShell

## Supported Input Formats

| Format | Description |
|--------|-------------|
| ISO | Raw UMD disc image |
| CSO | Compressed ISO (zlib / LZ4) |
| ZSO | Compressed ISO (LZ4) |
| PBP | PSP executable package (EBOOT.PBP) |

## How It Works

1. If the input is compressed (CSO/ZSO), it is decompressed to raw ISO
2. Game metadata (title, Title ID, version) is read from the disc's internal PARAM.SFO
3. An EBOOT.PBP is built with the ISO as DATA.PSAR, compatible with the Vita's PSP emulator
4. A Vita-format param.sfo and livearea assets are generated
5. Everything is packaged into a `.vpk` ready to install

## Building

Requires Rust (stable) and Node.js (16+).

```sh
npm install
npm run tauri build
```

For development:

```sh
npm run tauri dev
```

## License

Apache-2.0 — see [LICENSE](LICENSE).

## Links

- GitHub: https://github.com/Hexadecinull/PSP2-Converter
