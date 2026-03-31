use anyhow::{Context, Result};

use crate::adb;
use crate::cli::{InputAction, InputArgs};

/// Map friendly key names to Android keyevent codes.
fn keycode_for(name: &str) -> Result<u32> {
    match name.to_lowercase().as_str() {
        "home" => Ok(3),
        "back" => Ok(4),
        "call" => Ok(5),
        "endcall" => Ok(6),
        "dpad_center" | "enter" => Ok(66),
        "menu" => Ok(82),
        "search" => Ok(84),
        "power" => Ok(26),
        "volup" | "volume_up" => Ok(24),
        "voldown" | "volume_down" => Ok(25),
        "tab" => Ok(61),
        "delete" | "backspace" => Ok(67),
        "recent" | "app_switch" => Ok(187),
        "camera" => Ok(27),
        _ => anyhow::bail!("Unknown key name: {name}. Use: home, back, enter, menu, power, volup, voldown, tab, delete, recent"),
    }
}

/// Send text input to the device.
pub fn input_text(text: &str) -> Result<()> {
    // Escape special characters for adb shell input
    let escaped = text.replace(' ', "%s").replace('&', "\\&").replace('<', "\\<").replace('>', "\\>");
    adb::shell_str(&format!("input text '{escaped}'"))
        .context("Failed to send text input")?;
    Ok(())
}

/// Send a tap at coordinates.
pub fn tap(x: u32, y: u32) -> Result<()> {
    adb::shell_str(&format!("input tap {x} {y}"))
        .context("Failed to send tap")?;
    Ok(())
}

/// Send a swipe gesture.
pub fn swipe(x1: u32, y1: u32, x2: u32, y2: u32, duration_ms: u32) -> Result<()> {
    adb::shell_str(&format!("input swipe {x1} {y1} {x2} {y2} {duration_ms}"))
        .context("Failed to send swipe")?;
    Ok(())
}

/// Send a key event.
pub fn key(name: &str) -> Result<()> {
    let code = keycode_for(name)?;
    adb::shell_str(&format!("input keyevent {code}"))
        .context("Failed to send key event")?;
    Ok(())
}

/// Push text to device clipboard via a broadcast.
pub fn set_clipboard(text: &str) -> Result<()> {
    // Use am broadcast with a clipboard helper, or service call
    // Simplest cross-version approach: use input via a temp field
    adb::shell_str(&format!(
        "am broadcast -a clipper.set -e text '{text}'"
    ))
    .context(
        "Failed to set clipboard. Consider installing Clipper app or using Android 10+ clipboard API",
    )?;
    Ok(())
}

/// CLI entry point.
pub async fn run(args: InputArgs) -> Result<()> {
    match args.action {
        InputAction::Text { value } => {
            input_text(&value)?;
            println!("Typed: {value}");
        }
        InputAction::Tap { x, y } => {
            tap(x, y)?;
            println!("Tapped at ({x}, {y})");
        }
        InputAction::Swipe {
            x1,
            y1,
            x2,
            y2,
            duration,
        } => {
            swipe(x1, y1, x2, y2, duration)?;
            println!("Swiped ({x1},{y1}) -> ({x2},{y2}) in {duration}ms");
        }
        InputAction::Key { name } => {
            key(&name)?;
            println!("Sent key: {name}");
        }
        InputAction::Clip { text } => {
            set_clipboard(&text)?;
            println!("Clipboard set");
        }
    }

    Ok(())
}
