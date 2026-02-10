use std::time::Instant;
use xcap::Monitor;
use image::{RgbaImage, GenericImage};
use egui::Rect;

pub struct MonitorInfo {
    pub id: usize,
    pub rect: Rect,
    pub image: RgbaImage,
}

pub fn capture_all_monitors() -> Option<Vec<MonitorInfo>> {
    let start = Instant::now();
    let monitors = Monitor::all().ok()?;

    if monitors.is_empty() {
        return None;
    }

    let mut result = Vec::new();

    for (i, monitor) in monitors.into_iter().enumerate() {
        let x = monitor.x().unwrap();
        let y = monitor.y().unwrap();
        let w = monitor.width().unwrap();
        let h = monitor.height().unwrap();

        if let Ok(image) = monitor.capture_image() {
            result.push(MonitorInfo {
                id: i,
                rect: Rect::from_min_size(
                    egui::pos2(x as f32, y as f32),
                    egui::vec2(w as f32, h as f32),
                ),
                image,
            });
        }
    }

    println!("Screenshots captured in {:?}", start.elapsed());

    Some(result)
}

pub fn capture_screen_area(x: i32, y: i32, width: u32, height: u32) -> Option<RgbaImage> {
    let monitors = Monitor::all().ok()?;
    if monitors.is_empty() {
        return None;
    }

    // 找到包含该区域的显示器，或者简单地截取所有显示器然后裁剪
    // xcap 目前主要支持按显示器截图。
    // 我们可以先截取包含该区域的显示器，然后在内存中裁剪。

    // 简单实现：遍历所有显示器，找到包含 (x, y) 的那个
    for monitor in monitors {
        let m_x = monitor.x().unwrap();
        let m_y = monitor.y().unwrap();
        let m_w = monitor.width().unwrap();
        let m_h = monitor.height().unwrap();

        // 检查起始点是否在显示器内
        if x >= m_x && x < m_x + m_w as i32 && y >= m_y && y < m_y + m_h as i32 {
            // 截取该显示器
            if let Ok(mut image) = monitor.capture_image() {
                // 计算相对于显示器的坐标
                let rel_x = (x - m_x) as u32;
                let rel_y = (y - m_y) as u32;

                // 确保裁剪区域不超出图像边界
                let crop_w = width.min(m_w - rel_x);
                let crop_h = height.min(m_h - rel_y);

                use image::GenericImageView;
                let sub_image = image.view(rel_x, rel_y, crop_w, crop_h).to_image();
                return Some(sub_image);
            }
        }
    }

    None
}
