# ram-widget

A tiny desktop widget that shows how much RAM is in use. Like the memory view
in Task Manager, without the weight.

- One pill-shaped window, 170 by 36 pixels: `12.2/31.9 GB  38%` and a thin bar.
- Follows the OS light or dark theme.
- Tray icon with a live tooltip, so the window can stay hidden.
- Optional start at login. Off by default.
- Written in Rust. No runtime, no installer, one executable.

Target: under 20 MB of RAM idle, near-zero CPU. Windows first, then macOS and
Linux.

## Status

Early. Builds for Windows and Linux. Not yet verified live on Windows, see
[PLAN.md](PLAN.md) for the phase log and open risks.

## Install

Windows: download `ram-widget.exe` from the [latest release](https://github.com/ABandelaria/ram-widget/releases) and run it.
It is not code-signed yet, so SmartScreen will warn on first run.

Or build from source (below), then run the executable:

- Windows: `target\x86_64-pc-windows-gnu\release\ram-widget.exe`
- Linux: `target/release/ram-widget`

Right-click the tray icon for the menu: **Show widget**, **Start at login**,
**Quit**. Drag the pill anywhere to move it.

Linux needs `libxkbcommon-x11-0` at runtime and a desktop with a system tray
that speaks the StatusNotifier (KSNI) protocol. KDE, and GNOME with the
AppIndicator extension, work.

## Build from source

Requires a Rust toolchain from <https://rustup.rs>.

```sh
cargo build --release
```

### Cross-compile for Windows from Linux or WSL

```sh
sudo apt install mingw-w64          # Debian/Ubuntu
rustup target add x86_64-pc-windows-gnu
cargo build --release --target x86_64-pc-windows-gnu
```

`.cargo/config.toml` already points the Windows target at the MinGW linker.

## Development

Layout:

| Path | What |
|------|------|
| `src/main.rs` | event loop, tray, timer, autostart |
| `ui/main.slint` | the pill window |
| `build.rs` | compiles the `.slint` file |
| `PLAN.md` | phase plan and decisions |

Crates: `sysinfo` (memory), `slint` (window, software renderer),
`tray-icon` (tray), `auto-launch` (start at login).

```sh
cargo test
cargo build
```

The debug `target/` folder grows to several GB across two targets. `cargo clean`
removes it; the next build is slow again.

### Contributing

Personal learning project, but fixes and ideas are welcome. Open an issue
first for anything bigger than a small fix. Keep changes small and shippable,
one phase at a time, and measure idle RSS after any UI change.

## License

[MIT](LICENSE)
