//! 상용구 설정 창 (egui)

use std::sync::Arc;
use std::sync::atomic::Ordering;

use eframe::egui;
use raw_window_handle::{HasWindowHandle, RawWindowHandle};

use crate::config::{APP_NAME, APP_VERSION, Config, InputMode, Snippet, available_keys};
use crate::{Shared, autostart, dialog, service, update};

const ICON_PNG: &[u8] = include_bytes!("../assets/autotext.png");

pub fn run_settings(shared: Arc<Shared>) {
    let mut viewport = egui::ViewportBuilder::default()
        .with_title(format!("{APP_NAME} {APP_VERSION}"))
        .with_inner_size([820.0, 520.0])
        .with_min_inner_size([640.0, 400.0]);
    if let Ok(icon) = eframe::icon_data::from_png_bytes(ICON_PNG) {
        viewport = viewport.with_icon(Arc::new(icon));
    }
    let options = eframe::NativeOptions { viewport, centered: true, ..Default::default() };

    let result = eframe::run_native(
        APP_NAME,
        options,
        Box::new(move |cc| {
            setup_fonts(&cc.egui_ctx);
            if let Ok(handle) = cc.window_handle()
                && let RawWindowHandle::Win32(h) = handle.as_raw()
            {
                shared.ui_hwnd.store(h.hwnd.get(), Ordering::SeqCst);
            }
            *shared.egui_ctx.lock().unwrap() = Some(cc.egui_ctx.clone());
            Ok(Box::new(SettingsApp::new(shared)))
        }),
    );
    if let Err(e) = result {
        eprintln!("settings window error: {e}");
    }
}

/// 기본 폰트에 한글이 없으므로 맑은 고딕을 우선 폰트로 추가
fn setup_fonts(ctx: &egui::Context) {
    let windir = std::env::var("WINDIR").unwrap_or_else(|_| "C:\\Windows".into());
    let path = std::path::Path::new(&windir).join("Fonts").join("malgun.ttf");
    let Ok(bytes) = std::fs::read(path) else { return };

    let mut fonts = egui::FontDefinitions::default();
    fonts
        .font_data
        .insert("malgun".into(), Arc::new(egui::FontData::from_owned(bytes)));
    for family in [egui::FontFamily::Proportional, egui::FontFamily::Monospace] {
        fonts.families.entry(family).or_default().insert(0, "malgun".into());
    }
    ctx.set_fonts(fonts);
}

enum Message {
    Info(String),
    Error(String),
}

/// 파일을 고른 뒤 추가/교체 선택을 기다리는 불러오기
struct PendingImport {
    file_name: String,
    snippets: Vec<Snippet>,
}

struct SettingsApp {
    shared: Arc<Shared>,
    keys: Vec<String>,
    /// 편집 중인 상용구 (저장 전 사본)
    draft: Option<Snippet>,
    /// 편집 시작 시점의 원본 (변경 여부 판단용)
    original: Option<Snippet>,
    message: Option<Message>,
    first_frame: bool,
    /// Windows 시작 시 자동 실행 여부 (레지스트리 기준)
    autostart: bool,
    /// 새 버전 자동 확인 여부 (설정 파일 기준)
    check_updates: bool,
    pending_import: Option<PendingImport>,
}

impl SettingsApp {
    fn new(shared: Arc<Shared>) -> Self {
        let (first, check_updates) = {
            let cfg = shared.config.lock().unwrap();
            (cfg.snippets.first().cloned(), cfg.check_updates)
        };
        Self {
            shared,
            keys: available_keys(),
            draft: first.clone(),
            original: first,
            message: None,
            first_frame: true,
            autostart: autostart::is_enabled(),
            check_updates,
            pending_import: None,
        }
    }

