use std::path::PathBuf;
use egui::{Color32, CornerRadius, Rect, TextureHandle, Vec2};
use lru::LruCache;

pub fn draw_preview_bar(
    ctx: &egui::Context,
    previews: &[(usize, PathBuf)],
    thumb_cache: &mut LruCache<PathBuf, TextureHandle>,// 使用缩略图缓存
    current_idx: usize,
) -> Option<usize> {
    let mut clicked_idx = None;

    let screen_rect = ctx.content_rect();
    let bar_size = Vec2::new(450.0, 90.0);
    let pos = egui::pos2(
        screen_rect.center().x - bar_size.x / 2.0,
        screen_rect.bottom() - bar_size.y - 20.0,
    );

    egui::Area::new(egui::Id::new("preview_bar"))
        .fixed_pos(pos)
        .order(egui::Order::Foreground)
        .show(ctx, |ui| {
            egui::Frame::NONE
                .fill(Color32::from_rgba_unmultiplied(30, 30, 30, 200))
                .corner_radius(CornerRadius::same(12))
                .inner_margin(8.0)
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        // 加载逻辑
                        for (idx, path) in previews {
                            let size = Vec2::new(80.0, 60.0);
                            let (rect, response) = ui.allocate_exact_size(size,egui::Sense::click());

                            // 只要点击了这个区域（不管里面在转圈还是有图），就触发跳转
                            if response.clicked() {
                                clicked_idx = Some(*idx);
                            }

                            // 绘制外观
                            if ui.is_rect_visible(rect) {
                                if let Some(tex) = thumb_cache.get(path) {
                                    // 已加载：渲染图片内容
                                    render_thumbnail(ui, rect, tex, *idx == current_idx, response.hovered());
                                } else {
                                    // 1. 绘制加载中背景
                                    ui.painter().rect_filled(rect, CornerRadius::same(4), Color32::from_gray(40));

                                    // 2. 使用 scope_builder 在指定的 rect 开启新作用域
                                    // 这里的 UiBuilder::new().with_rect(rect) 确保了内部 UI 的坐标系就是这个格子
                                    ui.scope_builder(egui::UiBuilder::new().max_rect(rect), |ui| {
                                        ui.centered_and_justified(|ui| {
                                            ui.add(egui::Spinner::new());
                                        });
                                    });
                                }
                            }

                            // // 优先从缩略图缓存里拿
                            // if let Some(tex) = thumb_cache.get(path) {
                            //     let response = render_thumbnail(ui, tex, *idx == current_idx);
                            //     if response.clicked() { clicked_idx = Some(*idx); }
                            // } else {
                            //     // 如果缩略图还没加载好，显示菊花
                            //     ui.allocate_ui(Vec2::new(80.0, 60.0), |ui| {
                            //         ui.centered_and_justified(|ui| ui.spinner());
                            //     });
                            // }
                        }

                    });
                });
        });
    clicked_idx
}


// 渲染UI
fn render_thumbnail(ui: &mut egui::Ui, rect: Rect ,tex: &TextureHandle, is_current: bool, is_hovered: bool) {
        let mut mesh = egui::Mesh::with_texture(tex.id());
        // 简单的填充 UV
        mesh.add_rect_with_uv(
            rect,
            Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
            Color32::WHITE,
        );
        ui.painter().add(mesh);
        // 如果是当前图片，画一个高亮边框
        if is_current {
            ui.painter().rect_stroke(
                rect,
                CornerRadius::same(4),
                egui::Stroke::new(2.5, egui::Color32::from_rgb(200, 150, 50)),
                egui::StrokeKind::Outside // 向外描边，不遮挡缩略图
            );
        } else if is_hovered {
            ui.painter().rect_stroke(
                rect,
                CornerRadius::same(2),
                egui::Stroke::new(2.0, egui::Color32::WHITE),
                egui::StrokeKind::Inside // 向内描边，更适合悬停效果
            );
        }
}