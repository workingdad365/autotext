use std::thread;
use std::time::{Duration, Instant};

use windows::Win32::UI::Input::KeyboardAndMouse::{
    GetAsyncKeyState, INPUT, INPUT_0, INPUT_KEYBOARD, KEYBD_EVENT_FLAGS, KEYBDINPUT,
    KEYEVENTF_KEYUP, KEYEVENTF_UNICODE, SendInput, VIRTUAL_KEY, VK_CONTROL, VK_LWIN, VK_MENU,
    VK_RETURN, VK_RWIN, VK_SHIFT, VK_TAB,
};

use crate::clipboard;
use crate::config::InputMode;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyAction {
    /// UTF-16 코드 유닛 직접 입력
    Unicode(u16),
    /// 가상 키 입력 (줄바꿈, 탭)
    Vk(u16),
}

/// 텍스트를 키 입력 동작 목록으로 변환
pub fn build_actions(text: &str) -> Vec<KeyAction> {
    let mut actions = Vec::with_capacity(text.len());
    let mut chars = text.chars().peekable();
    while let Some(c) = chars.next() {
        match c {
            '\r' => {
                if chars.peek() != Some(&'\n') {
                    actions.push(KeyAction::Vk(VK_RETURN.0));
                }
            }
            '\n' => actions.push(KeyAction::Vk(VK_RETURN.0)),
            '\t' => actions.push(KeyAction::Vk(VK_TAB.0)),
            _ => {
                let mut buf = [0u16; 2];
                for unit in c.encode_utf16(&mut buf) {
                    actions.push(KeyAction::Unicode(*unit));
                }
            }
        }
    }
    actions
}

fn key_input(vk: u16, scan: u16, flags: KEYBD_EVENT_FLAGS) -> INPUT {
    INPUT {
        r#type: INPUT_KEYBOARD,
        Anonymous: INPUT_0 {
            ki: KEYBDINPUT {
                wVk: VIRTUAL_KEY(vk),
                wScan: scan,
                dwFlags: flags,
                time: 0,
                dwExtraInfo: 0,
            },
        },
    }
}

fn to_inputs(actions: &[KeyAction]) -> Vec<INPUT> {
    let mut inputs = Vec::with_capacity(actions.len() * 2);
    for a in actions {
        match *a {
            KeyAction::Unicode(u) => {
                inputs.push(key_input(0, u, KEYEVENTF_UNICODE));
                inputs.push(key_input(0, u, KEYEVENTF_UNICODE | KEYEVENTF_KEYUP));
            }
            KeyAction::Vk(vk) => {
                inputs.push(key_input(vk, 0, KEYBD_EVENT_FLAGS(0)));
                inputs.push(key_input(vk, 0, KEYEVENTF_KEYUP));
            }
        }
    }
    inputs
}

fn is_down(vk: i32) -> bool {
    unsafe { GetAsyncKeyState(vk) < 0 }
}

/// 핫키 조합 키가 모두 떼어질 때까지 대기 (최대 timeout)
fn wait_keys_released(trigger_vk: u32, timeout: Duration) {
    let keys = [
        VK_LWIN.0 as i32,
        VK_RWIN.0 as i32,
        VK_CONTROL.0 as i32,
        VK_MENU.0 as i32,
        VK_SHIFT.0 as i32,
        trigger_vk as i32,
    ];
    let start = Instant::now();
    while start.elapsed() < timeout {
        if !keys.iter().any(|&k| is_down(k)) {
            return;
        }
        thread::sleep(Duration::from_millis(10));
    }
}

/// 현재 포커스 창에 상용구 입력
pub fn insert_text(text: &str, trigger_vk: u32, mode: InputMode) {
    // 수식키가 눌린 채로 입력하면 Ctrl+문자 등으로 해석되므로 해제를 기다림
    wait_keys_released(trigger_vk, Duration::from_secs(3));
    thread::sleep(Duration::from_millis(30));

    match mode {
        InputMode::Type => type_text(text),
        InputMode::Paste => {
            if !paste_text(text) {
                // 클립보드를 사용할 수 없으면 직접 입력으로 대체
                type_text(text);
            }
        }
    }
}

fn paste_text(text: &str) -> bool {
    let backup = clipboard::backup();
    let Some(seq) = clipboard::set_text(text) else {
        if let Some(b) = &backup {
            clipboard::restore(b);
        }
        return false;
    };

    let ctrl = VK_CONTROL.0;
    let v = b'V' as u16;
    let inputs = [
        key_input(ctrl, 0, KEYBD_EVENT_FLAGS(0)),
        key_input(v, 0, KEYBD_EVENT_FLAGS(0)),
        key_input(v, 0, KEYEVENTF_KEYUP),
        key_input(ctrl, 0, KEYEVENTF_KEYUP),
    ];
    unsafe {
        SendInput(&inputs, size_of::<INPUT>() as i32);
    }

    // 대상 프로그램이 클립보드를 읽을 시간을 준 뒤 원래 내용 복원
    thread::sleep(Duration::from_millis(500));
    if let Some(b) = backup
        && clipboard::sequence() == seq
    {
        // 그사이 사용자가 새로 복사했다면 덮어쓰지 않음
        clipboard::restore(&b);
    }
    true
}

fn type_text(text: &str) {
    let inputs = to_inputs(&build_actions(text));
    // 긴 텍스트에서 입력 누락을 줄이기 위해 나누어 전송
    for chunk in inputs.chunks(400) {
        unsafe {
            SendInput(chunk, size_of::<INPUT>() as i32);
        }
        thread::sleep(Duration::from_millis(5));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ascii_and_korean() {
        let a = build_actions("Hi 가");
        assert_eq!(
            a,
            vec![
                KeyAction::Unicode('H' as u16),
                KeyAction::Unicode('i' as u16),
                KeyAction::Unicode(' ' as u16),
                KeyAction::Unicode(0xAC00),
            ]
        );
    }

    #[test]
    fn newline_and_tab() {
        let ret = KeyAction::Vk(VK_RETURN.0);
        let tab = KeyAction::Vk(VK_TAB.0);
        assert_eq!(build_actions("a\nb"), vec![KeyAction::Unicode(97), ret, KeyAction::Unicode(98)]);
        assert_eq!(build_actions("a\r\nb"), vec![KeyAction::Unicode(97), ret, KeyAction::Unicode(98)]);
        assert_eq!(build_actions("\r"), vec![ret]);
        assert_eq!(build_actions("\t"), vec![tab]);
    }

    #[test]
    fn surrogate_pair() {
        // U+1F600 은 서로게이트 쌍 2 유닛으로 분리됨
        assert_eq!(
            build_actions("\u{1F600}"),
            vec![KeyAction::Unicode(0xD83D), KeyAction::Unicode(0xDE00)]
        );
    }

    #[test]
    fn inputs_are_down_up_pairs() {
        let inputs = to_inputs(&build_actions("a\n"));
        assert_eq!(inputs.len(), 4);
        unsafe {
            assert_eq!(inputs[0].Anonymous.ki.dwFlags, KEYEVENTF_UNICODE);
            assert_eq!(inputs[1].Anonymous.ki.dwFlags, KEYEVENTF_UNICODE | KEYEVENTF_KEYUP);
            assert_eq!(inputs[2].Anonymous.ki.wVk, VK_RETURN);
            assert_eq!(inputs[3].Anonymous.ki.dwFlags, KEYEVENTF_KEYUP);
        }
    }
}