    /// 상용구 외 설정 변경을 설정 파일에 반영
    fn save_config(&mut self, edit: impl FnOnce(&mut Config)) -> Result<(), String> {
        let mut cfg = self.shared.config.lock().unwrap();
        let mut next = cfg.clone();
        edit(&mut next);
        next.save().map_err(|e| format!("설정 파일 저장 실패: {e}"))?;
        *cfg = next;
        Ok(())
    }

    fn toggle_check_updates(&mut self) {
        let enabled = self.check_updates;
        match self.save_config(|c| c.check_updates = enabled) {
            Ok(()) => {
                if enabled {
                    self.shared.post_service(service::WM_APP_CHECK_UPDATE, 0);
                }
            }
            Err(e) => {
                self.check_updates = !enabled;
                self.message = Some(Message::Error(e));
            }
        }
    }

    fn skip_update(&mut self, info: &update::UpdateInfo) {
        let tag = info.tag.clone();
        if let Err(e) = self.save_config(|c| c.skipped_version = Some(tag)) {
            self.message = Some(Message::Error(e));
            return;
        }
        self.shared.update.lock().unwrap().available = None;
        self.shared.post_service(service::WM_APP_UPDATE_CHANGED, 0);
    }

    /// 설치 파일 다운로드 후 설치기를 실행하고 프로그램 종료 (설치기가 교체 후 다시 실행함)
    fn install_update(&mut self, info: update::UpdateInfo) {
        {
            let mut st = self.shared.update.lock().unwrap();
            st.progress = Some(0.0);
            st.error = None;
        }
        let shared = self.shared.clone();
        std::thread::spawn(move || {
            let result = update::download(&info, |p| {
                shared.update.lock().unwrap().progress = Some(p);
                shared.repaint();
            })
            .and_then(|path| update::run_installer(&path));
            match result {
                Ok(()) => shared.post_service(service::WM_APP_QUIT, 0),
                Err(e) => {
                    let mut st = shared.update.lock().unwrap();
                    st.progress = None;
                    st.error = Some(e);
                    drop(st);
                    shared.repaint();
                }
            }
        });
    }

