//! 클립보드 백업, 텍스트 설정, 복원

use std::thread;
use std::time::Duration;

use windows::Win32::Foundation::{GlobalFree, HANDLE, HGLOBAL};
use windows::Win32::System::DataExchange::{
    CloseClipboard, EmptyClipboard, EnumClipboardFormats, GetClipboardData,
    GetClipboardSequenceNumber, OpenClipboard, RegisterClipboardFormatW, SetClipboardData,
};
use windows::Win32::System::Memory::{
    GMEM_MOVEABLE, GlobalAlloc, GlobalLock, GlobalSize, GlobalUnlock,
};
use windows::core::w;

const CF_UNICODETEXT: u32 = 13;
/// 백업 대상 총 용량 상한 (초과 시 이후 형식은 건너뜀)
const MAX_BACKUP_BYTES: usize = 64 * 1024 * 1024;

/// 클립보드에 있던 형식별 원본 데이터
#[derive(Debug, Default)]
pub struct Backup {
    items: Vec<(u32, Vec<u8>)>,
}

/// HGLOBAL 로 복사할 수 없는 형식 (GDI 핸들, 소유자 그리기, 프로세스 전용 핸들)
pub fn is_copyable_format(fmt: u32) -> bool {
    !matches!(
        fmt,
        2 | 3 | 9 | 14 // CF_BITMAP, CF_METAFILEPICT, CF_PALETTE, CF_ENHMETAFILE
            | 0x80 | 0x82 | 0x83 | 0x8E // CF_OWNERDISPLAY, CF_DSPBITMAP, CF_DSPMETAFILEPICT, CF_DSPENHMETAFILE
            | 0x200..=0x3FF // CF_PRIVATEFIRST..CF_GDIOBJLAST
    )
}

/// Windows 앱에서 줄바꿈이 깨지지 않도록 CRLF 로 통일
pub fn to_crlf(text: &str) -> String {
    text.replace("\r\n", "\n").replace('\n', "\r\n")
}

struct Opened;

impl Opened {
    fn open() -> Option<Opened> {
        // 다른 프로그램이 클립보드를 열고 있으면 잠시 재시도
        for _ in 0..50 {
            if unsafe { OpenClipboard(None) }.is_ok() {
                return Some(Opened);
            }
            thread::sleep(Duration::from_millis(10));
        }
        None
    }
}

impl Drop for Opened {
    fn drop(&mut self) {
        unsafe {
            let _ = CloseClipboard();
        }
    }
}

fn alloc_global(bytes: &[u8]) -> Option<HGLOBAL> {
    unsafe {
        let h = GlobalAlloc(GMEM_MOVEABLE, bytes.len().max(1)).ok()?;
        let p = GlobalLock(h) as *mut u8;
        if p.is_null() {
            let _ = GlobalFree(Some(h));
            return None;
        }
        std::ptr::copy_nonoverlapping(bytes.as_ptr(), p, bytes.len());
        let _ = GlobalUnlock(h);
        Some(h)
    }
}

/// 열린 클립보드에 데이터 설정 (성공 시 메모리 소유권은 시스템으로 넘어감)
fn put(fmt: u32, bytes: &[u8]) -> bool {
    let Some(h) = alloc_global(bytes) else { return false };
    unsafe {
        if SetClipboardData(fmt, Some(HANDLE(h.0))).is_ok() {
            true
        } else {
            let _ = GlobalFree(Some(h));
            false
        }
    }
}

/// Win+V 클립보드 기록과 클라우드 동기화에서 제외
fn mark_exclude_from_history() {
    let zero = 0u32.to_ne_bytes();
    unsafe {
        for name in [w!("CanIncludeInClipboardHistory"), w!("CanUploadToCloudClipboard")] {
            let fmt = RegisterClipboardFormatW(name);
            if fmt != 0 {
                put(fmt, &zero);
            }
        }
    }
}

pub fn backup() -> Option<Backup> {
    let _guard = Opened::open()?;
    let mut items = Vec::new();
    let mut total = 0usize;
    let mut fmt = 0u32;
    loop {
        fmt = unsafe { EnumClipboardFormats(fmt) };
        if fmt == 0 {
            break;
        }
        if !is_copyable_format(fmt) {
            continue;
        }
        unsafe {
            let Ok(h) = GetClipboardData(fmt) else { continue };
            let hg = HGLOBAL(h.0);
            let size = GlobalSize(hg);
            if size == 0 || total + size > MAX_BACKUP_BYTES {
                continue;
            }
            let p = GlobalLock(hg) as *const u8;
            if p.is_null() {
                continue;
            }
            let data = std::slice::from_raw_parts(p, size).to_vec();
            let _ = GlobalUnlock(hg);
            total += size;
            items.push((fmt, data));
        }
    }
    Some(Backup { items })
}

/// 텍스트를 클립보드에 설정하고 설정 직후의 시퀀스 번호 반환
pub fn set_text(text: &str) -> Option<u32> {
    let wide: Vec<u16> = to_crlf(text).encode_utf16().chain(std::iter::once(0)).collect();
    let bytes: Vec<u8> = wide.iter().flat_map(|u| u.to_ne_bytes()).collect();
    {
        let _guard = Opened::open()?;
        unsafe { EmptyClipboard().ok()? };
        if !put(CF_UNICODETEXT, &bytes) {
            return None;
        }
        mark_exclude_from_history();
    }
    Some(sequence())
}

pub fn restore(backup: &Backup) {
    let Some(_guard) = Opened::open() else { return };
    unsafe {
        if EmptyClipboard().is_err() {
            return;
        }
    }
    for (fmt, data) in &backup.items {
        put(*fmt, data);
    }
    // 복원한 내용이 기록에 중복으로 쌓이지 않도록 함
    if !backup.items.is_empty() {
        mark_exclude_from_history();
    }
}

pub fn sequence() -> u32 {
    unsafe { GetClipboardSequenceNumber() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn crlf_normalization() {
        assert_eq!(to_crlf("a\nb"), "a\r\nb");
        assert_eq!(to_crlf("a\r\nb"), "a\r\nb");
        assert_eq!(to_crlf("a\n\nb\r\n"), "a\r\n\r\nb\r\n");
        assert_eq!(to_crlf("abc"), "abc");
    }

    #[test]
    fn copyable_formats() {
        assert!(is_copyable_format(1)); // CF_TEXT
        assert!(is_copyable_format(8)); // CF_DIB
        assert!(is_copyable_format(13)); // CF_UNICODETEXT
        assert!(is_copyable_format(15)); // CF_HDROP
        assert!(is_copyable_format(17)); // CF_DIBV5
        assert!(is_copyable_format(0xC123)); // 등록 형식
        assert!(!is_copyable_format(2)); // CF_BITMAP
        assert!(!is_copyable_format(14)); // CF_ENHMETAFILE
        assert!(!is_copyable_format(0x250));
        assert!(!is_copyable_format(0x300));
    }
}
