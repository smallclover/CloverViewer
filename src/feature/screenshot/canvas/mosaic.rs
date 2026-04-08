use eframe::egui::{Color32, ColorImage, Context, Painter, Pos2, Rect, Vec2};
use image::RgbaImage;
use std::collections::HashSet;

use super::MOSAIC_BLOCK_SIZE;
use crate::feature::screenshot::capture::CapturedScreen;
use crate::feature::screenshot::state::MosaicCache;

fn mosaic_radius_phys(mosaic_width: f32, ppp: f32) -> f32 {
    (mosaic_width * ppp) / 2.0
}

fn collect_mosaic_grid_cells(
    points: &[Pos2],
    radius_phys: f32,
    block_size_phys: f32,
) -> HashSet<(i32, i32)> {
    let mut grid_cells = HashSet::new();

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

    grid_cells
}

fn clip_mosaic_cells(
    grid_cells: HashSet<(i32, i32)>,
    block_size_phys: f32,
    selection: Option<Rect>,
) -> Vec<(i32, i32, Rect)> {
    grid_cells
        .into_iter()
        .filter_map(|(cx, cy)| {
            let cell_rect_phys = Rect::from_min_size(
                Pos2::new(cx as f32 * block_size_phys, cy as f32 * block_size_phys),
                Vec2::splat(block_size_phys),
            );
            let clipped_rect_phys = if let Some(selection) = selection {
                cell_rect_phys.intersect(selection)
            } else {
                cell_rect_phys
            };

            if clipped_rect_phys.is_positive() {
                Some((cx, cy, clipped_rect_phys))
            } else {
                None
            }
        })
        .collect()
}

fn collect_clipped_mosaic_cells(
    points: &[Pos2],
    mosaic_width: f32,
    ppp: f32,
    block_size_phys: f32,
    selection: Option<Rect>,
) -> Vec<(i32, i32, Rect)> {
    let radius_phys = mosaic_radius_phys(mosaic_width, ppp);
    let grid_cells = collect_mosaic_grid_cells(points, radius_phys, block_size_phys);
    clip_mosaic_cells(grid_cells, block_size_phys, selection)
}

fn sample_mosaic_color(captures: &[CapturedScreen], cell_center_phys: Pos2) -> Color32 {
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
                return Color32::from_rgb(p[0], p[1], p[2]);
            }
            break;
        }
    }

    Color32::TRANSPARENT
}

