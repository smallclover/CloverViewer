use eframe::egui::{Color32, ColorImage, Pos2, Rect, TextureHandle};
use image::RgbaImage;
use std::collections::HashMap;
use std::sync::Arc;

use crate::feature::screenshot::color_picker::ColorPicker;
use crate::model::device::MonitorInfo;

const DEFAULT_ACTIVE_COLOR: Color32 = Color32::from_rgb(204, 0, 0);
const DEFAULT_STROKE_WIDTH: f32 = 2.0;
const DEFAULT_MOSAIC_WIDTH: f32 = 16.0;
const MAX_HISTORY_ENTRIES: usize = 50;

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
    pub text: Option<Arc<str>>,
    pub points: Option<Arc<Vec<Pos2>>>,
    /// 运行时缓存：文本的 egui Galley，避免每帧重排版
    pub cached_galley: Option<Arc<egui::Galley>>,
    /// 运行时缓存：马赛克的纹理，避免每帧采样原图
    pub cached_mosaic: Option<Arc<MosaicCache>>,
}

impl DrawnShape {
    fn clone_for_history(&self) -> Self {
        Self {
            tool: self.tool,
            start: self.start,
            end: self.end,
            color: self.color,
            stroke_width: self.stroke_width,
            text: self.text.clone(),
            points: self.points.clone(),
            cached_galley: None,
            cached_mosaic: None,
        }
    }
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

#[derive(Default)]
pub struct ScreenshotEditState {
    pub shapes: Vec<DrawnShape>,
    // 撤销/重做功能：历史栈
    pub history: Vec<HistoryEntry>,
    pub redo: Vec<HistoryEntry>,
}

#[derive(Default)]
pub struct ScreenshotCaptureState {
    pub captures: Vec<CapturedScreen>,
    pub window_rects: Vec<Rect>,
    pub is_capturing: bool,
    pub capture_receiver: Option<std::sync::mpsc::Receiver<(Vec<CapturedScreen>, Vec<Rect>)>>,
    pub texture_pool: HashMap<String, TextureHandle>,
}

#[derive(Default)]
pub struct ScreenshotSelectionState {
    pub selection: Option<Rect>,
    pub drag_start: Option<Pos2>,
    pub toolbar_pos: Option<Pos2>,
    pub hovered_window: Option<Rect>,
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

#[derive(Default)]
pub struct ScreenshotInputState {
    pub current_shape_start: Option<Pos2>,
    pub current_shape_end: Option<Pos2>,
    // 记录文本输入状态：Option<(文本所在的物理坐标, 文本内容)>
    pub active_text_input: Option<(Pos2, String)>,
    // 当前正在绘制的画笔轨迹
    pub current_pen_points: Vec<Pos2>,
    pub selection_change_origin: Option<SelectionChangeOrigin>,
}

#[derive(Clone)]
pub enum HistoryEntry {
    InsertShape {
        index: usize,
        shape: DrawnShape,
    },
    RemoveShape {
        index: usize,
    },
    ReplaceShape {
        index: usize,
        shape: DrawnShape,
    },
    RestoreSelectionAndShapes {
        selection: Option<Rect>,
        shapes: Vec<DrawnShape>,
    },
}

#[derive(Clone, Copy)]
pub struct SelectionChangeOrigin {
    pub previous_selection: Option<Rect>,
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

    pub fn record_shape_added(&mut self, index: usize) {
        self.push_undo_entry(HistoryEntry::RemoveShape { index });
    }

    pub fn record_shape_before_edit(&mut self, index: usize) {
        let Some(shape) = self.edit.shapes.get(index) else {
            return;
        };

        self.push_undo_entry(HistoryEntry::ReplaceShape {
            index,
            shape: shape.clone_for_history(),
        });
    }

    pub fn record_selection_change(&mut self, previous_selection: Option<Rect>) {
        let shapes = self
            .edit
            .shapes
            .iter()
            .map(DrawnShape::clone_for_history)
            .collect();

        self.push_undo_entry(HistoryEntry::RestoreSelectionAndShapes {
            selection: previous_selection,
            shapes,
        });
    }

    pub fn set_selection(&mut self, selection: Option<Rect>) {
        self.select.selection = selection;
        self.sync_toolbar_to_selection();
    }

