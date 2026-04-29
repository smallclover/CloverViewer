use global_hotkey::hotkey::Code;

/// 解析后的热键中间表示。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedHotkey {
    pub ctrl: bool,
    pub alt: bool,
    pub shift: bool,
    pub cmd: bool,
    pub key_name: String,
}

/// 将配置热键字符串（如 "Ctrl+Alt+C", "Cmd+Shift+F12"）解析为 ParsedHotkey。
pub fn parse_hotkey_str(hotkey_str: &str) -> Option<ParsedHotkey> {
    if hotkey_str.is_empty() {
        return None;
    }
    let parts: Vec<&str> = hotkey_str.split('+').collect();

    let mut ctrl = false;
    let mut alt = false;
    let mut shift = false;
    let mut cmd = false;
    let mut key_name = None;

    for part in parts {
        match part {
            "Ctrl" => ctrl = true,
            "Alt" => alt = true,
            "Shift" => shift = true,
            "Cmd" | "Super" => cmd = true,
            name => {
                if key_name.is_some() {
                    return None;
                }
                key_name = Some(name.to_string());
            }
        }
    }

    key_name.map(|k| ParsedHotkey {
        ctrl,
        alt,
        shift,
        cmd,
        key_name: k,
    })
}

/// key_name → `global_hotkey::hotkey::Code`
pub fn parsed_key_to_code(key_name: &str) -> Option<Code> {
    match key_name {
        "A" => Some(Code::KeyA),
        "B" => Some(Code::KeyB),
        "C" => Some(Code::KeyC),
        "D" => Some(Code::KeyD),
        "E" => Some(Code::KeyE),
        "F" => Some(Code::KeyF),
        "G" => Some(Code::KeyG),
        "H" => Some(Code::KeyH),
        "I" => Some(Code::KeyI),
        "J" => Some(Code::KeyJ),
        "K" => Some(Code::KeyK),
        "L" => Some(Code::KeyL),
        "M" => Some(Code::KeyM),
        "N" => Some(Code::KeyN),
        "O" => Some(Code::KeyO),
        "P" => Some(Code::KeyP),
        "Q" => Some(Code::KeyQ),
        "R" => Some(Code::KeyR),
        "S" => Some(Code::KeyS),
        "T" => Some(Code::KeyT),
        "U" => Some(Code::KeyU),
        "V" => Some(Code::KeyV),
        "W" => Some(Code::KeyW),
        "X" => Some(Code::KeyX),
        "Y" => Some(Code::KeyY),
        "Z" => Some(Code::KeyZ),
        "Num0" => Some(Code::Digit0),
        "Num1" => Some(Code::Digit1),
        "Num2" => Some(Code::Digit2),
        "Num3" => Some(Code::Digit3),
        "Num4" => Some(Code::Digit4),
        "Num5" => Some(Code::Digit5),
        "Num6" => Some(Code::Digit6),
        "Num7" => Some(Code::Digit7),
        "Num8" => Some(Code::Digit8),
        "Num9" => Some(Code::Digit9),
        "F1" => Some(Code::F1),
        "F2" => Some(Code::F2),
        "F3" => Some(Code::F3),
        "F4" => Some(Code::F4),
        "F5" => Some(Code::F5),
        "F6" => Some(Code::F6),
        "F7" => Some(Code::F7),
        "F8" => Some(Code::F8),
        "F9" => Some(Code::F9),
        "F10" => Some(Code::F10),
        "F11" => Some(Code::F11),
        "F12" => Some(Code::F12),
        "Space" => Some(Code::Space),
        "Enter" => Some(Code::Enter),
        "Escape" => Some(Code::Escape),
        "Tab" => Some(Code::Tab),
        "Backspace" => Some(Code::Backspace),
        _ => {
            tracing::debug!("Unknown key name for Code: {}", key_name);
            None
        }
    }
}

