use std::fs;
use std::io;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

pub const APP_NAME: &str = "AutoText";
pub const APP_VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct Hotkey {
    pub win: bool,
    pub ctrl: bool,
    pub alt: bool,
    pub shift: bool,
    /// 트리거 키 이름 (A-Z, 0-9, F1-F12)
    pub key: String,
}

impl Hotkey {
    pub fn display(&self) -> String {
        let mut parts: Vec<&str> = Vec::new();
        if self.ctrl {
            parts.push("Ctrl");
        }
        if self.alt {
            parts.push("Alt");
        }
        if self.shift {
            parts.push("Shift");
        }
        if self.win {
            parts.push("Win");
        }
        if !self.key.is_empty() {
            parts.push(&self.key);
        }
        if parts.is_empty() {
            "(없음)".to_string()
        } else {
            parts.join("+")
        }
    }

    pub fn validate(&self) -> Result<(), String> {
        if key_to_vk(&self.key).is_none() {
            return Err("트리거 키를 선택하세요.".to_string());
        }
        if !is_function_key(&self.key) && !(self.win || self.ctrl || self.alt) {
            return Err("문자/숫자 키에는 Win, Ctrl, Alt 중 하나 이상을 조합해야 합니다.".to_string());
        }
        Ok(())
    }

    /// 같은 키 조합인지 비교 (대소문자 무시)
    pub fn same_combo(&self, other: &Hotkey) -> bool {
        self.win == other.win
            && self.ctrl == other.ctrl
            && self.alt == other.alt
            && self.shift == other.shift
            && self.key.eq_ignore_ascii_case(&other.key)
    }
}

