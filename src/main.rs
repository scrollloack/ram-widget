#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::time::Duration;

use auto_launch::AutoLaunchBuilder;
use slint::{ComponentHandle, PhysicalPosition, Timer, TimerMode};
use sysinfo::System;
use tray_icon::menu::{CheckMenuItem, Menu, MenuEvent, MenuItem};
use tray_icon::{Icon, TrayIconBuilder};

slint::include_modules!();

const GB: f64 = 1024.0 * 1024.0 * 1024.0;
const MINIMIZED: &str = "--minimized";

/// `("12.2/31.9 GB", 38)`
fn stats(used: u64, total: u64) -> (String, i32) {
    let pct = (used as f64 / total as f64 * 100.0).round() as i32;
    (format!("{:.1}/{:.1} GB", used as f64 / GB, total as f64 / GB), pct)
}

// shortcut: solid 16x16 square in the accent color, upgrade to a drawn icon when the look matters
fn icon() -> Icon {
    let rgba = [0x00, 0x78, 0xD4, 0xFF].repeat(16 * 16);
    Icon::from_rgba(rgba, 16, 16).expect("icon buffer is 16x16 RGBA")
}

#[cfg(windows)]
fn windows_setup() -> bool {
    use windows_sys::Win32::Foundation::{ERROR_ALREADY_EXISTS, GetLastError};
    use windows_sys::Win32::System::Threading::CreateMutexW;
    use windows_sys::Win32::UI::WindowsAndMessaging::{MB_ICONERROR, MessageBoxW};
    use windows_sys::w;

    // the exe has no console in release, so show panics in a box instead of dying silently
    #[cfg(not(debug_assertions))]
    std::panic::set_hook(Box::new(|info| {
        let msg: Vec<u16> = info.to_string().encode_utf16().chain([0]).collect();
        unsafe { MessageBoxW(std::ptr::null_mut(), msg.as_ptr(), w!("RAM widget"), MB_ICONERROR) };
    }));

    // one widget per session: the mutex handle is leaked on purpose and dies with the process
    unsafe {
        CreateMutexW(std::ptr::null(), 0, w!("ram-widget"));
        if GetLastError() == ERROR_ALREADY_EXISTS {
            MessageBoxW(std::ptr::null_mut(), w!("RAM widget is already running. Look in the tray."), w!("RAM widget"), MB_ICONERROR);
            return false;
        }
    }
    true
}

fn main() {
    #[cfg(windows)]
    if !windows_setup() {
        return;
    }

    let ui = Main::new().expect("create window");

    let weak = ui.as_weak();
    ui.on_drag(move |dx, dy| {
        let win = weak.unwrap();
        let win = win.window();
        let s = win.scale_factor();
        let p = win.position();
        win.set_position(PhysicalPosition::new(p.x + (dx * s) as i32, p.y + (dy * s) as i32));
    });

    let exe = std::env::current_exe().expect("own exe path");
    let exe = exe.to_string_lossy();
    // auto-launch writes the Run key value unquoted, so a space in the path would split it
    let exe = if cfg!(windows) { format!("\"{exe}\"") } else { exe.into_owned() };
    let autostart = AutoLaunchBuilder::new()
        .set_app_name("ram-widget")
        .set_app_path(&exe)
        .set_args(&[MINIMIZED])
        .build()
        .expect("autostart config");
    // the stored path goes stale when the exe is moved, so point it at the running exe
    if autostart.is_enabled().unwrap_or(false) {
        let _ = autostart.enable();
    }
    let minimized = std::env::args().any(|a| a == MINIMIZED);

    let show = CheckMenuItem::new("Show widget", true, !minimized, None);
    let login = CheckMenuItem::new("Start at login", true, autostart.is_enabled().unwrap_or(false), None);
    let quit = MenuItem::new("Quit", true, None);
    let menu = Menu::new();
    menu.append_items(&[&show, &login, &quit]).expect("build menu");

    let tray = TrayIconBuilder::new()
        .with_menu(Box::new(menu))
        .with_tooltip("RAM")
        .with_icon(icon())
        .build()
        .expect("create tray icon");

    // Menu events may arrive off the main thread (Linux), so hop onto the Slint loop.
    // Only Send data crosses: ids, the weak window handle and the autostart config.
    let (show_id, login_id, quit_id) = (show.id().clone(), login.id().clone(), quit.id().clone());
    let weak = ui.as_weak();
    MenuEvent::set_event_handler(Some(move |e: MenuEvent| {
        let (show_id, login_id, quit_id, autostart) =
            (show_id.clone(), login_id.clone(), quit_id.clone(), autostart.clone());
        let _ = weak.upgrade_in_event_loop(move |ui| {
            if e.id == quit_id {
                let _ = slint::quit_event_loop();
            } else if e.id == show_id {
                let win = ui.window();
                let _ = if win.is_visible() { win.hide() } else { win.show() };
            } else if e.id == login_id {
                // the menu already flipped the check mark; a failure here leaves it wrong until restart
                let _ = match autostart.is_enabled() {
                    Ok(true) => autostart.disable(),
                    _ => autostart.enable(),
                };
            }
        });
    }));

    let mut sys = System::new().expect("sysinfo: unsupported platform");
    let weak = ui.as_weak();
    let timer = Timer::default();
    let mut last_tip = String::new();
    timer.start(TimerMode::Repeated, Duration::from_secs(1), move || {
        sys.refresh_memory();
        let (text, pct) = stats(sys.used_memory(), sys.total_memory());
        let tip = format!("{text}  {pct}%");
        if tip != last_tip {
            let _ = tray.set_tooltip(Some(&tip));
            last_tip = tip;
        }
        if let Some(ui) = weak.upgrade() {
            ui.set_text(text.into());
            ui.set_percent(pct);
        }
    });

    if !minimized {
        ui.show().expect("show window");
    }
    slint::run_event_loop_until_quit().expect("event loop");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stats_formats_gb_and_rounds_percent() {
        assert_eq!(stats(12_241_000_000, 34_250_000_000), ("11.4/31.9 GB".to_string(), 36));
    }
}
