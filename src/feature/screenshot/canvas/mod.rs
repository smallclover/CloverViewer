pub mod drag;
pub mod draw;
pub mod hit_test;
pub mod interaction;
pub mod mosaic;
pub mod render;
pub mod shape;
pub mod text_input;

use crate::feature::screenshot::capture::{DrawnShape, ScreenshotState, ScreenshotTool};
use eframe::egui::{Color32, Id, Pos2, Ui, Vec2};
use std::sync::Arc;

/// 物理坐标转换为本地逻辑坐标
pub fn phys_to_local(pos: Pos2, global_offset_phys: Pos2, ppp: f32) -> Pos2 {
    Pos2::ZERO + ((pos - global_offset_phys) / ppp)
}

/// 马赛克块大小（物理像素）
pub const MOSAIC_BLOCK_SIZE: f32 = 15.0;
/// 命中测试半径（本地坐标）
pub const HIT_TEST_RADIUS: f32 = 15.0;
/// 抓取容差最小值
pub const GRAB_TOLERANCE_MIN: f32 = 4.0;
/// 抓取容差最大值
pub const GRAB_TOLERANCE_MAX: f32 = 8.0;
/// 形状最小尺寸
pub const MIN_SHAPE_SIZE: f32 = 4.0;
/// 锚点大小
pub const ANCHOR_SIZE: f32 = 6.0;
/// 遮罩透明度
pub const OVERLAY_ALPHA: u8 = 128;

/// Resize 开始时的基准状态
#[derive(Clone, Copy, Debug)]
pub struct ResizeStartState {
    pub start: Pos2,
    pub end: Pos2,
}

/// 画布运行时状态，在帧间通过 egui temp data 持久化
#[derive(Default, Clone, Copy, Debug)]
pub struct CanvasState {
    pub hovered_shape: Option<usize>,
    pub selected_shape: Option<usize>,
    pub dragging_shape: Option<usize>,
    pub dragging_selection: bool,
    pub drag_start_phys: Option<Pos2>,
    pub dragging_handle: Option<usize>,
    pub resize_start_state: Option<ResizeStartState>,
}

impl CanvasState {
    const HOVERED_ID: &'static str = "cv_canvas_hovered_shape";
    const SELECTED_ID: &'static str = "cv_canvas_selected_shape";
    const DRAGGING_ID: &'static str = "cv_canvas_dragging_shape";
    const DRAGGING_SEL_ID: &'static str = "cv_canvas_dragging_selection";
    const DRAG_START_ID: &'static str = "cv_canvas_drag_start";
    const DRAGGING_HANDLE_ID: &'static str = "cv_canvas_dragging_handle";
    const RESIZE_START_STATE_ID: &'static str = "cv_canvas_resize_start_state";

    pub fn load_from_ui(ui: &Ui) -> Self {
        Self {
            hovered_shape: ui.data(|d| d.get_temp(Id::new(Self::HOVERED_ID))),
            selected_shape: ui.data(|d| d.get_temp(Id::new(Self::SELECTED_ID))),
            dragging_shape: ui.data(|d| d.get_temp(Id::new(Self::DRAGGING_ID))),
            dragging_selection: ui
                .data(|d| d.get_temp(Id::new(Self::DRAGGING_SEL_ID)))
                .unwrap_or(false),
            drag_start_phys: ui.data(|d| d.get_temp(Id::new(Self::DRAG_START_ID))),
            dragging_handle: ui.data(|d| d.get_temp(Id::new(Self::DRAGGING_HANDLE_ID))),
            resize_start_state: ui.data(|d| d.get_temp(Id::new(Self::RESIZE_START_STATE_ID))),
        }
    }

    pub fn save_to_ui(self, ui: &mut Ui) {
        ui.data_mut(|d| {
            if let Some(v) = self.hovered_shape {
                d.insert_temp(Id::new(Self::HOVERED_ID), v);
            } else {
                d.remove::<usize>(Id::new(Self::HOVERED_ID));
            }

            if let Some(v) = self.selected_shape {
                d.insert_temp(Id::new(Self::SELECTED_ID), v);
            } else {
                d.remove::<usize>(Id::new(Self::SELECTED_ID));
            }

            if let Some(v) = self.dragging_shape {
                d.insert_temp(Id::new(Self::DRAGGING_ID), v);
            } else {
                d.remove::<usize>(Id::new(Self::DRAGGING_ID));
            }

            d.insert_temp(Id::new(Self::DRAGGING_SEL_ID), self.dragging_selection);

            if let Some(v) = self.drag_start_phys {
                d.insert_temp(Id::new(Self::DRAG_START_ID), v);
            } else {
                d.remove::<Pos2>(Id::new(Self::DRAG_START_ID));
            }

            if let Some(v) = self.dragging_handle {
                d.insert_temp(Id::new(Self::DRAGGING_HANDLE_ID), v);
            } else {
                d.remove::<usize>(Id::new(Self::DRAGGING_HANDLE_ID));
            }

            if let Some(v) = self.resize_start_state {
                d.insert_temp(Id::new(Self::RESIZE_START_STATE_ID), v);
            } else {
                d.remove::<ResizeStartState>(Id::new(Self::RESIZE_START_STATE_ID));
            }
        });
    }
}

/// 提交文本输入为一个 DrawnShape
pub fn commit_text_shape(
    ui: &Ui,
    state: &mut ScreenshotState,
    pos: Pos2,
    text: String,
    global_offset_phys: Pos2,
    ppp: f32,
) {
    let font_size = 20.0 + (state.drawing.stroke_width * 2.0);
    let max_width_logical = if let Some(sel) = state.select.selection {
        let sel_max_x_local = Pos2::ZERO.x + ((sel.max.x - global_offset_phys.x) / ppp);
        let start_local_x = Pos2::ZERO.x + ((pos.x - global_offset_phys.x) / ppp);
        (sel_max_x_local - start_local_x - 16.0).max(20.0)
    } else {
        1000.0
    };

    let galley = ui.painter().layout(
        text.clone(),
        eframe::egui::FontId::proportional(font_size),
        Color32::WHITE,
        max_width_logical,
    );

    let mut baked_text = String::new();
    let rows_len = galley.rows.len();
    for (i, row) in galley.rows.iter().enumerate() {
        let mut row_str = String::new();
        for glyph in &row.glyphs {
            row_str.push(glyph.chr);
        }
        baked_text.push_str(row_str.trim_end_matches(&['\r', '\n'][..]));
        if i < rows_len - 1 {
            baked_text.push('\n');
        }
    }

    let start_pos_phys = pos + Vec2::new(8.0 * ppp, 8.0 * ppp);
    let text_width_phys = galley.size().x * ppp;
    let end_pos = start_pos_phys + Vec2::new(text_width_phys, 0.0);

    state.push_history_snapshot();

    state.edit.shapes.push(DrawnShape {
        tool: ScreenshotTool::Text,
        start: start_pos_phys,
        end: end_pos,
        color: state.drawing.active_color,
        stroke_width: state.drawing.stroke_width,
        text: Some(Arc::<str>::from(baked_text)),
        points: None,
        cached_galley: None,
        cached_mosaic: None,
    });
}
