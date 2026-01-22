use std::{
    collections::HashSet,
    path::PathBuf
};
use egui::{
    Color32, CornerRadius, Rect,
    TextureHandle, Vec2, Context,
    Area,Order,Frame,Stroke,StrokeKind,
    Ui,Align2,FontId,UiBuilder,Spinner,
    Mesh,Sense
};
use lru::LruCache;

pub fn draw_preview_bar(
    ctx: &Context,
    previews: &[(usize, PathBuf)],
    thumb_cache: &mut LruCache<PathBuf, TextureHandle>,// 使用缩略图缓存
    failed_thumbs: &HashSet<PathBuf>, // 传入失败集合
    current_idx: usize,
) -> Option<usize> {
    let mut clicked_idx = None;

    let screen_rect = ctx.content_rect();
    let bar_size = Vec2::new(450.0, 90.0);
    let pos = egui::pos2(
        screen_rect.center().x - bar_size.x / 2.0,
        screen_rect.bottom() - bar_size.y - 20.0,
    );

    Area::new(egui::Id::new("preview_bar"))
        .fixed_pos(pos)
        .order(Order::Foreground)
        .show(ctx, |ui| {
            Frame::NONE
                .fill(Color32::from_rgba_unmultiplied(30, 30, 30, 200))
                .corner_radius(CornerRadius::same(12))
                .inner_margin(8.0)
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        // 加载逻辑
                        for (idx, path) in previews {
                            let size = Vec2::new(80.0, 60.0);
                            let (rect, response) = ui.allocate_exact_size(size,Sense::click());
                            let is_current = *idx == current_idx;

                            // 只要点击了这个区域（不管里面在转圈还是有图），就触发跳转
                            if response.clicked() {
                                clicked_idx = Some(*idx);
                            }

                            // --- 1. 绘制底层内容 ---
                            if let Some(tex) = thumb_cache.get(path) {
                                paint_thumbnail_texture(ui, rect, tex);
                            } else if failed_thumbs.contains(path) {
                                paint_error_state(ui, rect);
                            } else {
                                paint_loading_state(ui, rect);
                            }

                            // --- 2. 统一绘制顶层装饰（选中框和悬停效果） ---
                            if is_current {
                                ui.painter().rect_stroke(
                                    rect,
                                    CornerRadius::same(4),
                                    Stroke::new(2.5, Color32::from_rgb(200, 150, 50)),
                                    StrokeKind::Outside,
                                );
                            } else if response.hovered() {
                                ui.painter().rect_stroke(
                                    rect,
                                    CornerRadius::same(4),
                                    Stroke::new(2.0, Color32::WHITE),
                                    StrokeKind::Inside,
                                );
                            }
                        }

                    });
                });
        });
    clicked_idx
}


/// 仅负责绘制纹理网格
fn paint_thumbnail_texture(ui: &mut Ui, rect: Rect, tex: &TextureHandle) {
    let mut mesh = Mesh::with_texture(tex.id());
    mesh.add_rect_with_uv(
        rect,
        Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
        Color32::WHITE,
    );
    ui.painter().add(mesh);
}

/// 仅负责绘制错误占位
fn paint_error_state(ui: &mut egui::Ui, rect: Rect) {
    ui.painter().rect_filled(rect, CornerRadius::same(4), Color32::from_rgb(60, 20, 20));
    ui.painter().text(
        rect.center(),
        Align2::CENTER_CENTER,
        "🚫",
        FontId::proportional(18.0),
        Color32::WHITE,
    );
}

/// 仅负责绘制加载占位
fn paint_loading_state(ui: &mut Ui, rect: Rect) {
    ui.painter().rect_filled(rect, CornerRadius::same(4), Color32::from_gray(40));
    ui.scope_builder(UiBuilder::new().max_rect(rect), |ui| {
        ui.centered_and_justified(|ui| {
            ui.add(Spinner::new());
        });
    });
}
