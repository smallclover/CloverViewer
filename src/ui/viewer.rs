use eframe::egui;
use egui::{
    CentralPanel, Color32, Context, CursorIcon, Frame,
    Image, RichText, ScrollArea, TextureHandle, Ui, UiBuilder, Rect
};
use crate::{
    ui::{
        arrows::{draw_arrows, Nav},
        ui_mode::UiMode
    },
    i18n::{get_text, Language}
};


pub struct ViewerState<'a> {
    pub texture: Option<&'a TextureHandle>,
    pub is_loading: bool,
    pub error: Option<&'a String>,
    pub zoom: f32,
    pub has_nav: bool,
}

pub fn draw_viewer(
    ctx: &Context,
    state: ViewerState,
    ui_mode: &mut UiMode,
    lang: Language,
) -> Option<Nav> {
    let mut nav_action = None;
    let text = get_text(lang);

    // 创建深色背景
    let background_frame = Frame::NONE.fill(Color32::from_rgb(25, 25, 25));

    CentralPanel::default().frame(background_frame).show(ctx, |ui| {
        let rect = ui.available_rect_before_wrap();

        // --- 1. 渲染图片内容 ---
        if let Some(tex) = state.texture {
            // 渲染图片，不返回交互响应
            render_image_viewer(ui, tex, state.zoom, state.is_loading);

            // --- 2. 全局右键检测 ---
            // 不使用 interact 捕获，而是直接检查输入状态
            // 这样不会消费事件，也不会被 ScrollArea 阻挡
            if ui.input(|i| i.pointer.secondary_clicked()) {
                if let Some(pos) = ui.input(|i| i.pointer.hover_pos()) {
                    // 检查点击位置是否在视图区域内
                    if rect.contains(pos) {
                        // 默认允许弹出
                        let mut allow_context_menu = true;

                        // 如果有导航箭头，需要排除左右两侧的感应区域
                        if state.has_nav {
                            let hover_zone_width = 100.0; // 固定感应区域宽度，与 arrows.rs 保持一致

                            // 定义中间的安全区域
                            let center_zone = Rect::from_min_max(
                                rect.min + egui::vec2(hover_zone_width, 0.0),
                                rect.max - egui::vec2(hover_zone_width, 0.0),
                            );

                            // 如果点击不在中间区域（即在左右两侧），则禁止弹出
                            if !center_zone.contains(pos) {
                                allow_context_menu = false;
                            }
                        }

                        if allow_context_menu {
                            *ui_mode = UiMode::ContextMenu(pos);
                        }
                    }
                }
            }

        }
        // --- 3. 处理其他状态（加载、失败、空状态） ---
        else if let Some(_) = state.error {
            ui.scope_builder(UiBuilder::new().max_rect(rect),|ui| {
                ui.vertical_centered(|ui| {
                    ui.add_space(ui.available_height() * 0.4);
                    ui.label(RichText::new(text.viewer_error).color(Color32::RED).size(14.0));
                });
            });
        } else if state.is_loading {
            // 正在加载且没图没报错，留白
        } else {
            ui.centered_and_justified(|ui| ui.label(text.viewer_drag_hint));
        }

        // --- 4. 渲染导航箭头（最后渲染，确保在最上层） ---
        if state.has_nav {
            if let Some(action) = draw_arrows(ui, rect) {
                // 只有在没有其他动作时才响应箭头
                nav_action = Some(action);
            }
        }
    });

    nav_action
}

/// 渲染图片查看器
fn render_image_viewer(
    ui: &mut Ui,
    tex: &TextureHandle,
    zoom: f32,
    is_loading_high_res: bool
) {
    // 计算是否能够拖动，然后变换鼠标样式
    let size = tex.size_vec2() * zoom;
    let available_size = ui.available_size();
    let is_draggable = size.x > available_size.x || size.y > available_size.y;

    if is_draggable {
        if ui.rect_contains_pointer(ui.max_rect()) {
            ui.ctx().set_cursor_icon(CursorIcon::Move);
        }
    }
    // 平滑动画：计算加载遮罩的透明度
    let fade_alpha = ui.ctx().animate_bool_with_time(
        egui::Id::new(tex.id()).with("loading_fade"),
        is_loading_high_res,
        0.25
    );

    ScrollArea::both()
        .scroll_source(egui::scroll_area::ScrollSource::DRAG)
        .auto_shrink([false; 2])
        .show(ui, |ui| {
            let x_offset = (available_size.x - size.x).max(0.0) * 0.5;
            let y_offset = (available_size.y - size.y).max(0.0) * 0.5;

            ui.horizontal(|ui| {
                ui.add_space(x_offset);
                ui.vertical(|ui| {
                    ui.add_space(y_offset);
                    // 1. 绘制图片
                    let img_rect = ui.allocate_exact_size(size, egui::Sense::hover()).0;
                    let img_widget = Image::from_texture(tex).fit_to_exact_size(size);
                    ui.put(img_rect, img_widget);

                    // 2. 绘制加载遮罩 (当正在加载大图时)
                    if fade_alpha > 0.0 {
                        let painter = ui.painter_at(img_rect);

                        // 遮罩层：让缩略图变暗，突出正在处理感
                        painter.rect_filled(
                            img_rect,
                            0.0,
                            Color32::BLACK.gamma_multiply(fade_alpha * 0.4)
                        );

                        // 在图片中心绘制一个精致的 Spinner
                        let spinner_size = 32.0;
                        let spinner_rect = Rect::from_center_size(
                            img_rect.center(),
                            egui::vec2(spinner_size, spinner_size)
                        );

                        // 使用你现有的 Spinner 样式，但在图片中心显示
                        ui.put(spinner_rect, egui::Spinner::new().size(spinner_size).color(Color32::WHITE.gamma_multiply(fade_alpha)));
                    }
                });
            });
        });
}