/// 상용구 입력 방식
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum InputMode {
    /// 한 글자씩 키 입력으로 전송
    #[default]
    Type,
    /// 클립보드에 넣고 Ctrl+V 로 붙여넣은 뒤 기존 클립보드 복원
    Paste,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct Snippet {
    pub id: u64,
    pub name: String,
    pub text: String,
    pub hotkey: Hotkey,
    pub enabled: bool,
    pub mode: InputMode,
}

impl Default for Snippet {
    fn default() -> Self {
        Self {
            id: 0,
            name: String::new(),
            text: String::new(),
            hotkey: Hotkey::default(),
            enabled: true,
            mode: InputMode::Type,
        }
    }
}

impl Snippet {
    /// 목록 표시용 이름 (이름이 없으면 내용 첫 줄 사용)
    pub fn label(&self) -> String {
        let name = self.name.trim();
        if !name.is_empty() {
            return name.to_string();
        }
        let first = self.text.lines().next().unwrap_or("").trim();
        if first.is_empty() {
            return "(이름 없음)".to_string();
        }
        let mut s: String = first.chars().take(20).collect();
        if first.chars().count() > 20 {
            s.push_str("...");
        }
        s
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct Config {
    pub snippets: Vec<Snippet>,
    /// 시작 직후와 하루 한 번 새 버전 자동 확인
    pub check_updates: bool,
    /// 다시 알리지 않을 버전 태그 (예: v1.0.1)
    pub skipped_version: Option<String>,
}

impl Default for Config {
    fn default() -> Self {
        Self { snippets: Vec::new(), check_updates: true, skipped_version: None }
    }
}

impl Config {
    pub fn next_id(&self) -> u64 {
        self.snippets.iter().map(|s| s.id).max().unwrap_or(0) + 1
    }

    /// 저장 전 검증 (자기 자신은 id 로 제외)
    pub fn validate_snippet(&self, snippet: &Snippet) -> Result<(), String> {
        if snippet.text.is_empty() {
            return Err("상용구 내용을 입력하세요.".to_string());
        }
        snippet.hotkey.validate()?;
        if snippet.enabled {
            if let Some(other) = self.snippets.iter().find(|s| {
                s.id != snippet.id && s.enabled && s.hotkey.same_combo(&snippet.hotkey)
            }) {
                return Err(format!(
                    "'{}' 상용구가 이미 {} 핫키를 사용 중입니다.",
                    other.label(),
                    snippet.hotkey.display()
                ));
            }
        }
        Ok(())
    }

    /// id 가 같으면 교체, 없으면 추가
    pub fn upsert(&mut self, snippet: Snippet) {
        match self.snippets.iter_mut().find(|s| s.id == snippet.id) {
            Some(slot) => *slot = snippet,
            None => self.snippets.push(snippet),
        }
    }

    pub fn remove(&mut self, id: u64) {
        self.snippets.retain(|s| s.id != id);
    }

    pub fn path() -> PathBuf {
        let base = std::env::var_os("APPDATA")
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from("."));
        base.join(APP_NAME).join("config.json")
    }

    pub fn load() -> Config {
        fs::read_to_string(Self::path())
            .ok()
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_default()
    }

    pub fn save(&self) -> io::Result<()> {
        let path = Self::path();
        if let Some(dir) = path.parent() {
            fs::create_dir_all(dir)?;
        }
        let json = serde_json::to_string_pretty(self).map_err(io::Error::other)?;
        // 임시 파일에 쓴 뒤 교체하여 저장 중 손상 방지
        let tmp = path.with_extension("json.tmp");
        fs::write(&tmp, json)?;
        fs::rename(&tmp, &path)
    }
}

/// 선택 가능한 트리거 키 목록
pub fn available_keys() -> Vec<String> {
    let mut keys: Vec<String> = ('A'..='Z').map(|c| c.to_string()).collect();
    keys.extend(('0'..='9').map(|c| c.to_string()));
    keys.extend((1..=12).map(|n| format!("F{n}")));
    keys
}

fn is_function_key(key: &str) -> bool {
    key.len() >= 2
        && key.starts_with(['F', 'f'])
        && key[1..].parse::<u32>().is_ok_and(|n| (1..=12).contains(&n))
}

/// 키 이름을 Windows 가상 키 코드로 변환
pub fn key_to_vk(key: &str) -> Option<u32> {
    if is_function_key(key) {
        let n: u32 = key[1..].parse().ok()?;
        return Some(0x70 + n - 1); // VK_F1 = 0x70
    }
    let mut chars = key.chars();
    let c = chars.next()?.to_ascii_uppercase();
    if chars.next().is_some() {
        return None;
    }
    match c {
        'A'..='Z' | '0'..='9' => Some(c as u32),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn hk(win: bool, ctrl: bool, alt: bool, shift: bool, key: &str) -> Hotkey {
        Hotkey { win, ctrl, alt, shift, key: key.to_string() }
    }

    fn snip(id: u64, hotkey: Hotkey, enabled: bool) -> Snippet {
        Snippet { id, name: format!("s{id}"), text: "hello".into(), hotkey, enabled, ..Default::default() }
    }

    #[test]
    fn vk_mapping() {
        assert_eq!(key_to_vk("A"), Some(0x41));
        assert_eq!(key_to_vk("z"), Some(0x5A));
        assert_eq!(key_to_vk("0"), Some(0x30));
        assert_eq!(key_to_vk("9"), Some(0x39));
        assert_eq!(key_to_vk("F1"), Some(0x70));
        assert_eq!(key_to_vk("F12"), Some(0x7B));
        assert_eq!(key_to_vk("F13"), None);
        assert_eq!(key_to_vk("F"), Some(0x46));
        assert_eq!(key_to_vk(""), None);
        assert_eq!(key_to_vk("AB"), None);
        assert_eq!(key_to_vk("-"), None);
    }

    #[test]
    fn all_available_keys_map_to_vk() {
        let keys = available_keys();
        assert_eq!(keys.len(), 26 + 10 + 12);
        for k in keys {
            assert!(key_to_vk(&k).is_some(), "{k}");
        }
    }

    #[test]
    fn display_order() {
        assert_eq!(hk(true, true, true, true, "A").display(), "Ctrl+Alt+Shift+Win+A");
        assert_eq!(hk(false, true, false, false, "F5").display(), "Ctrl+F5");
        assert_eq!(Hotkey::default().display(), "(없음)");
    }

    #[test]
    fn hotkey_validation() {
        assert!(hk(false, false, false, false, "").validate().is_err());
        assert!(hk(false, false, false, false, "A").validate().is_err());
        assert!(hk(false, false, false, true, "A").validate().is_err());
        assert!(hk(false, true, false, false, "A").validate().is_ok());
        assert!(hk(true, false, false, false, "1").validate().is_ok());
        assert!(hk(false, false, true, true, "Q").validate().is_ok());
        assert!(hk(false, false, false, false, "F8").validate().is_ok());
    }

    #[test]
    fn duplicate_detection() {
        let mut cfg = Config::default();
        cfg.upsert(snip(1, hk(false, true, true, false, "A"), true));

        let dup = snip(2, hk(false, true, true, false, "a"), true);
        assert!(cfg.validate_snippet(&dup).is_err());

        // 비활성 상용구끼리는 충돌로 보지 않음
        let disabled = snip(2, hk(false, true, true, false, "A"), false);
        assert!(cfg.validate_snippet(&disabled).is_ok());

        // 자기 자신 수정은 허용
        let same = snip(1, hk(false, true, true, false, "A"), true);
        assert!(cfg.validate_snippet(&same).is_ok());

        let other = snip(2, hk(false, true, true, true, "A"), true);
        assert!(cfg.validate_snippet(&other).is_ok());
    }

    #[test]
    fn empty_text_rejected() {
        let cfg = Config::default();
        let mut s = snip(1, hk(false, true, false, false, "A"), true);
        s.text.clear();
        assert!(cfg.validate_snippet(&s).is_err());
    }

    #[test]
    fn upsert_remove_next_id() {
        let mut cfg = Config::default();
        assert_eq!(cfg.next_id(), 1);
        cfg.upsert(snip(1, hk(false, true, false, false, "A"), true));
        cfg.upsert(snip(5, hk(false, true, false, false, "B"), true));
        assert_eq!(cfg.next_id(), 6);
        let mut edited = snip(1, hk(false, true, false, false, "C"), true);
        edited.text = "changed".into();
        cfg.upsert(edited);
        assert_eq!(cfg.snippets.len(), 2);
        assert_eq!(cfg.snippets[0].text, "changed");
        cfg.remove(1);
        assert_eq!(cfg.snippets.len(), 1);
        assert_eq!(cfg.snippets[0].id, 5);
    }

    #[test]
    fn label_fallback() {
        let mut s = Snippet { text: "첫 줄\n둘째 줄".into(), ..Default::default() };
        assert_eq!(s.label(), "첫 줄");
        s.name = "인사".into();
        assert_eq!(s.label(), "인사");
        let long = Snippet { text: "가".repeat(30), ..Default::default() };
        assert_eq!(long.label(), format!("{}...", "가".repeat(20)));
        assert_eq!(Snippet::default().label(), "(이름 없음)");
    }

    #[test]
    fn json_roundtrip_and_missing_fields() {
        let mut cfg = Config::default();
        cfg.upsert(snip(1, hk(true, false, false, false, "F1"), true));
        let json = serde_json::to_string(&cfg).unwrap();
        let back: Config = serde_json::from_str(&json).unwrap();
        assert_eq!(cfg, back);

        let partial: Config = serde_json::from_str(r#"{"snippets":[{"id":3,"text":"x"}]}"#).unwrap();
        assert_eq!(partial.snippets[0].id, 3);
        assert!(partial.snippets[0].enabled);
        assert_eq!(partial.snippets[0].hotkey, Hotkey::default());
        // mode 필드가 없는 기존 설정 호환: 직접 입력으로 처리
        assert_eq!(partial.snippets[0].mode, InputMode::Type);
        // 업데이트 필드가 없는 기존 설정 호환: 자동 확인 사용
        assert!(partial.check_updates);
        assert_eq!(partial.skipped_version, None);
    }

    #[test]
    fn input_mode_serialization() {
        let s = Snippet { mode: InputMode::Paste, ..Default::default() };
        let json = serde_json::to_string(&s).unwrap();
        assert!(json.contains(r#""mode":"paste""#), "{json}");
        let back: Snippet = serde_json::from_str(&json).unwrap();
        assert_eq!(back.mode, InputMode::Paste);
    }
}
