use std::sync::mpsc::Sender;

pub enum ClipboardEvent {
    Text(String),
    Image(Vec<u8>),
}

pub fn read_current_text() -> Option<String> {
    #[cfg(target_os = "windows")]
    {
        use std::ptr;
        use winapi::shared::minwindef::FALSE;
        use winapi::um::winbase::{GlobalLock, GlobalUnlock};
        use winapi::um::winuser::{
            CloseClipboard, GetClipboardData, IsClipboardFormatAvailable,
            OpenClipboard, CF_UNICODETEXT,
        };

        let opened = unsafe { OpenClipboard(ptr::null_mut()) };
        if opened == FALSE {
            return None;
        }

        let available = unsafe { IsClipboardFormatAvailable(CF_UNICODETEXT) };
        if available == FALSE {
            unsafe { CloseClipboard() };
            return None;
        }

        let data = unsafe { GetClipboardData(CF_UNICODETEXT) };
        if data.is_null() {
            unsafe { CloseClipboard() };
            return None;
        }

        let ptr = unsafe { GlobalLock(data) } as *const u16;
        if ptr.is_null() {
            unsafe { CloseClipboard() };
            return None;
        }

        let len = (0..).take_while(|&i| unsafe { *ptr.add(i) } != 0).count();
        let slice = unsafe { std::slice::from_raw_parts(ptr, len) };
        let text = String::from_utf16(slice).unwrap_or_default();
        unsafe { GlobalUnlock(data) };
        unsafe { CloseClipboard() };

        if text.is_empty() { None } else { Some(text) }
    }

    #[cfg(not(target_os = "windows"))]
    {
        None
    }
}

#[cfg_attr(not(target_os = "windows"), allow(unused_variables))]
pub fn start_monitoring(tx: Sender<ClipboardEvent>) {
    #[cfg(target_os = "windows")]
    monitor_windows(tx);

    #[cfg(not(target_os = "windows"))]
    {
        let _ = tx;
        log::warn!("Clipboard monitoring not yet supported on this platform");
    }
}

#[cfg(target_os = "windows")]
fn monitor_windows(tx: Sender<ClipboardEvent>) {
    std::thread::spawn(move || {
        use std::ptr;
        use winapi::shared::minwindef::FALSE;
        use winapi::um::winbase::{GlobalLock, GlobalUnlock};
        use winapi::um::winuser::{
            CloseClipboard, GetClipboardData, GetClipboardSequenceNumber,
            IsClipboardFormatAvailable, OpenClipboard, CF_UNICODETEXT,
        };

        let mut last_seq = unsafe { GetClipboardSequenceNumber() };
        loop {
            std::thread::sleep(std::time::Duration::from_millis(200));
            let current_seq = unsafe { GetClipboardSequenceNumber() };
            if current_seq != last_seq {
                last_seq = current_seq;
                let opened = unsafe { OpenClipboard(ptr::null_mut()) };
                if opened != FALSE {
                    let available = unsafe { IsClipboardFormatAvailable(CF_UNICODETEXT) };
                    if available != FALSE {
                        let data = unsafe { GetClipboardData(CF_UNICODETEXT) };
                        if !data.is_null() {
                            let ptr = unsafe { GlobalLock(data) } as *const u16;
                            if !ptr.is_null() {
                                let len = (0..).take_while(|&i| unsafe { *ptr.add(i) } != 0).count();
                                let slice = unsafe { std::slice::from_raw_parts(ptr, len) };
                                if let Ok(s) = String::from_utf16(slice) {
                                    let _ = tx.send(ClipboardEvent::Text(s));
                                }
                                unsafe { GlobalUnlock(data) };
                            }
                        }
                    }
                    unsafe { CloseClipboard() };
                }
            }
        }
    });
}