    fn update_banner(&mut self, ui: &mut egui::Ui) {
        let (info, progress, error) = {
            let st = self.shared.update.lock().unwrap();
            (st.available.clone(), st.progress, st.error.clone())
        };
        let Some(info) = info else { return };

        let mut install = false;
        let mut skip = false;
        egui::Panel::top("update").show(ui, |ui| {
            ui.add_space(6.0);
            ui.horizontal(|ui| {
                ui.strong(format!("새 버전 v{} 이 있습니다.", info.version_text()));
                ui.weak(format!("(현재 {APP_VERSION})"));
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let busy = progress.is_some();
                    skip = ui.add_enabled(!busy, egui::Button::new("이 버전 건너뛰기")).clicked();
                    install = ui.add_enabled(!busy, egui::Button::new("지금 설치")).clicked();
                });
            });
            if let Some(p) = progress {
                ui.add(egui::ProgressBar::new(p).show_percentage().text(format!(
                    "다운로드 중... {:.0}%  (완료되면 잠시 종료된 뒤 자동으로 다시 실행됩니다)",
                    p * 100.0
                )));
            }
            if let Some(e) = &error {
                ui.colored_label(ui.visuals().error_fg_color, format!("업데이트 실패: {e}"));
            }
            let notes = info.notes.trim();
            if !notes.is_empty() {
                egui::CollapsingHeader::new("변경 내용").show(ui, |ui| {
                    egui::ScrollArea::vertical().max_height(120.0).show(ui, |ui| {
                        ui.label(notes);
                    });
                });
            }
            ui.add_space(4.0);
        });

        if install {
            self.install_update(info);
        } else if skip {
            self.skip_update(&info);
        }
    }

    fn toggle_autostart(&mut self) {
        match autostart::set_enabled(self.autostart) {
            Ok(()) => {
                let m = if self.autostart {
                    "Windows 시작 시 자동으로 실행됩니다."
                } else {
                    "자동 실행을 해제했습니다."
                };
                self.message = Some(Message::Info(m.into()));
            }
            Err(e) => {
                self.autostart = !self.autostart;
                self.message = Some(Message::Error(format!("자동 실행 설정 실패: {e}")));
            }
        }
    }

    fn is_dirty(&self) -> bool {
        self.draft != self.original
    }

    fn select(&mut self, snippet: Snippet) {
        self.draft = Some(snippet.clone());
        self.original = Some(snippet);
        self.message = None;
    }

    fn new_snippet(&mut self) {
        let id = {
            let cfg = self.shared.config.lock().unwrap();
            // 저장되지 않은 새 항목이 있으면 id 중복을 피함
            let draft_id = self.draft.as_ref().map(|d| d.id).unwrap_or(0);
            cfg.next_id().max(draft_id + 1)
        };
        let mut s = Snippet { id, ..Default::default() };
        s.hotkey.ctrl = true;
        s.hotkey.alt = true;
        self.draft = Some(s);
        self.original = None;
        self.message = Some(Message::Info("새 상용구를 입력한 뒤 저장하세요.".into()));
    }

    fn save_draft(&mut self) {
        let Some(draft) = self.draft.clone() else { return };
        let result = {
            let mut cfg = self.shared.config.lock().unwrap();
            match cfg.validate_snippet(&draft) {
                Err(e) => Err(e),
                Ok(()) => {
                    let mut next = cfg.clone();
                    next.upsert(draft.clone());
                    match next.save() {
                        Ok(()) => {
                            *cfg = next;
                            Ok(())
                        }
                        Err(e) => Err(format!("설정 파일 저장 실패: {e}")),
                    }
                }
            }
        };
        match result {
            Ok(()) => {
                self.original = Some(draft);
                self.message = Some(Message::Info("저장했습니다.".into()));
                self.shared.request_reload();
            }
            Err(e) => self.message = Some(Message::Error(e)),
        }
    }

    fn delete_draft(&mut self) {
        let Some(draft) = self.draft.clone() else { return };
        let result = {
            let mut cfg = self.shared.config.lock().unwrap();
            let mut next = cfg.clone();
            next.remove(draft.id);
            match next.save() {
                Ok(()) => {
                    *cfg = next;
                    Ok(cfg.snippets.first().cloned())
                }
                Err(e) => Err(format!("설정 파일 저장 실패: {e}")),
            }
        };
        match result {
            Ok(first) => {
                self.draft = first.clone();
                self.original = first;
                self.message = Some(Message::Info("삭제했습니다.".into()));
                self.shared.request_reload();
            }
            Err(e) => self.message = Some(Message::Error(e)),
        }
    }

    /// 저장된 상용구 목록을 JSON 파일로 내보내기
    fn export_snippets(&mut self) {
        let owner = self.shared.ui_hwnd.load(Ordering::SeqCst);
        let Some(path) = dialog::save_json(owner, "상용구 내보내기", "AutoText-snippets.json") else { return };
        let (json, count) = {
            let cfg = self.shared.config.lock().unwrap();
            (cfg.export_json(), cfg.snippets.len())
        };
        self.message = Some(match std::fs::write(&path, json) {
            Ok(()) => Message::Info(format!("{count}개를 내보냈습니다.")),
            Err(e) => Message::Error(format!("내보내기 실패: {e}")),
        });
    }

    /// 파일을 읽어 추가/교체 선택 대화상자를 띄움
    fn begin_import(&mut self) {
        let owner = self.shared.ui_hwnd.load(Ordering::SeqCst);
        let Some(path) = dialog::open_json(owner, "상용구 불러오기") else { return };
        let parsed = std::fs::read_to_string(&path)
            .map_err(|e| format!("파일을 읽을 수 없습니다: {e}"))
            .and_then(|s| Config::parse_import(&s));
        match parsed {
            Ok(snippets) if snippets.is_empty() => {
                self.message = Some(Message::Error("파일에 상용구가 없습니다.".into()));
            }
            Ok(snippets) => {
                let file_name = path.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default();
                self.pending_import = Some(PendingImport { file_name, snippets });
            }
            Err(e) => self.message = Some(Message::Error(e)),
        }
    }

    fn apply_import(&mut self, pending: PendingImport, replace: bool) {
        let result = {
            let mut cfg = self.shared.config.lock().unwrap();
            let mut next = cfg.clone();
            let summary = next.import_snippets(pending.snippets, replace);
            match next.save() {
                Ok(()) => {
                    *cfg = next;
                    Ok((summary, cfg.clone()))
                }
                Err(e) => Err(format!("설정 파일 저장 실패: {e}")),
            }
        };
        match result {
            Ok((summary, cfg)) => {
                // 추가 모드에서는 기존 항목 편집을 유지하고, 그 외에는 첫 항목을 선택
                let keep = !replace
                    && self.original.as_ref().is_some_and(|o| cfg.snippets.iter().any(|s| s.id == o.id));
                if !keep {
                    let first = cfg.snippets.first().cloned();
                    self.draft = first.clone();
                    self.original = first;
                }
                self.message = Some(Message::Info(summary.describe()));
                self.shared.request_reload();
            }
            Err(e) => self.message = Some(Message::Error(e)),
        }
    }

    fn import_modal(&mut self, ctx: &egui::Context) {
        let Some(pending) = &self.pending_import else { return };
        let dirty = self.is_dirty();
        let mut choice: Option<bool> = None;
        let mut cancel = false;
        let resp = egui::Modal::new(egui::Id::unique("import_modal")).show(ctx, |ui| {
            ui.set_width(380.0);
            ui.heading("상용구 불러오기");
            ui.add_space(6.0);
            ui.label(format!("'{}' 에서 상용구 {}개를 찾았습니다.", pending.file_name, pending.snippets.len()));
            ui.add_space(4.0);
            ui.weak("추가: 기존 목록 뒤에 붙입니다. 완전히 같은 항목은 건너뛰고, 핫키가 겹치면 사용 안 함으로 추가합니다.");
            ui.weak("기존 목록 바꾸기: 현재 상용구를 모두 지우고 파일의 상용구로 바꿉니다.");
            if dirty {
                ui.add_space(4.0);
                ui.colored_label(ui.visuals().warn_fg_color, "편집 중인 저장하지 않은 내용은 사라질 수 있습니다.");
            }
            ui.add_space(10.0);
            ui.horizontal(|ui| {
                if ui.button("추가").clicked() {
                    choice = Some(false);
                }
                if ui.button("기존 목록 바꾸기").clicked() {
                    choice = Some(true);
                }
                if ui.button("취소").clicked() {
                    cancel = true;
                }
            });
        });
        if let Some(replace) = choice {
            if let Some(pending) = self.pending_import.take() {
                self.apply_import(pending, replace);
            }
        } else if cancel || resp.should_close() {
            self.pending_import = None;
        }
    }

    fn message_ui(&self, ui: &mut egui::Ui) {
        match &self.message {
            Some(Message::Info(m)) => {
                ui.label(m);
            }
            Some(Message::Error(m)) => {
                ui.colored_label(ui.visuals().error_fg_color, m);
            }
            None => {}
        }
    }

    fn list_ui(&mut self, ui: &mut egui::Ui, config: &Config) {
        ui.horizontal_wrapped(|ui| {
            if ui.button("+ 새 상용구").clicked() {
                self.new_snippet();
            }
            if ui.button("불러오기").on_hover_text("JSON 파일에서 상용구를 불러옵니다.").clicked() {
                self.begin_import();
            }
            if ui
                .add_enabled(!config.snippets.is_empty(), egui::Button::new("내보내기"))
                .on_hover_text("저장된 상용구를 JSON 파일로 내보냅니다.")
                .clicked()
            {
                self.export_snippets();
            }
        });
        ui.separator();

        let status = self.shared.status.lock().unwrap().clone();
        let selected_id = self.draft.as_ref().map(|d| d.id);
        let mut clicked: Option<Snippet> = None;

        egui::ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| {
            if config.snippets.is_empty() {
                ui.weak("등록된 상용구가 없습니다.");
            }
            for sn in &config.snippets {
                let mut label = format!("{}\n{}", sn.label(), sn.hotkey.display());
                if !sn.enabled {
                    label.push_str("  (사용 안 함)");
                }
                let mut text = egui::RichText::new(label);
                if status.contains_key(&sn.id) {
                    text = text.color(ui.visuals().error_fg_color);
                } else if !sn.enabled {
                    text = text.weak();
                }
                let resp = ui.add_sized(
                    [ui.available_width(), 0.0],
                    egui::Button::selectable(selected_id == Some(sn.id), text),
                );
                let resp = match status.get(&sn.id) {
                    Some(err) => resp.on_hover_text(err),
                    None => resp,
                };
                if resp.clicked() {
                    clicked = Some(sn.clone());
                }
            }
        });

        if let Some(sn) = clicked {
            self.select(sn);
        }
    }

    fn editor_ui(&mut self, ui: &mut egui::Ui, config: &Config) {
        let keys = &self.keys;
        let Some(draft) = self.draft.as_mut() else {
            ui.add_space(40.0);
            ui.vertical_centered(|ui| {
                ui.label("왼쪽 목록에서 상용구를 선택하거나 '+ 새 상용구' 버튼을 누르세요.");
                ui.add_space(8.0);
                self.message_ui(ui);
            });
            return;
        };
        let is_new = !config.snippets.iter().any(|s| s.id == draft.id);

        ui.heading(if is_new { "새 상용구" } else { "상용구 편집" });
        ui.add_space(6.0);

        egui::Grid::new("editor_grid").num_columns(2).spacing([12.0, 10.0]).show(ui, |ui| {
            ui.label("이름");
            ui.add(
                egui::TextEdit::singleline(&mut draft.name)
                    .hint_text("목록에 표시될 이름 (선택)")
                    .desired_width(f32::INFINITY),
            );
            ui.end_row();

            ui.label("핫키");
            ui.horizontal(|ui| {
                ui.checkbox(&mut draft.hotkey.ctrl, "Ctrl");
                ui.checkbox(&mut draft.hotkey.alt, "Alt");
                ui.checkbox(&mut draft.hotkey.shift, "Shift");
                ui.checkbox(&mut draft.hotkey.win, "Win");
                ui.label("+");
                let selected = if draft.hotkey.key.is_empty() {
                    "키 선택".to_string()
                } else {
                    draft.hotkey.key.clone()
                };
                egui::ComboBox::from_id_salt("trigger_key")
                    .selected_text(selected)
                    .height(320.0)
                    .show_ui(ui, |ui| {
                        for k in keys {
                            ui.selectable_value(&mut draft.hotkey.key, k.clone(), k);
                        }
                    });
                ui.add_space(8.0);
                ui.strong(draft.hotkey.display());
            });
            ui.end_row();

            ui.label("입력 방식");
            ui.horizontal(|ui| {
                ui.radio_value(&mut draft.mode, InputMode::Type, "직접 입력")
                    .on_hover_text("한 글자씩 키 입력으로 전송합니다. 클립보드를 건드리지 않습니다.");
                ui.radio_value(&mut draft.mode, InputMode::Paste, "클립보드 붙여넣기")
                    .on_hover_text(
                        "클립보드에 넣고 Ctrl+V 로 붙여넣은 뒤 원래 클립보드 내용을 복원합니다.\n긴 글이나 직접 입력이 잘 안 되는 프로그램에 적합합니다.",
                    );
            });
            ui.end_row();

            ui.label("");
            ui.checkbox(&mut draft.enabled, "이 상용구 사용");
            ui.end_row();
        });

        if let Some(err) = self.shared.status.lock().unwrap().get(&draft.id)
            && !is_new
        {
            ui.colored_label(ui.visuals().error_fg_color, format!("핫키 등록 실패: {err}"));
        }

        ui.add_space(6.0);
        ui.label("상용구 내용");
        let bottom_reserved = 48.0;
        let height = (ui.available_height() - bottom_reserved).max(80.0);
        egui::ScrollArea::vertical().max_height(height).show(ui, |ui| {
            ui.add_sized(
                [ui.available_width(), height],
                egui::TextEdit::multiline(&mut draft.text)
                    .hint_text("핫키를 누르면 입력될 내용")
                    .desired_rows(8),
            );
        });

        ui.add_space(8.0);
        let dirty = self.draft != self.original;
        let mut do_save = false;
        let mut do_revert = false;
        let mut do_delete = false;
        ui.horizontal(|ui| {
            let save_label = if dirty { "저장 *" } else { "저장" };
            if ui.add_enabled(dirty || is_new, egui::Button::new(save_label)).clicked() {
                do_save = true;
            }
            if ui.add_enabled(dirty && !is_new, egui::Button::new("되돌리기")).clicked() {
                do_revert = true;
            }
            if !is_new && ui.button("삭제").clicked() {
                do_delete = true;
            }
            self.message_ui(ui);
        });

        if do_save {
            self.save_draft();
        } else if do_revert {
            self.draft = self.original.clone();
            self.message = None;
        } else if do_delete {
            self.delete_draft();
        }
    }
}

