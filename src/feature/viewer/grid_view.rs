use eframe::egui;
use egui::{Context, ScrollArea, Ui, Color32, Frame, Stroke, Sense, Align2, FontId, Vec2, Rect, Pos2};
use crate::{
    core::viewer_state::{ViewerState, ViewMode},
    i18n::lang::get_i18n_text
};

pub fn draw_grid_view(
    ctx: &Context,
    ui: &mut Ui,
    viewer: &mut ViewerState,
) {
    let text = get_i18n_text(ctx);

    if viewer.list.is_empty() {
        ui.centered_and_justified(|ui| {
            ui.label(text.viewer_no_images);
        });
        return;
    }

    // 1. 尺寸定义
    let item_size = Vec2::new(150.0, 150.0);
    let frame_margin = 4.0;
    let border_width = 2.0;
    let spacing = 16.0; // 舒适的间距

    // 一个单元格的物理总宽度 = 内部图片(150) + 两边内边距(8) + 两边边框(4)
    let cell_total_width = item_size.x + (frame_margin * 2.0) + (border_width * 2.0);

    // 2. 核心居中算法
    let available_width = ui.available_width();
    // 计算当前宽度最多能放下几列
    let mut columns = ((available_width + spacing) / (cell_total_width + spacing)).floor() as usize;
    columns = columns.max(1);

    // 计算这几列加起来的真实总宽度
    let content_width = (columns as f32 * cell_total_width) + ((columns as f32 - 1.0) * spacing);
    // 剩下的留白除以 2，就是左侧需要垫入的精准空白
    let left_padding = ((available_width - content_width) / 2.0).max(0.0);

    let mut clicked_index = None;
    let mut double_clicked_index = None;

    let list = &viewer.list;
    let current_index = viewer.index;
    let thumb_cache = &mut viewer.thumb_cache;
    let loading_thumbs = &mut viewer.loading_thumbs;
    let failed_thumbs = &viewer.failed_thumbs;
    let loader = &mut viewer.loader;

    let visible_rect = ui.clip_rect();
    let preload_rect = visible_rect.expand(500.0);

    // 3. 开始渲染
    ScrollArea::vertical()
        .auto_shrink([false; 2]) // 强制占满可用高度
        .show(ui, |ui| {
            ui.add_space(spacing); // 顶部的呼吸空间

            // 将一维列表按列数切分成一块块的 row
            for (row_idx, row_items) in list.chunks(columns).enumerate() {
                ui.horizontal(|ui| {
                    // 填入计算好的左侧空白，瞬间居中！
                    ui.add_space(left_padding);
                    // 覆盖默认的水平间距，使用我们自定义的间距
                    ui.spacing_mut().item_spacing.x = spacing;

                    for (col_idx, path) in row_items.iter().enumerate() {
                        let global_index = row_idx * columns + col_idx;
                        let is_selected = global_index == current_index;

                        let (stroke_color, bg_color) = if is_selected {
                            (Color32::from_rgb(0, 120, 215), Color32::from_gray(45)) // 稍微提亮选中颜色，更现代
                        } else {
                            (Color32::from_gray(60), Color32::from_gray(30))
                        };

                        let frame = Frame::default()
                            .fill(bg_color)
                            .stroke(Stroke::new(border_width, stroke_color))
                            .inner_margin(frame_margin)
                            .corner_radius(6.0); // 加一点点圆角更精致

                        // 绘制每个单元格
                        frame.show(ui, |ui| {
                            let (rect, response) = ui.allocate_exact_size(item_size, Sense::click());

                            // 预加载逻辑
                            if preload_rect.intersects(rect) {
                                if !thumb_cache.contains(path) && !failed_thumbs.contains(path) && !loading_thumbs.contains(path) {
                                    loading_thumbs.insert(path.clone());
                                    loader.load_async(ctx.clone(), path.clone(), false, Some((200, 200)));
                                }
                            }

                            // 渲染图像或加载文本
                            if ui.is_rect_visible(rect) {
                                if let Some(t) = thumb_cache.get(path) {
                                    let tex_size = t.size_vec2();
                                    let scale = (rect.width() / tex_size.x).min(rect.height() / tex_size.y);
                                    let target_size = tex_size * scale;
                                    let target_rect = Rect::from_center_size(rect.center(), target_size);

                                    ui.painter().image(
                                        t.id(),
                                        target_rect,
                                        Rect::from_min_max(Pos2::new(0.0, 0.0), Pos2::new(1.0, 1.0)),
                                        Color32::WHITE
                                    );
                                } else {
                                    ui.painter().text(
                                        rect.center(),
                                        Align2::CENTER_CENTER,
                                        text.grid_loading,
                                        FontId::proportional(14.0),
                                        Color32::GRAY,
                                    );
                                }
                            }

                            if response.double_clicked() {
                                double_clicked_index = Some(global_index);
                            } else if response.clicked() {
                                clicked_index = Some(global_index);
                            }
                        });
                    }
                });

                ui.add_space(spacing); // 行与行之间的垂直间距
            }
        });

    // 处理交互状态改变
    if let Some(index) = double_clicked_index {
        viewer.set_index(index);
        viewer.view_mode = ViewMode::Single;
        viewer.load_current(ctx.clone());
    } else if let Some(index) = clicked_index {
        viewer.set_index(index);
    }
}