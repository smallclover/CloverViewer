use eframe::emath::{Pos2, Rect};
use crate::feature::screenshot::capture::CapturedScreen;
use crate::model::device::MonitorInfo;

/// 获取屏幕的物理边界矩形
#[inline]
pub fn get_screen_phys_rect(info: &MonitorInfo) -> Rect {
    Rect::from_min_size(
        Pos2::new(info.x as f32, info.y as f32),
        egui::vec2(info.width as f32, info.height as f32),
    )
}

/// 根据物理坐标，查找包含该坐标的屏幕物理矩形
pub fn find_target_screen_rect(captures: &[CapturedScreen], pos: Pos2) -> Option<Rect> {
    captures.iter().find_map(|cap| {
        let rect = get_screen_phys_rect(&cap.screen_info);
        if rect.contains(pos) {
            Some(rect)
        } else {
            None
        }
    })
}