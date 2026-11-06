//! Win32 공용 파일 열기/저장 대화상자

use std::path::PathBuf;

use windows::Win32::Foundation::HWND;
use windows::Win32::UI::Controls::Dialogs::{
    GetOpenFileNameW, GetSaveFileNameW, OFN_EXPLORER, OFN_FILEMUSTEXIST, OFN_NOCHANGEDIR, OFN_OVERWRITEPROMPT,
    OFN_PATHMUSTEXIST, OPEN_FILENAME_FLAGS, OPENFILENAMEW,
};
use windows::core::{HSTRING, PCWSTR, PWSTR, w};

const FILE_BUF_LEN: usize = 4096;

fn wide_nul(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(std::iter::once(0)).collect()
}

/// 이름\0패턴\0 ... \0 형식의 필터 문자열
fn json_filter() -> Vec<u16> {
    let mut v = Vec::new();
    for part in ["AutoText 상용구 (*.json)", "*.json", "모든 파일 (*.*)", "*.*"] {
        v.extend(wide_nul(part));
    }
    v.push(0);
    v
}

fn run(owner: isize, title: &str, default_name: &str, save: bool) -> Option<PathBuf> {
    let filter = json_filter();
    let title = HSTRING::from(title);
    let mut buf = vec![0u16; FILE_BUF_LEN];
    let name: Vec<u16> = default_name.encode_utf16().take(FILE_BUF_LEN - 1).collect();
    buf[..name.len()].copy_from_slice(&name);

    let mut flags: OPEN_FILENAME_FLAGS = OFN_EXPLORER | OFN_NOCHANGEDIR | OFN_PATHMUSTEXIST;
    flags |= if save { OFN_OVERWRITEPROMPT } else { OFN_FILEMUSTEXIST };

    let mut ofn = OPENFILENAMEW {
        lStructSize: size_of::<OPENFILENAMEW>() as u32,
        hwndOwner: HWND(owner as *mut _),
        lpstrFilter: PCWSTR(filter.as_ptr()),
        nFilterIndex: 1,
        lpstrFile: PWSTR(buf.as_mut_ptr()),
        nMaxFile: buf.len() as u32,
        lpstrTitle: PCWSTR(title.as_ptr()),
        lpstrDefExt: w!("json"),
        Flags: flags,
        ..Default::default()
    };
    let ok = unsafe { if save { GetSaveFileNameW(&mut ofn) } else { GetOpenFileNameW(&mut ofn) } };
    if !ok.as_bool() {
        return None;
    }
    let len = buf.iter().position(|&c| c == 0).unwrap_or(buf.len());
    Some(PathBuf::from(String::from_utf16_lossy(&buf[..len])))
}

/// 취소하면 None
pub fn open_json(owner: isize, title: &str) -> Option<PathBuf> {
    run(owner, title, "", false)
}

/// 취소하면 None. 같은 이름의 파일이 있으면 덮어쓰기 확인을 받음
pub fn save_json(owner: isize, title: &str, default_name: &str) -> Option<PathBuf> {
    run(owner, title, default_name, true)
}
