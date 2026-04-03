pub mod drag;
pub mod draw;
pub mod hit_test;
pub mod interaction;
pub mod mosaic;
pub mod render;
pub mod shape;
pub mod text_input;

use eframe::egui::{Id, Pos2, Ui};

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
            dragging_selection: ui.data(|d| d.get_temp(Id::new(Self::DRAGGING_SEL_ID))).unwrap_or(false),
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
