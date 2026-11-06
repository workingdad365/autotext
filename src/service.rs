//! 트레이 아이콘과 전역 핫키를 담당하는 백그라운드 스레드

use std::cell::RefCell;
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::Ordering;
use std::sync::mpsc::Sender;
use std::thread;

use tray_icon::menu::{Menu, MenuEvent, MenuItem, PredefinedMenuItem};
use tray_icon::{Icon, MouseButton, MouseButtonState, TrayIcon, TrayIconBuilder, TrayIconEvent};
use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, WPARAM};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::UI::Input::KeyboardAndMouse::{
    MOD_ALT, MOD_CONTROL, MOD_NOREPEAT, MOD_SHIFT, MOD_WIN, RegisterHotKey, UnregisterHotKey,
};
use windows::Win32::UI::WindowsAndMessaging::{
    CreateWindowExW, DefWindowProcW, DispatchMessageW, GetMessageW, GetSystemMetrics, HWND_MESSAGE,
    IsIconic, MSG, RegisterClassW, SM_CXSMICON, SM_CYSMICON, SW_RESTORE, SW_SHOW,
    SetForegroundWindow, ShowWindow, TranslateMessage, WINDOW_EX_STYLE, WINDOW_STYLE, WM_APP,
    WM_HOTKEY, WNDCLASSW,
};
use windows::core::{PCWSTR, w};

use crate::config::{APP_NAME, APP_VERSION, InputMode, key_to_vk};
use crate::{MainCmd, Shared, input};

pub const SERVICE_CLASS: PCWSTR = w!("AutoText.ServiceWindow");
/// 설정 변경 후 핫키 재등록 요청
pub const WM_APP_RELOAD: u32 = WM_APP + 1;
/// 설정 창 열기 요청 (중복 실행 시 기존 인스턴스에 전달)
pub const WM_APP_SHOW: u32 = WM_APP + 2;

struct State {
    shared: Arc<Shared>,
    tx: Sender<MainCmd>,
    hwnd: HWND,
    /// 등록 id -> (상용구 내용, 트리거 가상 키, 입력 방식)
    hotkeys: HashMap<i32, (String, u32, InputMode)>,
}

thread_local! {
    static STATE: RefCell<Option<State>> = const { RefCell::new(None) };
}

pub fn spawn(shared: Arc<Shared>, tx: Sender<MainCmd>) {
    thread::Builder::new()
        .name("autotext-service".into())
        .spawn(move || run(shared, tx))
        .expect("서비스 스레드 생성 실패");
}

fn run(shared: Arc<Shared>, tx: Sender<MainCmd>) {
    let hwnd = match create_message_window() {
        Ok(h) => h,
        Err(e) => {
            eprintln!("service window creation failed: {e}");
            std::process::exit(1);
        }
    };
    shared.service_hwnd.store(hwnd.0 as isize, Ordering::SeqCst);

    let open_item = MenuItem::new("설정 열기", true, None);
    let quit_item = MenuItem::new("종료", true, None);
    let menu = Menu::new();
    let _ = menu.append_items(&[&open_item, &PredefinedMenuItem::separator(), &quit_item]);

    let tray = build_tray(menu);

    STATE.with(|s| {
        *s.borrow_mut() = Some(State { shared, tx, hwnd, hotkeys: HashMap::new() });
    });
    reload_hotkeys();

    let mut msg = MSG::default();
    loop {
        let ret = unsafe { GetMessageW(&mut msg, None, 0, 0) };
        if ret.0 <= 0 {
            break;
        }
        unsafe {
            let _ = TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }

        while let Ok(ev) = TrayIconEvent::receiver().try_recv() {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = ev
            {
                open_settings();
            }
        }
        while let Ok(ev) = MenuEvent::receiver().try_recv() {
            if ev.id == *open_item.id() {
                open_settings();
            } else if ev.id == *quit_item.id() {
                quit(tray);
            }
        }
    }
    quit(tray);
}

fn build_tray(menu: Menu) -> Option<TrayIcon> {
    let size = unsafe {
        (GetSystemMetrics(SM_CXSMICON) as u32, GetSystemMetrics(SM_CYSMICON) as u32)
    };
    let icon = Icon::from_resource(1, Some(size))
        .or_else(|_| Icon::from_resource(1, None))
        .ok();
    let mut builder = TrayIconBuilder::new()
        .with_menu(Box::new(menu))
        .with_menu_on_left_click(false)
        .with_tooltip(format!("{APP_NAME} {APP_VERSION}"));
    if let Some(icon) = icon {
        builder = builder.with_icon(icon);
    }
    match builder.build() {
        Ok(t) => Some(t),
        Err(e) => {
            eprintln!("tray icon creation failed: {e}");
            None
        }
    }
}

