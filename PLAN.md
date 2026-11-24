# ram-widget: plan

Personal project. A small, light RAM monitor widget. Rust. Windows first, then macOS and Linux (Debian, Arch, RHEL).

Also a Rust learning project. Keep every phase small and shippable.

## Goal

- Show total RAM capacity and current RAM usage in percent, live (1s refresh).
- Lighter than Task Manager: target under 20 MB RSS, near-zero CPU.
- Three delivery forms, built in order: CLI, tray icon, small always-on-top window.
- Option to start at login, off by default.

## Why Rust

- The GUI toolkit decides RAM cost, not the language. Rust has the smallest cross-platform GUI options.
- `sysinfo` crate reads memory on all three OS with one API.
- No runtime, no GC. Single static exe, no installer.

Rejected: Go (GUI needs cgo or webview, 40 MB+ idle), C++ (Qt is heavy, or three native codebases), Bend (GPU research language, no GUI, no Windows story).

## Dev environment

- WSL2 Ubuntu. Windows is the first run target.
- Repo lives on the WSL filesystem: `/home/bandelaria/dev/ram-widget`. Not under `/mnt/c` (cargo is 5 to 10x slower there).
- Cross-compile to `x86_64-pc-windows-gnu` from WSL. Run the `.exe` straight from the WSL shell; interop launches it on Windows. No Windows-side Rust install.
- Fallback if a crate refuses the gnu target: `cargo-xwin` for the MSVC target from WSL.

## Crate list

| Crate | Purpose | Phase |
|-------|---------|-------|
| `sysinfo` | total and used memory, all OS | 1 |
| `tray-icon` | system tray icon, tooltip, menu | 2 |
| ~~`tao`~~ | removed in Phase 3, Slint's winit loop pumps messages for the tray | 2 |
| `auto-launch` | start at login; registry Run key, LaunchAgent, XDG autostart | 2b |
| `slint` | small window; software renderer, winit backend | 3 |

egui/eframe skipped: fastest to write, but the GPU context costs 30 to 50 MB. Add only if a richer UI is wanted later.

## Phase 0: toolchain

1. `curl https://sh.rustup.rs -sSf | sh`, then restart the shell.
2. `sudo apt install mingw-w64 build-essential`
3. `rustup target add x86_64-pc-windows-gnu`
4. `cargo init` in this folder (not `cargo new`, folder already has this file).
5. Add `.cargo/config.toml`:

   ```toml
   [target.x86_64-pc-windows-gnu]
   linker = "x86_64-w64-mingw32-gcc"
   ```

6. `cargo build --target x86_64-pc-windows-gnu`
7. Run `./target/x86_64-pc-windows-gnu/debug/ram-widget.exe`. Hello world must appear.
   - **Blocked on this laptop (2026-11-24).** CrowdStrike Falcon blocks and quarantines any unsigned exe on first run, from the WSL path and from `C:\Users\...\Temp` alike. Build works, run does not. Until IT grants an exclusion or a personal Windows machine is used, develop and run the native Linux build (`cargo build`, `./target/debug/ram-widget`) and keep the Windows cross-build compiling.
8. `git init`, first commit.

## Phase 1: CLI

Proves the pipeline. First real Rust.

- Dep: `sysinfo`.
- `System::new()`, `refresh_memory()`, `total_memory()`, `used_memory()`. Values are bytes.
- Percent = used / total * 100.
- Print one line every second: `Total: 31.9 GB  Used: 12.2 GB  38.2%`. Ctrl+C exits.
- Done when the `.exe` matches Task Manager "In use" within 1%.

## Phase 2: tray icon

- Deps: `tray-icon`, `tao`.
- Tooltip: `12.2 / 31.9 GB  38%`.
- Right-click menu: Quit.
- A timer thread refreshes memory every 1s and sends the new text to the event loop.
- Icon: one static 16 px PNG embedded with `include_bytes!`. A redrawn percent icon is a later extra.
- Done when the tray entry updates live and Quit exits clean.

## Phase 2b: start at login

- Dep: `auto-launch`.
  - Windows: `HKCU\Software\Microsoft\Windows\CurrentVersion\Run`. No admin needed.
  - macOS: LaunchAgent plist in `~/Library/LaunchAgents`.
  - Linux: `.desktop` file in `~/.config/autostart`.
- Tray menu gets a check item: **Start at login**. Checked state comes from `is_enabled()` at startup, so no config file. Toggle calls `enable()` or `disable()`.
- Exe path from `std::env::current_exe()`.
- Login start passes an arg such as `--minimized` so only the tray opens, not the Phase 3 window.
- Default: off. User opts in.
- Known limit: the entry stores the absolute exe path. Moving or renaming the `.exe` breaks autostart silently. Re-toggling fixes it.

## Phase 3: small window

- Dep: `slint`, features `renderer-software` and `backend-winit`.
- Design chosen 2026-11-24: option A "Pill" from https://claude.ai/artifact/8pdmxazNkzskWLc9qzDS5r
  - 170x36 px, radius 8. One line: `12.2/31.9 GB` left, `38%` bold right. 3 px bar below in the accent color.
  - Colors follow the OS theme via Slint `Palette` (light `#F3F3F3`/`#1B1B1B`, dark `#202020`/`#FFFFFF`). Accent fixed `#0078D4` for now; reading the real OS accent is a later extra.
- One `.slint` file.
- Frameless, always on top, fixed size, draggable.
- Same 1s refresh via `slint::Timer`.
- Start at login stays in the tray menu only. The 170x36 pill has no room for a checkbox.
- Done 2026-11-25. Slint owns the event loop, `tao` removed. Menu events hop onto the Slint loop via `upgrade_in_event_loop`. Tray menu: Show widget, Start at login, Quit.
- Done when window and tray show the same numbers.

## Phase 3b: background transparency (optional, decision pending)

- A setting to make the pill background see-through (text and bar stay solid). Off by default.
- Needs `Window { background: transparent }` and a transparent winit window. Unverified with `renderer-software` on Windows; fallback is the Skia renderer, about 5 to 10 MB more RSS.
- Gate: measure idle RSS with and without. Build only if the cost is acceptable. User decides after seeing the numbers.

## Phase 4: cross-platform check (later)

- Native Linux build in WSL: `cargo build`. Tray needs `libgtk-3-dev` and `libxdo-dev` to link (installed 2026-11-24), (no longer needed since Phase 3: Linux tray uses the `ksni` backend, pure D-Bus). Runtime needs `libxkbcommon-x11-0`; winit was built X11-only, so WSLg Wayland alone does not work. WSLg has no tray host, so Linux tray testing needs a real desktop.
- macOS needs a Mac. No code change expected for the memory part.
- Per-distro packaging (deb, AUR, rpm) is out of scope until the app is stable.

## Order and rules

- 0 → 1 → 2 → 2b → 3 → 4. Each phase shippable. Commit per phase.
- Smallest change that works. No abstraction before the second use.
- Measure RSS after each GUI phase. If a phase pushes idle RSS over 20 MB, stop and find out why before adding more.

## Open risks

- Windows run target blocked by CrowdStrike on the work laptop. Phase 1 and later verify on Linux first. Task Manager comparison waits for a machine that can run the exe.

- `windows-gnu` target can hit linker issues in crates that assume MSVC. Fallback: `cargo-xwin`.
- Slint software renderer under `windows-gnu` not verified yet.
- `auto-launch` under a gnu cross-build not verified, but it is plain registry calls and should link clean.
- Any GUI window on Windows allocates a DWM surface. Under 10 MB holds only for the tray-only form.