/// 实时马赛克渲染（采样原图）
pub fn draw_realtime_mosaic(
    painter: &Painter,
    points: &[Pos2],
    mosaic_width: f32,
    global_offset_phys: Pos2,
    ppp: f32,
    selection: Option<Rect>,
    captures: &[CapturedScreen],
) {
    if points.is_empty() {
        return;
    }

    let block_size_phys = MOSAIC_BLOCK_SIZE;
    let clipped_cells =
        collect_clipped_mosaic_cells(points, mosaic_width, ppp, block_size_phys, selection);

    // 抓取原图像素并渲染方块
    for (cx, cy, clipped_rect_phys) in clipped_cells {
        let phys_x = cx as f32 * block_size_phys;
        let phys_y = cy as f32 * block_size_phys;

        let cell_center_phys = Pos2::new(
            phys_x + block_size_phys * 0.5,
            phys_y + block_size_phys * 0.5,
        );
        let color = sample_mosaic_color(captures, cell_center_phys);

        if color != Color32::TRANSPARENT {
            let local_min = Pos2::ZERO + ((clipped_rect_phys.min - global_offset_phys) / ppp);
            let local_rect = Rect::from_min_size(local_min, clipped_rect_phys.size() / ppp);
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
    ppp: f32,
    selection: Option<Rect>,
    captures: &[CapturedScreen],
) -> Option<MosaicCache> {
    if points.is_empty() {
        return None;
    }

    let block_size_phys = MOSAIC_BLOCK_SIZE;
    let clipped_cells =
        collect_clipped_mosaic_cells(points, mosaic_width, ppp, block_size_phys, selection);

    if clipped_cells.is_empty() {
        return None;
    }

    let min_x_phys = clipped_cells
        .iter()
        .map(|(_, _, rect)| rect.min.x)
        .fold(f32::INFINITY, f32::min);
    let min_y_phys = clipped_cells
        .iter()
        .map(|(_, _, rect)| rect.min.y)
        .fold(f32::INFINITY, f32::min);
    let max_x_phys = clipped_cells
        .iter()
        .map(|(_, _, rect)| rect.max.x)
        .fold(f32::NEG_INFINITY, f32::max);
    let max_y_phys = clipped_cells
        .iter()
        .map(|(_, _, rect)| rect.max.y)
        .fold(f32::NEG_INFINITY, f32::max);

    let width_phys = max_x_phys - min_x_phys;
    let height_phys = max_y_phys - min_y_phys;

    // 创建图像（使用像素尺寸，1:1 映射物理像素）
    let img_width = width_phys.ceil() as usize;
    let img_height = height_phys.ceil() as usize;

    if img_width == 0 || img_height == 0 {
        return None;
    }

    // 填充像素
    let mut pixels: Vec<u8> = vec![0; img_width * img_height * 4];

    for (cx, cy, clipped_rect_phys) in clipped_cells {
        let phys_x = cx as f32 * block_size_phys;
        let phys_y = cy as f32 * block_size_phys;

        // 计算相对于边界框的偏移
        let rel_x = clipped_rect_phys.min.x - min_x_phys;
        let rel_y = clipped_rect_phys.min.y - min_y_phys;

        // 采样颜色
        let cell_center_phys = Pos2::new(
            phys_x + block_size_phys * 0.5,
            phys_y + block_size_phys * 0.5,
        );
        let color = sample_mosaic_color(captures, cell_center_phys);

        if color == Color32::TRANSPARENT {
            continue;
        }

        // 填充该网格单元对应的像素块
        let start_x = rel_x.floor() as usize;
        let start_y = rel_y.floor() as usize;
        let end_x = (rel_x + clipped_rect_phys.width()).ceil() as usize;
        let end_y = (rel_y + clipped_rect_phys.height()).ceil() as usize;
        let end_x = end_x.min(img_width);
        let end_y = end_y.min(img_height);

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

    Some(MosaicCache { texture, phys_rect })
}

pub fn apply_mosaic_to_cropped_image(
    final_image: &mut RgbaImage,
    points: &[Pos2],
    mosaic_width: f32,
    selection_phys: Rect,
) {
    if points.is_empty() {
        return;
    }

    let block_size_phys = MOSAIC_BLOCK_SIZE;
    let clipped_cells = collect_clipped_mosaic_cells(
        points,
        mosaic_width,
        1.0,
        block_size_phys,
        Some(selection_phys),
    );

    if clipped_cells.is_empty() {
        return;
    }

    for (_, _, clipped_rect_phys) in clipped_cells {
        let sample_x = clipped_rect_phys.center().x - selection_phys.min.x;
        let sample_y = clipped_rect_phys.center().y - selection_phys.min.y;
        let sample_x = sample_x
            .floor()
            .clamp(0.0, (final_image.width().saturating_sub(1)) as f32)
            as u32;
        let sample_y = sample_y
            .floor()
            .clamp(0.0, (final_image.height().saturating_sub(1)) as f32)
            as u32;
        let pixel = *final_image.get_pixel(sample_x, sample_y);

        let start_x = (clipped_rect_phys.min.x - selection_phys.min.x)
            .floor()
            .max(0.0) as u32;
        let start_y = (clipped_rect_phys.min.y - selection_phys.min.y)
            .floor()
            .max(0.0) as u32;
        let end_x = (clipped_rect_phys.max.x - selection_phys.min.x)
            .ceil()
            .min(final_image.width() as f32) as u32;
        let end_y = (clipped_rect_phys.max.y - selection_phys.min.y)
            .ceil()
            .min(final_image.height() as f32) as u32;

        for y in start_y..end_y {
            for x in start_x..end_x {
                final_image.put_pixel(x, y, pixel);
            }
        }
    }
}