fn create_message_window() -> windows::core::Result<HWND> {
    unsafe {
        let hinstance = GetModuleHandleW(None)?;
        let wc = WNDCLASSW {
            lpfnWndProc: Some(wnd_proc),
            hInstance: hinstance.into(),
            lpszClassName: SERVICE_CLASS,
            ..Default::default()
        };
        RegisterClassW(&wc);
        CreateWindowExW(
            WINDOW_EX_STYLE(0),
            SERVICE_CLASS,
            w!("AutoText Service"),
            WINDOW_STYLE(0),
            0,
            0,
            0,
            0,
            Some(HWND_MESSAGE),
            None,
            Some(hinstance.into()),
            None,
        )
    }
}

unsafe extern "system" fn wnd_proc(hwnd: HWND, msg: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    match msg {
        WM_HOTKEY => {
            let id = wparam.0 as i32;
            let entry = STATE.with(|s| {
                s.borrow().as_ref().and_then(|st| st.hotkeys.get(&id).cloned())
            });
            if let Some((text, vk, mode)) = entry {
                thread::spawn(move || input::insert_text(&text, vk, mode));
            }
            LRESULT(0)
        }
        WM_APP_RELOAD => {
            reload_hotkeys();
            LRESULT(0)
        }
        WM_APP_SHOW => {
            open_settings();
            LRESULT(0)
        }
        _ => unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) },
    }
}

fn reload_hotkeys() {
    STATE.with(|s| {
        let mut guard = s.borrow_mut();
        let Some(st) = guard.as_mut() else { return };

        for id in st.hotkeys.keys() {
            unsafe {
                let _ = UnregisterHotKey(Some(st.hwnd), *id);
            }
        }
        st.hotkeys.clear();

        let config = st.shared.config.lock().unwrap().clone();
        let mut status = HashMap::new();
        let mut next_id = 1;
        for sn in config.snippets.iter().filter(|s| s.enabled) {
            if let Err(e) = sn.hotkey.validate() {
                status.insert(sn.id, e);
                continue;
            }
            let Some(vk) = key_to_vk(&sn.hotkey.key) else { continue };
            let mut mods = MOD_NOREPEAT;
            if sn.hotkey.win {
                mods |= MOD_WIN;
            }
            if sn.hotkey.ctrl {
                mods |= MOD_CONTROL;
            }
            if sn.hotkey.alt {
                mods |= MOD_ALT;
            }
            if sn.hotkey.shift {
                mods |= MOD_SHIFT;
            }
            match unsafe { RegisterHotKey(Some(st.hwnd), next_id, mods, vk) } {
                Ok(()) => {
                    st.hotkeys.insert(next_id, (sn.text.clone(), vk, sn.mode));
                }
                Err(_) => {
                    status.insert(
                        sn.id,
                        "다른 프로그램이나 Windows 가 이미 사용 중인 핫키입니다.".to_string(),
                    );
                }
            }
            next_id += 1;
        }

        *st.shared.status.lock().unwrap() = status;
        if let Some(ctx) = st.shared.egui_ctx.lock().unwrap().as_ref() {
            ctx.request_repaint();
        }
    });
}

fn open_settings() {
    STATE.with(|s| {
        let guard = s.borrow();
        let Some(st) = guard.as_ref() else { return };
        let ui = st.shared.ui_hwnd.load(Ordering::SeqCst);
        if ui != 0 {
            focus_window(HWND(ui as *mut _));
        } else {
            let _ = st.tx.send(MainCmd::OpenSettings);
        }
    });
}

pub fn focus_window(hwnd: HWND) {
    unsafe {
        if IsIconic(hwnd).as_bool() {
            let _ = ShowWindow(hwnd, SW_RESTORE);
        } else {
            let _ = ShowWindow(hwnd, SW_SHOW);
        }
        let _ = SetForegroundWindow(hwnd);
    }
}

fn quit(tray: Option<TrayIcon>) -> ! {
    STATE.with(|s| {
        if let Some(st) = s.borrow().as_ref() {
            for id in st.hotkeys.keys() {
                unsafe {
                    let _ = UnregisterHotKey(Some(st.hwnd), *id);
                }
            }
        }
    });
    // 트레이 아이콘을 먼저 제거하지 않으면 종료 후에도 아이콘이 남음
    drop(tray);
    std::process::exit(0);
}
