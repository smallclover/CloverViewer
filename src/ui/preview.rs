use std::{
    collections::HashSet,
    path::PathBuf
};
use egui::{
    Color32, CornerRadius, Rect,
    TextureHandle, Vec2, Context,
    Area,Frame,Stroke,StrokeKind,
    Ui,Align2,FontId,UiBuilder,Spinner,
    Mesh,Sense
};
use lru::LruCache;

enum ThumbnailState<'a> {
    Loaded(&'a TextureHandle),//已经加载
    Failed,//加载失败
    Loading,//加载中
}


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
        // .order(Order::Foreground)
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
                            let (rect, response) = ui.allocate_exact_size(size, Sense::click());

                            // A. 逻辑层：处理点击
                            if response.clicked() { clicked_idx = Some(*idx); }

                            // B. 状态判定层：将复杂的数据判断转化为简单的状态枚举
                            let state = if let Some(tex) = thumb_cache.get(path) {
                                ThumbnailState::Loaded(tex)
                            } else if failed_thumbs.contains(path) {
                                ThumbnailState::Failed
                            } else {
                                ThumbnailState::Loading
                            };
                            // C. 表现层：调用统一渲染器
                            render_preview_item(ui, rect, state, *idx == current_idx, &response);
                        }

                    });
                });
        });
    clicked_idx
}


/// 渲染预览窗口
fn render_preview_item(
    ui: &mut Ui,
    rect: Rect,
    state: ThumbnailState,
    is_current: bool,
    response: &egui::Response
) {
    if !ui.is_rect_visible(rect) { return; }

    // 1. 绘制主体内容（根据状态）
    match state {
        ThumbnailState::Loaded(tex) => {
            paint_thumbnail_texture(ui, rect, tex);
        }
        ThumbnailState::Failed => {
            paint_error_state(ui, rect);
        }
        ThumbnailState::Loading => {
            paint_loading_state(ui, rect);
        }
    }

    // 2. 绘制 UI 装饰层（选中、悬停）
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
fn paint_error_state(ui: &mut Ui, rect: Rect) {
    ui.painter().rect_filled(rect, CornerRadius::same(4), Color32::from_rgb(60, 20, 20));
    ui.painter().text(
        rect.center(),
        Align2::CENTER_CENTER,
        "🚫",
        FontId::proportional(18.0),
        Color32::RED,
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
