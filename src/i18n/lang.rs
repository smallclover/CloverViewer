use egui::Context;
use serde::{Deserialize, Serialize};
use crate::model::config::get_context_config;

#[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
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
    pub menu_edit: &'static str,
    pub menu_help: &'static str,
    pub menu_screenshot: &'static str,
    pub menu_settings: &'static str,
    pub menu_about: &'static str,
    pub menu_exit: &'static str,

    // Context Menu
    pub context_menu_copy: &'static str,
    pub context_menu_copy_path: &'static str,
    pub context_menu_properties: &'static str,

    // Settings
    pub settings_title: &'static str,
    pub settings_general: &'static str,
    pub settings_language: &'static str,
    pub settings_minimize_on_close: &'static str,
    pub settings_magnifier_enabled: &'static str,
    pub settings_screenshot_hides_main_window: &'static str,
    pub settings_close: &'static str,
    pub settings_apply: &'static str,
    pub settings_shortcut_key: &'static str,

    pub shortcut_key_screenshot: &'static str,
    pub shortcut_key_copy_color: &'static str,
    pub shortcut_key_modified: &'static str,
    // About
    pub about_title: &'static str,
    pub about_desc: &'static str,
    pub about_github: &'static str,
    pub about_close: &'static str,
    pub about_thankful_head: &'static str,
    pub about_thankful_main: &'static str,

    // Viewer
    pub viewer_error: &'static str,
    pub viewer_drag_hint: &'static str,
    pub viewer_no_images: &'static str,

    // Loading
    pub loading_parsing: &'static str,

    // Toast
    pub copied_message: &'static str,
    pub copy_failed_message: &'static str,
    pub coping_message: &'static str,

    // 属性
    pub img_prop: &'static str,
    pub img_name: &'static str,
    pub img_date: &'static str,
    pub img_path: &'static str,
    // pub img_size:  &'static str,
    pub img_dim: &'static str,

    // 状态栏
    pub status_gird: &'static str,
    pub status_single: &'static str,

    // Tooltips
    pub tooltip_draw_text: &'static str,
    pub tooltip_draw_rect: &'static str,
    pub tooltip_draw_circle: &'static str,
    pub tooltip_draw_arrow: &'static str,
    pub tooltip_draw_pencil: &'static str,
    pub tooltip_draw_mosaic: &'static str,
    pub tooltip_ocr: &'static str,
    pub tooltip_cancel: &'static str,
    pub tooltip_save: &'static str,
    pub tooltip_save_to_clipboard: &'static str,
    pub tooltip_mouse_copy_color: &'static str,

    // help box
    pub help_shortcuts: &'static str,
    pub help_esc: &'static str,
    pub help_undo: &'static str,
    pub help_copy: &'static str,
    pub help_tools: &'static str,

    // OCR
    pub ocr_title: &'static str,
    pub ocr_processing: &'static str,
    pub ocr_copy_all: &'static str,
    pub ocr_engine_failed: &'static str,

    // Properties
    pub prop_no_image: &'static str,

    // Grid
    pub grid_loading: &'static str,

    // Magnifier
    pub magnifier_pos: &'static str,
    pub magnifier_hex: &'static str,
}

