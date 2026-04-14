use crate::model::config::get_context_config;
use egui::Context;
use serde::{Deserialize, Serialize};
use sys_locale::get_locale;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Language {
    Zh,
    En,
    Ja,
}

impl Language {
    pub fn as_str(&self) -> &'static str {
        match self {
            Language::Zh => "中文",
            Language::En => "English",
            Language::Ja => "日本語",
        }
    }

    pub fn detect_system() -> Self {
        get_locale()
            .as_deref()
            .and_then(Self::from_locale_tag)
            .unwrap_or(Self::Zh)
    }

    fn from_locale_tag(locale: &str) -> Option<Self> {
        let normalized = locale.replace('_', "-").to_ascii_lowercase();
        let primary = normalized.split('-').next()?;

        if primary == "zh" {
            return Some(Self::Zh);
        }

        if primary == "ja" {
            return Some(Self::Ja);
        }

        if primary == "en" {
            return Some(Self::En);
        }

        None
    }
}

impl Default for Language {
    fn default() -> Self {
        Self::detect_system()
    }
}

pub struct MenuText {
    pub file: &'static str,
    pub open_file: &'static str,
    pub open_folder: &'static str,
    pub edit: &'static str,
    pub help: &'static str,
    pub screenshot: &'static str,
    pub settings: &'static str,
    pub about: &'static str,
    pub exit: &'static str,
}

pub struct ContextMenuText {
    pub copy: &'static str,
    pub copy_path: &'static str,
    pub properties: &'static str,
}

pub struct SettingsText {
    pub title: &'static str,
    pub general: &'static str,
    pub language: &'static str,
    pub minimize_on_close: &'static str,
    pub magnifier_enabled: &'static str,
    pub screenshot_hides_main_window: &'static str,
    pub close: &'static str,
    pub apply: &'static str,
    pub shortcut_key: &'static str,
}

pub struct ShortcutText {
    pub screenshot: &'static str,
    pub copy_color: &'static str,
    pub modified: &'static str,
}

pub struct AboutText {
    pub title: &'static str,
    pub description: &'static str,
    pub github: &'static str,
    pub close: &'static str,
    pub thankful_head: &'static str,
    pub thankful_main: &'static str,
}

pub struct ViewerText {
    pub error: &'static str,
    pub drag_hint: &'static str,
    pub no_images: &'static str,
}

pub struct LoadingText {
    pub parsing: &'static str,
}

pub struct ToastText {
    pub copied: &'static str,
    pub copy_failed: &'static str,
    pub copying: &'static str,
}

pub struct ImageText {
    pub properties: &'static str,
    pub name: &'static str,
    pub date: &'static str,
    pub path: &'static str,
    pub dimensions: &'static str,
}

pub struct StatusText {
    pub grid: &'static str,
    pub single: &'static str,
}

pub struct TooltipText {
    pub draw_text: &'static str,
    pub draw_rect: &'static str,
    pub draw_circle: &'static str,
    pub draw_arrow: &'static str,
    pub draw_pencil: &'static str,
    pub draw_mosaic: &'static str,
    pub ocr: &'static str,
    pub cancel: &'static str,
    pub save: &'static str,
    pub save_to_clipboard: &'static str,
    pub mouse_copy_color: &'static str,
}

pub struct HelpText {
    pub shortcuts: &'static str,
    pub esc: &'static str,
    pub undo: &'static str,
    pub redo: &'static str,
    pub copy: &'static str,
    pub tools: &'static str,
}

pub struct OcrText {
    pub title: &'static str,
    pub processing: &'static str,
    pub copy_all: &'static str,
    pub engine_failed: &'static str,
}

pub struct PropertiesText {
    pub no_image: &'static str,
}

pub struct GridText {
    pub loading: &'static str,
}

pub struct MagnifierText {
    pub pos: &'static str,
    pub hex: &'static str,
}

pub struct TextBundle {
    pub menu: MenuText,
    pub context_menu: ContextMenuText,
    pub settings: SettingsText,
    pub shortcuts: ShortcutText,
    pub about: AboutText,
    pub viewer: ViewerText,
    pub loading: LoadingText,
    pub toast: ToastText,
    pub image: ImageText,
    pub status: StatusText,
    pub tooltip: TooltipText,
    pub help: HelpText,
    pub ocr: OcrText,
    pub properties: PropertiesText,
    pub grid: GridText,
    pub magnifier: MagnifierText,
}

