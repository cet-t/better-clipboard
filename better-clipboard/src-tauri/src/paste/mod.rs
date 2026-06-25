use std::mem;
use std::ptr;

use anyhow::{bail, Result};
use winapi::shared::minwindef::FALSE;
use winapi::um::winbase::{GlobalAlloc, GlobalLock, GlobalUnlock, GMEM_MOVEABLE};
use winapi::um::winuser::{
    CloseClipboard, EmptyClipboard, OpenClipboard, SendInput, SetClipboardData, CF_UNICODETEXT,
    INPUT, INPUT_KEYBOARD, KEYBDINPUT, KEYEVENTF_KEYUP, VK_CONTROL,
};

const VK_V: u16 = 0x56;

pub fn paste_text(text: &str) -> Result<()> {
    set_clipboard_text(text)?;
    log::info!("clipboard set, sending Ctrl+V via SendInput");

    send_ctrl_v()?;
    log::info!("Ctrl+V sent");
    Ok(())
}

fn key_input(vk: u16, key_up: bool) -> INPUT {
    let mut input: INPUT = unsafe { mem::zeroed() };
    input.type_ = INPUT_KEYBOARD;
    let ki = KEYBDINPUT {
        wVk: vk,
        wScan: 0,
        dwFlags: if key_up { KEYEVENTF_KEYUP } else { 0 },
        time: 0,
        dwExtraInfo: 0,
    };
    unsafe {
        *input.u.ki_mut() = ki;
    }
    input
}

fn send_ctrl_v() -> Result<()> {
    let mut inputs = [
        key_input(VK_CONTROL as u16, false),
        key_input(VK_V, false),
        key_input(VK_V, true),
        key_input(VK_CONTROL as u16, true),
    ];

    let sent = unsafe {
        SendInput(
            inputs.len() as u32,
            inputs.as_mut_ptr(),
            mem::size_of::<INPUT>() as i32,
        )
    };

    if sent != inputs.len() as u32 {
        bail!("SendInput failed: sent {} of {} events", sent, inputs.len());
    }
    Ok(())
}

pub fn set_clipboard_text(text: &str) -> Result<()> {
    let opened = unsafe { OpenClipboard(ptr::null_mut()) };
    if opened == FALSE {
        bail!("Failed to open clipboard");
    }

    unsafe { EmptyClipboard() };

    let utf16: Vec<u16> = text.encode_utf16().chain(std::iter::once(0)).collect();
    let bytes = utf16.len() * 2;

    let h_mem = unsafe { GlobalAlloc(GMEM_MOVEABLE, bytes) };
    if h_mem.is_null() {
        unsafe { CloseClipboard() };
        bail!("Failed to allocate global memory");
    }

    let p_dest = unsafe { GlobalLock(h_mem) } as *mut u16;
    if p_dest.is_null() {
        unsafe { GlobalUnlock(h_mem) };
        unsafe { CloseClipboard() };
        bail!("Failed to lock global memory");
    }

    unsafe {
        ptr::copy_nonoverlapping(utf16.as_ptr(), p_dest, utf16.len());
        GlobalUnlock(h_mem);
        SetClipboardData(CF_UNICODETEXT, h_mem as _);
        CloseClipboard();
    }

    Ok(())
}
