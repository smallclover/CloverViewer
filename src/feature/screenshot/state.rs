use std::collections::HashMap;
use std::sync::Arc;
use eframe::egui::{Color32, ColorImage, Pos2, Rect, TextureHandle};
use image::RgbaImage;

use crate::model::device::MonitorInfo;
use crate::feature::screenshot::color_picker::ColorPicker;

#[derive(PartialEq, Clone, Copy)]
pub enum ScreenshotAction {
    None,
    Close,
    SaveAndClose,
    SaveToClipboard,
    Ocr,
}

#[derive(PartialEq, Clone, Copy)]
pub enum ScreenshotTool {
    Rect,
    Circle,
    Arrow,
    Text,
    Pen,
    Mosaic
}

#[derive(Clone)]
pub struct DrawnShape {
    pub tool: ScreenshotTool,
    pub start: Pos2,
    pub end: Pos2,
    pub color: Color32,
    pub stroke_width: f32,
    pub text: Option<String>,
    pub points: Option<Vec<Pos2>>,
    /// 运行时缓存：文本的 egui Galley，避免每帧重排版
    pub cached_galley: Option<Arc<egui::Galley>>,
    /// 运行时缓存：马赛克的纹理，避免每帧采样原图
    pub cached_mosaic: Option<Arc<MosaicCache>>,
}

/// 马赛克纹理缓存
#[derive(Clone)]
pub struct MosaicCache {
    /// 纹理句柄
    pub texture: TextureHandle,
    /// 纹理对应的物理坐标范围
    pub phys_rect: Rect,
    /// 缩放比例（ppp）
    pub ppp: f32,
}

pub struct ScreenshotState {
    pub captures: Vec<CapturedScreen>,
    pub selection: Option<Rect>,
    pub drag_start: Option<Pos2>,
    pub toolbar_pos: Option<Pos2>,
    pub window_rects: Vec<Rect>,
    pub hovered_window: Option<Rect>,
    pub is_capturing: bool,
    pub capture_receiver: Option<std::sync::mpsc::Receiver<(Vec<CapturedScreen>, Vec<Rect>)>>,
    pub current_tool: Option<ScreenshotTool>,
    pub active_color: Color32,
    pub stroke_width: f32,
    pub mosaic_width: f32, // 马赛克专用的粗细度
    pub color_picker: ColorPicker,
    pub color_picker_anchor: Option<Rect>,
    pub shapes: Vec<DrawnShape>,
    pub current_shape_start: Option<Pos2>,
    pub current_shape_end: Option<Pos2>,
    pub copy_requested: bool,
    pub texture_pool: HashMap<String, TextureHandle>,

    pub window_configured: bool,
    pub prev_window_state: WindowPrevState,

    // 撤销功能：历史栈
    pub history: Vec<HistoryEntry>,
    // 记录文本输入状态：Option<(文本所在的物理坐标, 文本内容)>
    pub active_text_input: Option<(Pos2, String)>,
    // 当前正在绘制的画笔轨迹
    pub current_pen_points: Vec<Pos2>,
}

#[derive(Clone)]
pub struct HistoryEntry {
    pub shapes: Vec<DrawnShape>,
    pub selection: Option<Rect>,
}

impl Default for ScreenshotState {
    fn default() -> Self {
        Self::new(WindowPrevState::Normal)
    }
}

impl ScreenshotState {
    pub fn new(prev_state: WindowPrevState) -> Self {
        let default_color = Color32::from_rgb(204, 0, 0);
        Self {
            captures: Vec::new(),
            selection: None,
            drag_start: None,
            toolbar_pos: None,
            window_rects: Vec::new(),
            hovered_window: None,
            is_capturing: false,
            capture_receiver: None,
            current_tool: None,
            active_color: default_color,
            stroke_width: 2.0,
            mosaic_width: 16.0, // 【新增】马赛克默认非常粗
            color_picker: ColorPicker::new(default_color),
            color_picker_anchor: None,
            shapes: Vec::new(),
            current_shape_start: None,
            current_shape_end: None,
            copy_requested: false,
            texture_pool: HashMap::new(),

            window_configured: false,
            prev_window_state: prev_state,
            history: Vec::new(),
            active_text_input: None,
            current_pen_points: Vec::new(),
        }
    }
}

#[derive(Clone)]
pub struct CapturedScreen {
    pub raw_image: Arc<RgbaImage>,
    pub image: ColorImage,
    pub screen_info: MonitorInfo,
}

#[derive(PartialEq, Clone, Copy, Debug)]
pub enum WindowPrevState {
    Normal,
    Minimized,
    Tray,
}