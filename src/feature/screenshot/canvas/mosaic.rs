use eframe::egui::{Color32, Painter, Pos2, Rect, Vec2};
use std::collections::HashSet;

use crate::feature::screenshot::capture::CapturedScreen;

/// 实时马赛克渲染（采样原图）
pub fn draw_realtime_mosaic(
    painter: &Painter,
    points: &[Pos2],
    mosaic_width: f32,
    global_offset_phys: Pos2,
    ppp: f32,
    captures: &[CapturedScreen],
) {
    if points.is_empty() {
        return;
    }

    let block_size_phys = 15.0;
    let mut grid_cells: HashSet<(i32, i32)> = HashSet::new();
    let radius_phys = (mosaic_width * ppp) / 2.0;

    // 将画笔触及的点映射为马赛克网格区域
    for &p_phys in points {
        let min_x = ((p_phys.x - radius_phys) / block_size_phys).floor() as i32;
        let max_x = ((p_phys.x + radius_phys) / block_size_phys).ceil() as i32;
        let min_y = ((p_phys.y - radius_phys) / block_size_phys).floor() as i32;
        let max_y = ((p_phys.y + radius_phys) / block_size_phys).ceil() as i32;

        for cy in min_y..=max_y {
            for cx in min_x..=max_x {
                let cell_center_x = (cx as f32 + 0.5) * block_size_phys;
                let cell_center_y = (cy as f32 + 0.5) * block_size_phys;

                // 圆形碰撞探测
                if p_phys.distance(Pos2::new(cell_center_x, cell_center_y))
                    <= radius_phys + (block_size_phys * 0.707)
                {
                    grid_cells.insert((cx, cy));
                }
            }
        }
    }

    // 抓取原图像素并渲染方块
    for (cx, cy) in grid_cells {
        let phys_x = cx as f32 * block_size_phys;
        let phys_y = cy as f32 * block_size_phys;

        let mut color = Color32::TRANSPARENT;
        let cell_center_phys = Pos2::new(phys_x + block_size_phys * 0.5, phys_y + block_size_phys * 0.5);

        // 从原始截图中采样颜色
        for cap in captures {
            let rect = Rect::from_min_size(
                Pos2::new(cap.screen_info.x as f32, cap.screen_info.y as f32),
                Vec2::new(cap.screen_info.width as f32, cap.screen_info.height as f32),
            );
            if rect.contains(cell_center_phys) {
                let local_x = (cell_center_phys.x - rect.min.x) as u32;
                let local_y = (cell_center_phys.y - rect.min.y) as u32;
                if local_x < cap.raw_image.width() && local_y < cap.raw_image.height() {
                    let p = cap.raw_image.get_pixel(local_x, local_y);
                    color = Color32::from_rgb(p[0], p[1], p[2]);
                }
                break;
            }
        }

        if color != Color32::TRANSPARENT {
            let local_min = Pos2::ZERO + ((Pos2::new(phys_x, phys_y) - global_offset_phys) / ppp);
            let local_rect = Rect::from_min_size(local_min, Vec2::splat(block_size_phys / ppp));
            painter.rect_filled(local_rect, 0.0, color);
        }
    }
}