/// key_name → `egui::Key`
pub fn parsed_key_to_egui_key(key_name: &str) -> Option<egui::Key> {
    use egui::Key;
    match key_name {
        "A" => Some(Key::A),
        "B" => Some(Key::B),
        "C" => Some(Key::C),
        "D" => Some(Key::D),
        "E" => Some(Key::E),
        "F" => Some(Key::F),
        "G" => Some(Key::G),
        "H" => Some(Key::H),
        "I" => Some(Key::I),
        "J" => Some(Key::J),
        "K" => Some(Key::K),
        "L" => Some(Key::L),
        "M" => Some(Key::M),
        "N" => Some(Key::N),
        "O" => Some(Key::O),
        "P" => Some(Key::P),
        "Q" => Some(Key::Q),
        "R" => Some(Key::R),
        "S" => Some(Key::S),
        "T" => Some(Key::T),
        "U" => Some(Key::U),
        "V" => Some(Key::V),
        "W" => Some(Key::W),
        "X" => Some(Key::X),
        "Y" => Some(Key::Y),
        "Z" => Some(Key::Z),
        "Num0" => Some(Key::Num0),
        "Num1" => Some(Key::Num1),
        "Num2" => Some(Key::Num2),
        "Num3" => Some(Key::Num3),
        "Num4" => Some(Key::Num4),
        "Num5" => Some(Key::Num5),
        "Num6" => Some(Key::Num6),
        "Num7" => Some(Key::Num7),
        "Num8" => Some(Key::Num8),
        "Num9" => Some(Key::Num9),
        "F1" => Some(Key::F1),
        "F2" => Some(Key::F2),
        "F3" => Some(Key::F3),
        "F4" => Some(Key::F4),
        "F5" => Some(Key::F5),
        "F6" => Some(Key::F6),
        "F7" => Some(Key::F7),
        "F8" => Some(Key::F8),
        "F9" => Some(Key::F9),
        "F10" => Some(Key::F10),
        "F11" => Some(Key::F11),
        "F12" => Some(Key::F12),
        "Space" => Some(Key::Space),
        "Enter" => Some(Key::Enter),
        "Escape" => Some(Key::Escape),
        "Tab" => Some(Key::Tab),
        "Backspace" => Some(Key::Backspace),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_basic_combinations() {
        let h = parse_hotkey_str("Ctrl+Alt+S").unwrap();
        assert!(h.ctrl);
        assert!(h.alt);
        assert!(!h.shift);
        assert!(!h.cmd);
        assert_eq!(h.key_name, "S");

        let h = parse_hotkey_str("Cmd+Shift+F12").unwrap();
        assert!(!h.ctrl);
        assert!(!h.alt);
        assert!(h.shift);
        assert!(h.cmd);
        assert_eq!(h.key_name, "F12");
    }

    #[test]
    fn parse_super_as_cmd() {
        let h = parse_hotkey_str("Super+X").unwrap();
        assert!(h.cmd);
        assert_eq!(h.key_name, "X");
    }

    #[test]
    fn parse_single_key() {
        let h = parse_hotkey_str("F5").unwrap();
        assert!(!h.ctrl && !h.alt && !h.shift && !h.cmd);
        assert_eq!(h.key_name, "F5");
    }

    #[test]
    fn reject_empty() {
        assert!(parse_hotkey_str("").is_none());
    }

    #[test]
    fn key_to_code_maps() {
        assert_eq!(parsed_key_to_code("A"), Some(Code::KeyA));
        assert_eq!(parsed_key_to_code("F12"), Some(Code::F12));
        assert_eq!(parsed_key_to_code("Space"), Some(Code::Space));
        assert_eq!(parsed_key_to_code("Num0"), Some(Code::Digit0));
        assert!(parsed_key_to_code("UnknownKey").is_none());
    }

    #[test]
    fn key_to_egui_maps() {
        assert_eq!(parsed_key_to_egui_key("A"), Some(egui::Key::A));
        assert_eq!(parsed_key_to_egui_key("F12"), Some(egui::Key::F12));
        assert_eq!(parsed_key_to_egui_key("Space"), Some(egui::Key::Space));
        assert_eq!(parsed_key_to_egui_key("Num0"), Some(egui::Key::Num0));
        assert!(parsed_key_to_egui_key("UnknownKey").is_none());
    }
}