    pub fn update_selection_only(&mut self, selection: Option<Rect>) {
        self.select.selection = selection;
    }

    pub fn sync_toolbar_to_selection(&mut self) {
        self.select.toolbar_pos = self
            .select
            .selection
            .map(|selection| selection.right_bottom());
    }

    pub fn has_positive_selection(&self) -> bool {
        self.select
            .selection
            .map(|rect| rect.is_positive())
            .unwrap_or(false)
    }

    pub fn clear_toolbar(&mut self) {
        self.select.toolbar_pos = None;
    }

    pub fn undo_last(&mut self) {
        let Some(entry) = self.edit.history.pop() else {
            return;
        };

        let Some(inverse) = self.apply_history_entry(entry) else {
            return;
        };

        Self::push_bounded_entry(&mut self.edit.redo, inverse);
    }

    pub fn redo_last(&mut self) {
        let Some(entry) = self.edit.redo.pop() else {
            return;
        };

        let Some(inverse) = self.apply_history_entry(entry) else {
            return;
        };

        Self::push_bounded_entry(&mut self.edit.history, inverse);
    }

    fn apply_history_entry(&mut self, entry: HistoryEntry) -> Option<HistoryEntry> {
        match entry {
            HistoryEntry::InsertShape { index, shape } => {
                if index <= self.edit.shapes.len() {
                    self.edit.shapes.insert(index, shape);
                    Some(HistoryEntry::RemoveShape { index })
                } else {
                    tracing::warn!(
                        "History insert-shape entry was out of bounds: index={}, len={}",
                        index,
                        self.edit.shapes.len()
                    );
                    None
                }
            }
            HistoryEntry::RemoveShape { index } => {
                if index < self.edit.shapes.len() {
                    let removed = self.edit.shapes.remove(index);
                    Some(HistoryEntry::InsertShape {
                        index,
                        shape: removed.clone_for_history(),
                    })
                } else {
                    tracing::warn!(
                        "History remove-shape entry was out of bounds: index={}, len={}",
                        index,
                        self.edit.shapes.len()
                    );
                    None
                }
            }
            HistoryEntry::ReplaceShape { index, shape } => {
                if let Some(target) = self.edit.shapes.get_mut(index) {
                    let inverse = HistoryEntry::ReplaceShape {
                        index,
                        shape: target.clone_for_history(),
                    };
                    *target = shape;
                    Some(inverse)
                } else {
                    tracing::warn!(
                        "History replace-shape entry was out of bounds: index={}, len={}",
                        index,
                        self.edit.shapes.len()
                    );
                    None
                }
            }
            HistoryEntry::RestoreSelectionAndShapes { selection, shapes } => {
                let previous_selection = self.select.selection;
                let previous_shapes = self
                    .edit
                    .shapes
                    .iter()
                    .map(DrawnShape::clone_for_history)
                    .collect();

                self.edit.shapes = shapes;
                self.set_selection(selection);

                Some(HistoryEntry::RestoreSelectionAndShapes {
                    selection: previous_selection,
                    shapes: previous_shapes,
                })
            }
        }
    }

    fn push_undo_entry(&mut self, entry: HistoryEntry) {
        self.edit.redo.clear();
        Self::push_bounded_entry(&mut self.edit.history, entry);
    }

