use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Debug)]
pub enum Language {
    Zh,
    En,
    Ja,
}

impl Default for Language {
    fn default() -> Self {
        Language::Zh
    }
}

impl Language {
    pub fn as_str(&self) -> &'static str {
        match self {
            Language::Zh => "中文",
            Language::En => "English",
            Language::Ja => "日本語",
        }
    }
}

pub struct TextBundle {
    // Menu
    pub menu_file: &'static str,
    pub menu_open_file: &'static str,
    pub menu_open_folder: &'static str,
    pub menu_settings: &'static str,
    pub menu_about: &'static str,

    // Context Menu
    pub context_copy_image: &'static str,
    pub context_copy_path: &'static str,

    // Settings
    pub settings_title: &'static str,
    pub settings_language: &'static str,
    pub settings_close: &'static str,

    // About
    pub about_title: &'static str,
    pub about_desc: &'static str,
    pub about_github: &'static str,
    pub about_close: &'static str,

    // Viewer
    pub viewer_error: &'static str,
    pub viewer_drag_hint: &'static str,

    // Loading
    pub loading_parsing: &'static str,
}

pub const ZH_TEXT: TextBundle = TextBundle {
    menu_file: "文件",
    menu_open_file: "打开文件…",
    menu_open_folder: "打开文件夹…",
    menu_settings: "设置",
    menu_about: "关于",
    context_copy_image: "复制图片",
    context_copy_path: "复制图片路径",
    settings_title: "设置",
    settings_language: "语言",
    settings_close: "关闭",
    about_title: "关于项目",
    about_desc: "Rust 图片查看器",
    about_github: "GitHub 源码地址",
    about_close: "我知道了",
    viewer_error: "文件损坏或格式不支持",
    viewer_drag_hint: "拖拽或打开图片",
    loading_parsing: "正在解析像素...",
};

pub const EN_TEXT: TextBundle = TextBundle {
    menu_file: "File",
    menu_open_file: "Open File...",
    menu_open_folder: "Open Folder...",
    menu_settings: "Settings",
    menu_about: "About",
    context_copy_image: "Copy Image",
    context_copy_path: "Copy Image Path",
    settings_title: "Settings",
    settings_language: "Language",
    settings_close: "Close",
    about_title: "About",
    about_desc: "Rust Image Viewer",
    about_github: "GitHub Repository",
    about_close: "Close",
    viewer_error: "File damaged or format not supported",
    viewer_drag_hint: "Drag or open image",
    loading_parsing: "Parsing pixels...",
};

pub const JA_TEXT: TextBundle = TextBundle {
    menu_file: "ファイル",
    menu_open_file: "ファイルを開く...",
    menu_open_folder: "フォルダを開く...",
    menu_settings: "設定",
    menu_about: "について",
    context_copy_image: "画像をコピー",
    context_copy_path: "画像パスをコピー",
    settings_title: "設定",
    settings_language: "言語",
    settings_close: "閉じる",
    about_title: "プロジェクトについて",
    about_desc: "Rust 画像ビューア",
    about_github: "GitHub ソースコード",
    about_close: "閉じる",
    viewer_error: "ファイルが破損しているか、形式がサポートされていません",
    viewer_drag_hint: "画像をドラッグまたは開く",
    loading_parsing: "ピクセルを解析中...",
};

pub fn get_text(lang: Language) -> &'static TextBundle {
    match lang {
        Language::Zh => &ZH_TEXT,
        Language::En => &EN_TEXT,
        Language::Ja => &JA_TEXT,
    }
}
