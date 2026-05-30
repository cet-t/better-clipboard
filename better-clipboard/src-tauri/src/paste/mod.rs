use std::ptr;

use anyhow::{bail, Result};
use winapi::shared::minwindef::{FALSE, TRUE};
use winapi::um::processthreadsapi::GetCurrentThreadId;
use winapi::um::winbase::{GlobalAlloc, GlobalLock, GlobalUnlock, GMEM_MOVEABLE};
use winapi::um::winuser::{
    AttachThreadInput, CloseClipboard, EmptyClipboard, GetFocus, GetForegroundWindow,
    GetWindowThreadProcessId, OpenClipboard, SendMessageA, SetClipboardData, CF_UNICODETEXT,
    WM_PASTE,
};

pub fn paste_text(text: &str) -> Result<()> {
    set_clipboard_text(text)?;
    log::info!("clipboard set, sending WM_PASTE");

    let hwnd = unsafe { GetForegroundWindow() };
    log::info!("foreground window: {:p}", hwnd);

    if hwnd.is_null() {
        bail!("No foreground window");
    }

    let focused = unsafe {
        let target_tid = GetWindowThreadProcessId(hwnd, ptr::null_mut());
        let current_tid = GetCurrentThreadId();
        AttachThreadInput(current_tid, target_tid, TRUE);
        let fg_focus = GetFocus();
        AttachThreadInput(current_tid, target_tid, FALSE);
        fg_focus
    };

    log::info!("focused control: {:p}", focused);

    if !focused.is_null() {
        unsafe { SendMessageA(focused, WM_PASTE, 0, 0) };
        log::info!("WM_PASTE sent to focused control");
    } else {
        unsafe { SendMessageA(hwnd, WM_PASTE, 0, 0) };
        log::info!("WM_PASTE sent to foreground window (fallback)");
    }
    Ok(())
}

fn set_clipboard_text(text: &str) -> Result<()> {
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