    fn push_bounded_entry(stack: &mut Vec<HistoryEntry>, entry: HistoryEntry) {
        stack.push(entry);
        if stack.len() > MAX_HISTORY_ENTRIES {
            stack.remove(0);
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

#[cfg(test)]
mod tests {
    use super::{DrawnShape, HistoryEntry, MAX_HISTORY_ENTRIES, ScreenshotState, ScreenshotTool};
    use eframe::egui::{Color32, Pos2, Rect};
    use std::sync::Arc;

    fn make_shape(start_x: f32) -> DrawnShape {
        DrawnShape {
            tool: ScreenshotTool::Rect,
            start: Pos2::new(start_x, 0.0),
            end: Pos2::new(start_x + 10.0, 10.0),
            color: Color32::WHITE,
            stroke_width: 2.0,
            text: None,
            points: None,
            cached_galley: None,
            cached_mosaic: None,
        }
    }

    #[test]
    fn selection_history_shares_shape_payloads() {
        let shared_text: Arc<str> = Arc::from("hello");
        let shared_points = Arc::new(vec![Pos2::new(1.0, 2.0), Pos2::new(3.0, 4.0)]);
        let mut state = ScreenshotState::default();
        state.edit.shapes.push(DrawnShape {
            tool: ScreenshotTool::Pen,
            start: Pos2::new(0.0, 0.0),
            end: Pos2::new(10.0, 10.0),
            color: Color32::WHITE,
            stroke_width: 2.0,
            text: Some(shared_text.clone()),
            points: Some(shared_points.clone()),
            cached_galley: None,
            cached_mosaic: None,
        });

        state.record_selection_change(None);

        let HistoryEntry::RestoreSelectionAndShapes { shapes, .. } = &state.edit.history[0] else {
            panic!("expected selection restore history entry");
        };

        let snapshot = &shapes[0];
        assert!(Arc::ptr_eq(snapshot.text.as_ref().expect("text missing"), &shared_text));
        assert!(Arc::ptr_eq(snapshot.points.as_ref().expect("points missing"), &shared_points));
        assert!(snapshot.cached_galley.is_none());
        assert!(snapshot.cached_mosaic.is_none());
    }

    #[test]
    fn history_is_bounded() {
        let mut state = ScreenshotState::default();

        for index in 0..=MAX_HISTORY_ENTRIES {
            state.record_shape_added(index);
        }

        assert_eq!(state.edit.history.len(), MAX_HISTORY_ENTRIES);
    }

    #[test]
    fn undo_redo_shape_addition_roundtrip() {
        let mut state = ScreenshotState::default();
        state.edit.shapes.push(make_shape(10.0));
        state.record_shape_added(0);

        state.undo_last();
        assert!(state.edit.shapes.is_empty());
        assert!(state.edit.history.is_empty());
        assert_eq!(state.edit.redo.len(), 1);

        state.redo_last();
        assert_eq!(state.edit.shapes.len(), 1);
        assert_eq!(state.edit.shapes[0].start.x, 10.0);
        assert_eq!(state.edit.history.len(), 1);
        assert!(state.edit.redo.is_empty());
    }

    #[test]
    fn undo_redo_shape_edit_roundtrip() {
        let mut state = ScreenshotState::default();
        state.edit.shapes.push(make_shape(10.0));
        state.record_shape_before_edit(0);
        state.edit.shapes[0].start.x = 40.0;
        state.edit.shapes[0].end.x = 60.0;

        state.undo_last();
        assert_eq!(state.edit.shapes[0].start.x, 10.0);
        assert_eq!(state.edit.shapes[0].end.x, 20.0);

        state.redo_last();
        assert_eq!(state.edit.shapes[0].start.x, 40.0);
        assert_eq!(state.edit.shapes[0].end.x, 60.0);
    }

    #[test]
    fn undo_redo_selection_change_roundtrip() {
        let mut state = ScreenshotState::default();
        let original_selection = Some(Rect::from_min_max(
            Pos2::new(0.0, 0.0),
            Pos2::new(100.0, 100.0),
        ));
        let changed_selection = Some(Rect::from_min_max(
            Pos2::new(10.0, 10.0),
            Pos2::new(80.0, 80.0),
        ));

        state.set_selection(original_selection);
        state.edit.shapes.push(make_shape(5.0));
        state.record_selection_change(original_selection);

        state.set_selection(changed_selection);
        state.edit.shapes.clear();

        state.undo_last();
        assert_eq!(state.select.selection, original_selection);
        assert_eq!(state.edit.shapes.len(), 1);
        assert_eq!(state.edit.shapes[0].start.x, 5.0);

        state.redo_last();
        assert_eq!(state.select.selection, changed_selection);
        assert!(state.edit.shapes.is_empty());
    }

    #[test]
    fn redo_is_cleared_after_new_edit() {
        let mut state = ScreenshotState::default();
        state.edit.shapes.push(make_shape(10.0));
        state.record_shape_added(0);
        state.undo_last();

        state.edit.shapes.push(make_shape(20.0));
        state.record_shape_added(0);

        assert!(state.edit.redo.is_empty());

        state.redo_last();
        assert_eq!(state.edit.shapes.len(), 1);
        assert_eq!(state.edit.shapes[0].start.x, 20.0);
    }
}