pub const ZH_TEXT: TextBundle = TextBundle {
    menu: MenuText {
        file: "文件",
        open_file: "打开文件…",
        open_folder: "打开文件夹…",
        edit: "编辑",
        help: "帮助",
        screenshot: "截图",
        settings: "设置",
        about: "关于",
        exit: "退出",
    },
    context_menu: ContextMenuText {
        copy: "复制",
        copy_path: "复制路径",
        properties: "属性",
    },
    settings: SettingsText {
        title: "设置",
        general: "常规",
        language: "语言",
        minimize_on_close: "关闭窗口时最小化到托盘",
        magnifier_enabled: "启用放大镜",
        screenshot_hides_main_window: "截图后隐藏主窗口",
        close: "关闭",
        apply: "应用",
        shortcut_key: "快捷键",
    },
    shortcuts: ShortcutText {
        screenshot: "截图",
        copy_color: "复制颜色",
        modified: "请按下按键...",
    },
    about: AboutText {
        title: "关于项目",
        description: "Rust实现的图片查看器和截图工具",
        github: "GitHub 源码地址",
        close: "我知道了",
        thankful_head: "献给我的妻子",
        thankful_main: "谢谢她不嫌弃我把无数个周末都花在对着电脑屏幕发呆上",
    },
    viewer: ViewerText {
        error: "文件损坏或格式不支持",
        drag_hint: "拖拽或打开文件夹",
        no_images: "该文件夹下没有图片",
    },
    loading: LoadingText {
        parsing: "正在解析像素...",
    },
    toast: ToastText {
        copied: "已复制",
        copy_failed: "复制失败",
        copying: "正在复制中...",
    },
    image: ImageText {
        properties: "属性",
        name: "名称",
        date: "日期",
        path: "图片路径",
        dimensions: "图片尺寸",
    },
    status: StatusText {
        grid: "网格视图",
        single: "单图视图",
    },
    tooltip: TooltipText {
        draw_text: "文字",
        draw_rect: "矩形",
        draw_circle: "圆形",
        draw_arrow: "箭头",
        draw_pencil: "铅笔",
        draw_mosaic: "马赛克",
        ocr: "提取文字 (OCR)",
        cancel: "取消",
        save: "保存到桌面",
        save_to_clipboard: "复制到剪贴板",
        mouse_copy_color: "按 Ctrl + C 复制颜色",
    },
    help: HelpText {
        shortcuts: "【快捷键】",
        esc: "Esc : 退出截图",
        undo: "Ctrl+Z : 撤销上一步绘制",
        redo: "Ctrl+Y / Ctrl+Shift+Z : 重做上一步绘制",
        copy: "复制截图",
        tools: "【工具说明】",
    },
    ocr: OcrText {
        title: "📝 文字识别 (OCR)",
        processing: "正在提取文字，请稍候...",
        copy_all: "📋 复制全部到剪贴板",
        engine_failed: "OCR 引擎调用失败: ",
    },
    properties: PropertiesText {
        no_image: "未加载图片。",
    },
    grid: GridText {
        loading: "加载中...",
    },
    magnifier: MagnifierText {
        pos: "坐标: ",
        hex: "色值: ",
    },
};

pub const EN_TEXT: TextBundle = TextBundle {
    menu: MenuText {
        file: "File",
        open_file: "Open File...",
        open_folder: "Open Folder...",
        edit: "Edit",
        help: "Help",
        screenshot: "Screenshot",
        settings: "Settings",
        about: "About",
        exit: "Exit",
    },
    context_menu: ContextMenuText {
        copy: "Copy",
        copy_path: "Copy Path",
        properties: "Properties",
    },
    settings: SettingsText {
        title: "Settings",
        general: "General",
        language: "Language",
        minimize_on_close: "Minimize to tray on close",
        magnifier_enabled: "Enable Magnifier",
        screenshot_hides_main_window: "Hide main window after screenshot",
        close: "Close",
        apply: "Apply",
        shortcut_key: "Keyboard Shortcut",
    },
    shortcuts: ShortcutText {
        screenshot: "Screenshot",
        copy_color: "Copy Color",
        modified: "Please Press The Key…",
    },
    about: AboutText {
        title: "About",
        description: "Image viewer and screenshot tool implemented in Rust",
        github: "GitHub Repository",
        close: "Close",
        thankful_head: "Dedicated to my wife",
        thankful_main: "who didn't mind me spending countless weekends staring at a monitor",
    },
    viewer: ViewerText {
        error: "File damaged or format not supported",
        drag_hint: "Drag and drop or open a folder",
        no_images: "No images in this folder",
    },
    loading: LoadingText {
        parsing: "Parsing pixels...",
    },
    toast: ToastText {
        copied: "Copied",
        copy_failed: "Copy failed",
        copying: "Copying...",
    },
    image: ImageText {
        properties: "Properties",
        name: "Name",
        date: "Datetime",
        path: "File Path",
        dimensions: "Dimension",
    },
    status: StatusText {
        grid: "Grid View",
        single: "Single View",
    },
    tooltip: TooltipText {
        draw_text: "Font",
        draw_rect: "Rectangle",
        draw_circle: "Circle",
        draw_arrow: "Arrow",
        draw_pencil: "Pencil",
        draw_mosaic: "Mosaic",
        ocr: "Extract Text (OCR)",
        cancel: "Cancel",
        save: "Save to Desktop",
        save_to_clipboard: "Copy to Clipboard",
        mouse_copy_color: "Press Ctrl + C Copy Color",
    },
    help: HelpText {
        shortcuts: "[ Shortcuts ]",
        esc: "Esc : Exit Screenshot",
        undo: "Ctrl + Z : Undo Drawing",
        redo: "Ctrl + Y / Ctrl + Shift + Z : Redo Drawing",
        copy: "Copy Screenshot",
        tools: "[ Tools ]",
    },
    ocr: OcrText {
        title: "📝 Text Recognition (OCR)",
        processing: "Extracting text, please wait...",
        copy_all: "📋 Copy All to Clipboard",
        engine_failed: "OCR engine error: ",
    },
    properties: PropertiesText {
        no_image: "No image loaded.",
    },
    grid: GridText {
        loading: "Loading...",
    },
    magnifier: MagnifierText {
        pos: "POS: ",
        hex: "HEX: ",
    },
};

