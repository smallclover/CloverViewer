use eframe::egui::{Color32, ColorImage, Context, Painter, Pos2, Rect, Vec2};
use std::collections::HashSet;

use crate::feature::screenshot::capture::CapturedScreen;
use crate::feature::screenshot::state::MosaicCache;

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

/// 生成马赛克纹理缓存
/// 返回纹理和对应的物理坐标范围
pub fn generate_mosaic_texture(
    ctx: &Context,
    points: &[Pos2],
    mosaic_width: f32,
    captures: &[CapturedScreen],
) -> Option<MosaicCache> {
    if points.is_empty() {
        return None;
    }

    let block_size_phys = 15.0_f32;
    let mut grid_cells: HashSet<(i32, i32)> = HashSet::new();
    let radius_phys = mosaic_width / 2.0;

    // 计算所有触及的网格单元
    for &p_phys in points {
        let min_x = ((p_phys.x - radius_phys) / block_size_phys).floor() as i32;
        let max_x = ((p_phys.x + radius_phys) / block_size_phys).ceil() as i32;
        let min_y = ((p_phys.y - radius_phys) / block_size_phys).floor() as i32;
        let max_y = ((p_phys.y + radius_phys) / block_size_phys).ceil() as i32;

        for cy in min_y..=max_y {
            for cx in min_x..=max_x {
                let cell_center_x = (cx as f32 + 0.5) * block_size_phys;
                let cell_center_y = (cy as f32 + 0.5) * block_size_phys;

                if p_phys.distance(Pos2::new(cell_center_x, cell_center_y))
                    <= radius_phys + (block_size_phys * 0.707)
                {
                    grid_cells.insert((cx, cy));
                }
            }
        }
    }

    if grid_cells.is_empty() {
        return None;
    }

    // 计算边界框
    let min_cx = grid_cells.iter().map(|(cx, _)| *cx).min().unwrap();
    let max_cx = grid_cells.iter().map(|(cx, _)| *cx).max().unwrap();
    let min_cy = grid_cells.iter().map(|(_, cy)| *cy).min().unwrap();
    let max_cy = grid_cells.iter().map(|(_, cy)| *cy).max().unwrap();

    let min_x_phys = min_cx as f32 * block_size_phys;
    let min_y_phys = min_cy as f32 * block_size_phys;
    let width_phys = (max_cx - min_cx + 1) as f32 * block_size_phys;
    let height_phys = (max_cy - min_cy + 1) as f32 * block_size_phys;

    // 创建图像（使用像素尺寸，1:1 映射物理像素）
    let img_width = width_phys.ceil() as usize;
    let img_height = height_phys.ceil() as usize;

    if img_width == 0 || img_height == 0 {
        return None;
    }

    // 填充像素
    let mut pixels: Vec<u8> = vec![0; img_width * img_height * 4];

    for (cx, cy) in grid_cells {
        let phys_x = cx as f32 * block_size_phys;
        let phys_y = cy as f32 * block_size_phys;

        // 计算相对于边界框的偏移
        let rel_x = phys_x - min_x_phys;
        let rel_y = phys_y - min_y_phys;

        // 采样颜色
        let cell_center_phys = Pos2::new(phys_x + block_size_phys * 0.5, phys_y + block_size_phys * 0.5);
        let mut color = Color32::TRANSPARENT;

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

        if color == Color32::TRANSPARENT {
            continue;
        }

        // 填充该网格单元对应的像素块
        let start_x = rel_x as usize;
        let start_y = rel_y as usize;
        let end_x = ((rel_x + block_size_phys) as usize).min(img_width);
        let end_y = ((rel_y + block_size_phys) as usize).min(img_height);

        for y in start_y..end_y {
            for x in start_x..end_x {
                let idx = (y * img_width + x) * 4;
                if idx + 3 < pixels.len() {
                    pixels[idx] = color.r();
                    pixels[idx + 1] = color.g();
                    pixels[idx + 2] = color.b();
                    pixels[idx + 3] = 255; // Alpha
                }
            }
        }
    }

    let color_image = ColorImage::from_rgba_unmultiplied([img_width, img_height], &pixels);
    let texture = ctx.load_texture(
        format!("mosaic_{}_{}", min_x_phys, min_y_phys),
        color_image,
        Default::default(),
    );

    let phys_rect = Rect::from_min_size(
        Pos2::new(min_x_phys, min_y_phys),
        Vec2::new(width_phys, height_phys),
    );

    Some(MosaicCache {
        texture,
        phys_rect,
        ppp: 1.0, // 物理像素 1:1
    })
}