impl eframe::App for SettingsApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        if self.first_frame {
            self.first_frame = false;
            ui.ctx().send_viewport_cmd(egui::ViewportCommand::Focus);
        }

        // Ctrl+S 저장
        if ui.input_mut(|i| i.consume_key(egui::Modifiers::COMMAND, egui::Key::S)) && self.is_dirty() {
            self.save_draft();
        }

        let config = self.shared.config.lock().unwrap().clone();

        self.update_banner(ui);

        let mut autostart_changed = false;
        let mut check_updates_changed = false;
        egui::Panel::bottom("footer").show(ui, |ui| {
            ui.horizontal(|ui| {
                autostart_changed = ui
                    .checkbox(&mut self.autostart, "Windows 시작 시 자동 실행")
                    .changed();
                check_updates_changed = ui
                    .checkbox(&mut self.check_updates, "새 버전 자동 확인")
                    .on_hover_text(
                        "시작 직후와 하루 한 번 GitHub 릴리스를 확인합니다.\n트레이 메뉴의 '업데이트 확인'으로 언제든 직접 확인할 수 있습니다.",
                    )
                    .changed();
                ui.separator();
                ui.weak(format!("{APP_NAME} {APP_VERSION}"));
                ui.weak(" - 창을 닫아도 트레이에서 계속 동작합니다.");
            });
        });
        if autostart_changed {
            self.toggle_autostart();
        }
        if check_updates_changed {
            self.toggle_check_updates();
        }

        egui::Panel::left("list")
            .resizable(true)
            .default_size(250.0)
            .min_size(180.0)
            .show(ui, |ui| {
                ui.add_space(6.0);
                self.list_ui(ui, &config);
            });

        egui::CentralPanel::default().show(ui, |ui| {
            ui.add_space(6.0);
            self.editor_ui(ui, &config);
        });

        let ctx = ui.ctx().clone();
        self.import_modal(&ctx);
    }
}

impl Drop for SettingsApp {
    fn drop(&mut self) {
        self.shared.ui_hwnd.store(0, Ordering::SeqCst);
        *self.shared.egui_ctx.lock().unwrap() = None;
    }
}
