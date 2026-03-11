use std::collections::HashMap;
use std::sync::Arc;
use eframe::egui::{Color32, ColorImage, Pos2, Rect, TextureHandle};
use image::RgbaImage;

use crate::model::device::MonitorInfo;
use crate::ui::screenshot::color_picker::ColorPicker;

#[derive(PartialEq, Clone, Copy)]
pub enum ScreenshotAction {
    None,
    Close,
    SaveAndClose,
    SaveToClipboard,
}

#[derive(PartialEq, Clone, Copy)]
pub enum ScreenshotTool {
    Rect,
    Circle,
}

#[derive(Clone)]
pub struct DrawnShape {
    pub tool: ScreenshotTool,
    pub start: Pos2,
    pub end: Pos2,
    pub color: Color32,
    pub stroke_width: f32,
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
}

#[derive(Clone)]
pub struct HistoryEntry {
    pub shapes: Vec<DrawnShape>,
    pub selection: Option<Rect>,
}

impl Default for ScreenshotState {
    fn default() -> Self {
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
            color_picker: ColorPicker::new(default_color),
            color_picker_anchor: None,
            shapes: Vec::new(),
            current_shape_start: None,
            current_shape_end: None,
            copy_requested: false,
            texture_pool: HashMap::new(),

            window_configured: false,
            prev_window_state: WindowPrevState::Normal,
            history: Vec::new(),
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
