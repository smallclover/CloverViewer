use eframe::egui::{Color32, ColorImage, Pos2, Rect, TextureHandle};
use image::RgbaImage;
use std::collections::HashMap;
use std::sync::Arc;

use crate::feature::screenshot::color_picker::ColorPicker;
use crate::model::device::MonitorInfo;

const DEFAULT_ACTIVE_COLOR: Color32 = Color32::from_rgb(204, 0, 0);
const DEFAULT_STROKE_WIDTH: f32 = 2.0;
const DEFAULT_MOSAIC_WIDTH: f32 = 16.0;

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
    Mosaic,
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
}

pub struct ScreenshotState {
    pub capture: ScreenshotCaptureState,
    pub select: ScreenshotSelectionState,
    pub drawing: ScreenshotDrawingState,
    pub edit: ScreenshotEditState,
    pub runtime: ScreenshotRuntimeState,
    pub input: ScreenshotInputState,
}

pub struct ScreenshotEditState {
    pub shapes: Vec<DrawnShape>,
    // 撤销功能：历史栈
    pub history: Vec<HistoryEntry>,
}

impl Default for ScreenshotEditState {
    fn default() -> Self {
        Self {
            shapes: Vec::new(),
            history: Vec::new(),
        }
    }
}

pub struct ScreenshotCaptureState {
    pub captures: Vec<CapturedScreen>,
    pub window_rects: Vec<Rect>,
    pub is_capturing: bool,
    pub capture_receiver: Option<std::sync::mpsc::Receiver<(Vec<CapturedScreen>, Vec<Rect>)>>,
    pub texture_pool: HashMap<String, TextureHandle>,
}

impl Default for ScreenshotCaptureState {
    fn default() -> Self {
        Self {
            captures: Vec::new(),
            window_rects: Vec::new(),
            is_capturing: false,
            capture_receiver: None,
            texture_pool: HashMap::new(),
        }
    }
}

pub struct ScreenshotSelectionState {
    pub selection: Option<Rect>,
    pub drag_start: Option<Pos2>,
    pub toolbar_pos: Option<Pos2>,
    pub hovered_window: Option<Rect>,
}

impl Default for ScreenshotSelectionState {
    fn default() -> Self {
        Self {
            selection: None,
            drag_start: None,
            toolbar_pos: None,
            hovered_window: None,
        }
    }
}

pub struct ScreenshotDrawingState {
    pub current_tool: Option<ScreenshotTool>,
    pub active_color: Color32,
    pub stroke_width: f32,
    pub mosaic_width: f32,
    pub color_picker: ColorPicker,
    pub color_picker_anchor: Option<Rect>,
}

impl Default for ScreenshotDrawingState {
    fn default() -> Self {
        Self {
            current_tool: None,
            active_color: DEFAULT_ACTIVE_COLOR,
            stroke_width: DEFAULT_STROKE_WIDTH,
            mosaic_width: DEFAULT_MOSAIC_WIDTH,
            color_picker: ColorPicker::new(DEFAULT_ACTIVE_COLOR),
            color_picker_anchor: None,
        }
    }
}

pub struct ScreenshotRuntimeState {
    pub window_configured: bool,
    pub prev_window_state: WindowPrevState,
}

impl ScreenshotRuntimeState {
    fn new(prev_window_state: WindowPrevState) -> Self {
        Self {
            window_configured: false,
            prev_window_state,
        }
    }
}

pub struct ScreenshotInputState {
    pub copy_requested: bool,
    pub current_shape_start: Option<Pos2>,
    pub current_shape_end: Option<Pos2>,
    // 记录文本输入状态：Option<(文本所在的物理坐标, 文本内容)>
    pub active_text_input: Option<(Pos2, String)>,
    // 当前正在绘制的画笔轨迹
    pub current_pen_points: Vec<Pos2>,
}

impl Default for ScreenshotInputState {
    fn default() -> Self {
        Self {
            copy_requested: false,
            current_shape_start: None,
            current_shape_end: None,
            active_text_input: None,
            current_pen_points: Vec::new(),
        }
    }
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
        Self {
            capture: ScreenshotCaptureState::default(),
            select: ScreenshotSelectionState::default(),
            drawing: ScreenshotDrawingState::default(),
            edit: ScreenshotEditState::default(),
            runtime: ScreenshotRuntimeState::new(prev_state),
            input: ScreenshotInputState::default(),
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