pub const ZH_TEXT: TextBundle = TextBundle {
    menu_file: "文件",
    menu_open_file: "打开文件…",
    menu_open_folder: "打开文件夹…",
    menu_edit: "编辑",
    menu_help: "帮助",
    menu_screenshot: "截图",
    menu_settings: "设置",
    menu_about: "关于",
    menu_exit: "退出",

    context_menu_copy: "复制",
    context_menu_copy_path: "复制路径",
    context_menu_properties: "属性",

    settings_title: "设置",
    settings_general: "常规",
    settings_language: "语言",
    settings_minimize_on_close: "关闭窗口时最小化到托盘",
    settings_magnifier_enabled: "启用放大镜",
    settings_screenshot_hides_main_window: "截图后隐藏主窗口",
    settings_close: "关闭",
    settings_apply: "应用",

    settings_shortcut_key: "快捷键",
    shortcut_key_screenshot: "截图",
    shortcut_key_copy_color: "复制颜色",
    shortcut_key_modified: "请按下按键...",

    about_title: "关于项目",
    about_desc: "Rust实现的图片查看器和截图工具",
    about_github: "GitHub 源码地址",
    about_close: "我知道了",
    about_thankful_head: "献给我的妻子",
    about_thankful_main: "谢谢她不嫌弃我把无数个周末都花在对着电脑屏幕发呆上",

    viewer_error: "文件损坏或格式不支持",
    viewer_drag_hint: "拖拽或打开文件夹",
    viewer_no_images: "该文件夹下没有图片",
    loading_parsing: "正在解析像素...",
    copied_message: "已复制",
    copy_failed_message: "复制失败",
    coping_message: "正在复制中...",
    img_prop: "属性",

    img_name: "名称",
    img_date: "日期",
    img_path: "图片路径",
    // img_size: "图片大小",
    img_dim: "图片尺寸",
    status_gird: "网格视图",
    status_single: "单图视图",

    tooltip_draw_text: "文字",
    tooltip_draw_rect: "矩形",
    tooltip_draw_circle: "圆形",
    tooltip_draw_arrow: "箭头",
    tooltip_draw_pencil: "铅笔",
    tooltip_draw_mosaic: "马赛克",
    tooltip_ocr: "提取文字 (OCR)",
    tooltip_cancel: "取消",
    tooltip_save: "保存到桌面",
    tooltip_save_to_clipboard: "复制到剪贴板",
    tooltip_mouse_copy_color: "按 Ctrl + C 复制颜色",

    help_shortcuts: "【快捷键】",
    help_esc: "Esc : 退出截图",
    help_undo: "Ctrl+Z : 撤销上一步绘制",
    help_copy: "复制截图",
    help_tools: "【工具说明】",

    ocr_title: "📝 文字识别 (OCR)",
    ocr_processing: "正在提取文字，请稍候...",
    ocr_copy_all: "📋 复制全部到剪贴板",
    ocr_engine_failed: "OCR 引擎调用失败: ",

    prop_no_image: "未加载图片。",
    grid_loading: "加载中...",
    magnifier_pos: "坐标: ",
    magnifier_hex: "色值: ",
};

pub const EN_TEXT: TextBundle = TextBundle {
    menu_file: "File",
    menu_open_file: "Open File...",
    menu_open_folder: "Open Folder...",
    menu_edit: "Edit",
    menu_help: "Help",
    menu_screenshot: "Screenshot",
    menu_settings: "Settings",
    menu_about: "About",
    menu_exit: "Exit",

    context_menu_copy: "Copy",
    context_menu_copy_path: "Copy Path",
    context_menu_properties: "Properties",

    settings_title: "Settings",
    settings_general: "General",
    settings_language: "Language",
    settings_minimize_on_close: "Minimize to tray on close",
    settings_magnifier_enabled: "Enable Magnifier",
    settings_screenshot_hides_main_window: "Hide main window after screenshot",
    settings_close: "Close",
    settings_apply: "Apply",

    settings_shortcut_key: "Keyboard Shortcut",
    shortcut_key_screenshot: "Screenshot",
    shortcut_key_copy_color: "Copy Color",
    shortcut_key_modified: "Please Press The Key…",

    about_title: "About",
    about_desc: "Image viewer and screenshot tool implemented in Rust",
    about_github: "GitHub Repository",
    about_close: "Close",
    about_thankful_head: "Dedicated to my wife",
    about_thankful_main: "who didn't mind me spending countless weekends staring at a monitor",

    viewer_error: "File damaged or format not supported",
    viewer_drag_hint: "Drag and drop or open a folder",
    viewer_no_images: "No images in this folder",
    loading_parsing: "Parsing pixels...",
    copied_message: "Copied",
    copy_failed_message: "Copy failed",
    coping_message: "Coping...",

    img_prop: "Properties",
    img_name: "Name",
    img_date: "Datetime",
    img_path: "File Path",
    // img_size: "Size",
    img_dim: "Dimension",
    status_gird: "Grid View",
    status_single: "Single View",

    tooltip_draw_text: "Font",
    tooltip_draw_rect: "Rectangle",
    tooltip_draw_circle: "Circle",
    tooltip_draw_arrow: "Arrow",
    tooltip_draw_pencil: "Pencil",
    tooltip_draw_mosaic: "Mosaic",
    tooltip_ocr: "Extract Text (OCR)",
    tooltip_cancel: "Cancel",
    tooltip_save: "Save to Desktop",
    tooltip_save_to_clipboard: "Copy to Clipboard",
    tooltip_mouse_copy_color: "Press Ctrl + C Copy Color",

    help_shortcuts: "[ Shortcuts ]",
    help_esc: "Esc : Exit Screenshot",
    help_undo: "Ctrl + Z : Undo Drawing",
    help_copy: "Copy Screenshot",
    help_tools: "[ Tools ]",

    ocr_title: "📝 Text Recognition (OCR)",
    ocr_processing: "Extracting text, please wait...",
    ocr_copy_all: "📋 Copy All to Clipboard",
    ocr_engine_failed: "OCR engine error: ",

    prop_no_image: "No image loaded.",
    grid_loading: "Loading...",
    magnifier_pos: "POS: ",
    magnifier_hex: "HEX: ",
};