pub const JA_TEXT: TextBundle = TextBundle {
    menu: MenuText {
        file: "ファイル",
        open_file: "ファイルを開く...",
        open_folder: "フォルダを開く...",
        edit: "編集",
        help: "ヘルプ",
        screenshot: "スクリーンショット",
        settings: "設定",
        about: "について",
        exit: "終了",
    },
    context_menu: ContextMenuText {
        copy: "コピー",
        copy_path: "パスコピー",
        properties: "プロパティ",
    },
    settings: SettingsText {
        title: "設定",
        general: "一般",
        language: "言語",
        minimize_on_close: "閉じるときにトレイに最小化",
        magnifier_enabled: "虫眼鏡を有効にする",
        screenshot_hides_main_window: "スクリーンショット後にメインウィンドウを隠す",
        close: "閉じる",
        apply: "設定",
        shortcut_key: "ショートカットキー",
    },
    shortcuts: ShortcutText {
        screenshot: "スクリーンショット",
        copy_color: "カラーコピー",
        modified: "キーを押してください…",
    },
    about: AboutText {
        title: "プロジェクトについて",
        description: "Rust製の画像ビューアおよびスクリーンショットツール",
        github: "GitHub ソースコード",
        close: "閉じる",
        thankful_head: "私の妻に捧げます",
        thankful_main: "数え切れないほどの週末をモニターの前で過ごす私を、文句も言わずに見守ってくれたことに感謝して",
    },
    viewer: ViewerText {
        error: "ファイルが破損しているか、形式がサポートされていません",
        drag_hint: "ドラッグ＆ドロップまたはフォルダを開く",
        no_images: "このフォルダに画像はありません",
    },
    loading: LoadingText {
        parsing: "ピクセルを解析中...",
    },
    toast: ToastText {
        copied: "コピーしました",
        copy_failed: "コピーに失敗しました",
        copying: "コピー中...",
    },
    image: ImageText {
        properties: "プロパティーズ",
        name: "ネーム",
        date: "日付",
        path: "ファイルパス",
        dimensions: "ディメンション",
    },
    status: StatusText {
        grid: "グリッド表示",
        single: "単一表示",
    },
    tooltip: TooltipText {
        draw_text: "文字",
        draw_rect: "矩形",
        draw_circle: "円形",
        draw_arrow: "矢印",
        draw_pencil: "鉛筆",
        draw_mosaic: "モザイク",
        ocr: "テキスト抽出 (OCR)",
        cancel: "キャンセル",
        save: "デスクトップに保存",
        save_to_clipboard: "クリップボードにコピー",
        mouse_copy_color: "Ctrl + C を押して色をコピー",
    },
    help: HelpText {
        shortcuts: "【ショートカット】",
        esc: "Esc : スクリーンショットを終了",
        undo: "Ctrl + Z : 元に戻す",
        redo: "Ctrl + Y / Ctrl + Shift + Z : やり直す",
        copy: "スクリーンショットをコピー",
        tools: "【ツール】",
    },
    ocr: OcrText {
        title: "📝 文字認識 (OCR)",
        processing: "テキストを抽出中です、お待ちください...",
        copy_all: "📋 クリップボードにすべてコピー",
        engine_failed: "OCR エンジンエラー: ",
    },
    properties: PropertiesText {
        no_image: "画像が読み込まれていません。",
    },
    grid: GridText {
        loading: "読み込み中...",
    },
    magnifier: MagnifierText {
        pos: "座標: ",
        hex: "カラー: ",
    },
};

pub fn get_text(lang: Language) -> &'static TextBundle {
    match lang {
        Language::Zh => &ZH_TEXT,
        Language::En => &EN_TEXT,
        Language::Ja => &JA_TEXT,
    }
}

pub fn get_i18n_text(ctx: &Context) -> &'static TextBundle {
    let config = get_context_config(ctx);
    get_text(config.language)
}

#[cfg(test)]
mod tests {
    use super::Language;

    #[test]
    fn locale_tag_maps_to_supported_language() {
        assert_eq!(Language::from_locale_tag("zh-CN"), Some(Language::Zh));
        assert_eq!(Language::from_locale_tag("zh_Hant_TW"), Some(Language::Zh));
        assert_eq!(Language::from_locale_tag("en-US"), Some(Language::En));
        assert_eq!(Language::from_locale_tag("ja-JP"), Some(Language::Ja));
    }

    #[test]
    fn unsupported_locale_returns_none() {
        assert_eq!(Language::from_locale_tag("fr-FR"), None);
    }
}
