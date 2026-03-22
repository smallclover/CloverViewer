// src/os/device.rs
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
    pub monitors: Vec<MonitorInfo>,
    pub phys_min_x: i32,
    pub phys_min_y: i32,
    pub phys_total_w: u32,
    pub phys_total_h: u32,
}

impl DeviceInfo {
    pub fn load() -> Self {
        let xcap_monitors = Monitor::all().unwrap_or_else(|err| {
            eprintln!("[ERROR] 获取显示器信息失败: {}", err);
            vec![]
        });

        let mut monitors = Vec::new();
        let mut phys_min_x = i32::MAX;
        let mut phys_min_y = i32::MAX;
        let mut phys_max_x = i32::MIN;
        let mut phys_max_y = i32::MIN;

        if xcap_monitors.is_empty() {
            return Self::default();
        }

        // 计算边界并缓存纯数据结构 MonitorInfo
        for m in xcap_monitors {
            let x = m.x().unwrap_or(0);
            let y = m.y().unwrap_or(0);
            let w = m.width().unwrap_or(800);
            let h = m.height().unwrap_or(600);

            if x < phys_min_x { phys_min_x = x; }
            if y < phys_min_y { phys_min_y = y; }
            if x + (w as i32) > phys_max_x { phys_max_x = x + (w as i32); }
            if y + (h as i32) > phys_max_y { phys_max_y = y + (h as i32); }

            monitors.push(MonitorInfo {
                name: m.name().unwrap_or_default(),
                x,
                y,
                width: w,
                height: h,
                scale_factor: m.scale_factor().unwrap_or(1.0),
            });
        }

        let phys_total_w = (phys_max_x - phys_min_x).max(0) as u32;
        let phys_total_h = (phys_max_y - phys_min_y).max(0) as u32;

        Self {
            monitors,
            phys_min_x,
            phys_min_y,
            phys_total_w,
            phys_total_h,
        }
    }

    /// 取得主屏幕的缩放率
    pub fn primary_scale(&self) -> f32 {
        for m in &self.monitors {
            // 在 Windows 系统的虚拟坐标系中，主显示器的物理坐标必定是 (0, 0)
            if m.x == 0 && m.y == 0 {
                return m.scale_factor;
            }
        }
        // 兜底方案
        self.monitors.first().map(|m| m.scale_factor).unwrap_or(1.0)
    }

    /// 1. 计算并返回大画布的全局逻辑坐标和尺寸 (供 ViewportBuilder 启动窗口使用)
    pub fn global_logical_rect(&self) -> (Pos2, Vec2) {
        let scale = self.primary_scale();
        let logic_x = self.phys_min_x as f32 / scale;
        let logic_y = self.phys_min_y as f32 / scale;
        let logic_w = self.phys_total_w as f32 / scale;
        let logic_h = self.phys_total_h as f32 / scale;
        (Pos2::new(logic_x, logic_y), Vec2::new(logic_w, logic_h))
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