pub const JA_TEXT: TextBundle = TextBundle {
    menu_file: "ファイル",
    menu_open_file: "ファイルを開く...",
    menu_open_folder: "フォルダを開く...",
    menu_edit: "編集",
    menu_help: "ヘルプ",
    menu_screenshot: "スクリーンショット",
    menu_settings: "設定",
    menu_about: "について",
    menu_exit: "終了",

    context_menu_copy: "コピー",
    context_menu_copy_path: "パスコピー",
    context_menu_properties: "プロパティ",

    settings_title: "設定",
    settings_general: "一般",
    settings_language: "言語",
    settings_minimize_on_close: "閉じるときにトレイに最小化",
    settings_magnifier_enabled: "虫眼鏡を有効にする",
    settings_screenshot_hides_main_window: "スクリーンショット後にメインウィンドウを隠す",
    settings_close: "閉じる",
    settings_apply: "設定",

    settings_shortcut_key: "ショートカットキー",
    shortcut_key_screenshot: "スクリーンショット",
    shortcut_key_copy_color: "カラーコピー",
    shortcut_key_modified: "キーを押してください…",

    about_title: "プロジェクトについて",
    about_desc: "Rust製の画像ビューアおよびスクリーンショットツール",
    about_github: "GitHub ソースコード",
    about_close: "閉じる",
    about_thankful_head: "私の妻に捧げます",
    about_thankful_main: "数え切れないほどの週末をモニターの前で過ごす私を、文句も言わずに見守ってくれたことに感謝して",

    viewer_error: "ファイルが破損しているか、形式がサポートされていません",
    viewer_drag_hint: "ドラッグ＆ドロップまたはフォルダを開く",
    viewer_no_images: "このフォルダに画像はありません",
    loading_parsing: "ピクセルを解析中...",
    copied_message: "コピーしました",
    copy_failed_message: "コピーに失敗しました",
    coping_message: "コピー中...",

    img_prop: "プロパティーズ",
    img_name: "ネーム",
    img_date: "日付",
    img_path: "ファイルパス",
    // img_size: "サイズ",
    img_dim: "ディメンション",
    status_gird: "グリッド表示",
    status_single: "単一表示",

    tooltip_draw_text: "文字",
    tooltip_draw_rect: "矩形",
    tooltip_draw_circle: "円形",
    tooltip_draw_arrow: "矢印",
    tooltip_draw_pencil: "鉛筆",
    tooltip_draw_mosaic: "モザイク",
    tooltip_ocr: "テキスト抽出 (OCR)",
    tooltip_cancel: "キャンセル",
    tooltip_save: "デスクトップに保存",
    tooltip_save_to_clipboard: "クリップボードにコピー",
    tooltip_mouse_copy_color: "Ctrl + C を押して色をコピー",

    help_shortcuts: "【ショートカット】",
    help_esc: "Esc : スクリーンショットを終了",
    help_undo: "Ctrl + Z : 元に戻す",
    help_copy: "スクリーンショットをコピー",
    help_tools: "【ツール】",

    ocr_title: "📝 文字認識 (OCR)",
    ocr_processing: "テキストを抽出中です、お待ちください...",
    ocr_copy_all: "📋 クリップボードにすべてコピー",
    ocr_engine_failed: "OCR エンジンエラー: ",

    prop_no_image: "画像が読み込まれていません。",
    grid_loading: "読み込み中...",
    magnifier_pos: "座標: ",
    magnifier_hex: "カラー: ",
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