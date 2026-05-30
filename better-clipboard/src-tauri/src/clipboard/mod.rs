use std::ptr;
use std::sync::mpsc::Sender;
use std::time::Duration;

use winapi::shared::minwindef::FALSE;
use winapi::um::winbase::{GlobalLock, GlobalUnlock};
use winapi::um::winuser::{
    CloseClipboard, EmptyClipboard, GetClipboardData, GetClipboardSequenceNumber,
    IsClipboardFormatAvailable, OpenClipboard, CF_UNICODETEXT,
};

#[allow(dead_code)]
pub enum ClipboardEvent {
    Text(String),
    Image(Vec<u8>),
}

pub fn read_current_text() -> Option<String> {
    unsafe {
        let opened = OpenClipboard(ptr::null_mut());
        if opened == FALSE {
            return None;
        }

        let available = IsClipboardFormatAvailable(CF_UNICODETEXT);
        if available == FALSE {
            CloseClipboard();
            return None;
        }

        let data = GetClipboardData(CF_UNICODETEXT);
        if data.is_null() {
            CloseClipboard();
            return None;
        }

        let ptr = GlobalLock(data) as *const u16;
        if ptr.is_null() {
            CloseClipboard();
            return None;
        }

        let len = (0..).take_while(|&i| *ptr.add(i) != 0).count();
        let slice = std::slice::from_raw_parts(ptr, len);
        let text = String::from_utf16(slice).unwrap_or_default();
        GlobalUnlock(data);
        CloseClipboard();

        if text.is_empty() { None } else { Some(text) }
    }
}

pub fn empty() {
    unsafe {
        let opened = OpenClipboard(ptr::null_mut());
        if opened != FALSE {
            EmptyClipboard();
            CloseClipboard();
        }
    }
}

pub fn start_monitoring(tx: Sender<ClipboardEvent>) {
    std::thread::spawn(move || {
        let mut last_seq = unsafe { GetClipboardSequenceNumber() };
        loop {
            std::thread::sleep(Duration::from_millis(200));
            let current_seq = unsafe { GetClipboardSequenceNumber() };
            if current_seq != last_seq {
                last_seq = current_seq;
                if let Some(text) = read_current_text() {
                    let _ = tx.send(ClipboardEvent::Text(text));
                }
            }
        }
    });
}
