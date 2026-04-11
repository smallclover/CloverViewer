use crate::feature::screenshot::capture::CapturedScreen;
use eframe::egui::{Pos2, Rect, Vec2};
use xcap::Monitor;

#[derive(Clone, Debug)]
pub struct MonitorInfo {
    pub name: String,
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
    pub scale_factor: f32,
}

#[derive(Clone, Debug, Default)]
pub struct DeviceInfo {
    pub phys_min_x: i32,
    pub phys_min_y: i32,
}

impl DeviceInfo {
    pub fn load() -> Self {
        let xcap_monitors = Monitor::all().unwrap_or_else(|err| {
            tracing::error!("获取显示器信息失败: {}", err);
            vec![]
        });

        let mut phys_min_x = i32::MAX;
        let mut phys_min_y = i32::MAX;

        if xcap_monitors.is_empty() {
            return Self::default();
        }

        // 计算所有显示器的最小物理起点
        for m in xcap_monitors {
            let x = m.x().unwrap_or(0);
            let y = m.y().unwrap_or(0);

            if x < phys_min_x {
                phys_min_x = x;
            }
            if y < phys_min_y {
                phys_min_y = y;
            }
        }

        Self {
            phys_min_x,
            phys_min_y,
        }
    }

    /// 2. 将某个物理屏幕的绝对坐标，转换为大画布内的相对逻辑坐标 (供贴图和遮罩使用)
    pub fn screen_logical_rect(&self, screen: &MonitorInfo, scale: f32) -> Rect {
        let phys_rel_x = (screen.x - self.phys_min_x) as f32;
        let phys_rel_y = (screen.y - self.phys_min_y) as f32;

        let logic_x = phys_rel_x / scale;
        let logic_y = phys_rel_y / scale;
        let logic_w = screen.width as f32 / scale;
        let logic_h = screen.height as f32 / scale;

        Rect::from_min_size(Pos2::new(logic_x, logic_y), Vec2::new(logic_w, logic_h))
    }
}

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
        if rect.contains(pos) { Some(rect) } else { None }
    })
}
