use winapi::shared::minwindef::DWORD;
use winapi::um::winreg::{RegCloseKey, RegEnumValueW, RegOpenKeyExW, HKEY_CURRENT_USER, HKEY_LOCAL_MACHINE};

const KEY_READ: DWORD = 0x20019;

pub fn get_system_families() -> Vec<String> {
    let mut families = Vec::new();
    unsafe {
        read_font_key(HKEY_LOCAL_MACHINE, &mut families);
        read_font_key(HKEY_CURRENT_USER, &mut families);
    }
    families.sort();
    families.dedup();
    families
}

unsafe fn read_font_key(root: winapi::shared::minwindef::HKEY, out: &mut Vec<String>) {
    let key_path: Vec<u16> = "SOFTWARE\\Microsoft\\Windows NT\\CurrentVersion\\Fonts\0"
        .encode_utf16()
        .collect();

    let mut hkey = std::ptr::null_mut();
    let res = RegOpenKeyExW(root, key_path.as_ptr(), 0, KEY_READ, &mut hkey);
    if res != 0 || hkey.is_null() {
        return;
    }

    let mut idx: DWORD = 0;
    loop {
        let mut name_buf = [0u16; 512];
        let mut name_len: DWORD = 512;

        let res = RegEnumValueW(
            hkey,
            idx,
            name_buf.as_mut_ptr(),
            &mut name_len,
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            std::ptr::null_mut(),
        );

        if res != 0 {
            break;
        }
        idx += 1;

        let len = (0..name_len as usize)
            .take_while(|&i| name_buf[i] != 0)
            .count();
        let raw = String::from_utf16_lossy(&name_buf[..len]).to_string();

        let display = raw
            .trim_end_matches(" (TrueType)")
            .trim_end_matches(" (OpenType)")
            .trim_end_matches(" (Type1)")
            .trim_end_matches(" (TrueType Collection)")
            .to_string();

        if !display.is_empty() && !out.contains(&display) {
            out.push(display);
        }
    }

    RegCloseKey(hkey);
}
