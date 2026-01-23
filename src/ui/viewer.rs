use eframe::egui;
use egui::{
    CentralPanel, Color32, Context, CursorIcon, Frame,
    Image, RichText, ScrollArea, TextureHandle, Ui, UiBuilder
};
use crate::ui::arrows::{draw_arrows, Nav};
use crate::ui::loading::corner_loading;
use crate::i18n::{get_text, Language};

pub enum ViewerAction {
    Prev,
    Next,
    None,
}

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
    show_about: bool,
    lang: Language,
) -> ViewerAction {
    let mut action = ViewerAction::None;
    let text = get_text(lang);

    // 创建深色背景，添加内边距 (margin)
    let background_frame = Frame::NONE
        .fill(Color32::from_rgb(25, 25, 25)) // 深炭灰色，能很好地衬托图片
        .inner_margin(0.0); // 如果希望图片贴边，设为 0

    CentralPanel::default().frame(background_frame).show(ctx, |ui| {
        ui.add_enabled_ui(!show_about, |ui| {
             let rect = ui.available_rect_before_wrap();

            // 情况 1：只要当前有纹理（无论是原图还是缩略图），就先画出来
            if let Some(tex) = state.texture {
                render_image_viewer(ui, tex, state.zoom);

                // 如果正在加载（说明现在显示的是缩略图，高清图还在路上），在右上角画个小菊花
                if state.is_loading {
                    corner_loading(ui);
                }
            }
            // 情况 2：加载失败
            else if let Some(_) = state.error {
                // 即使主图报错，这里的渲染也要确保不阻塞后面的 UI
                ui.scope_builder(UiBuilder::new().max_rect(rect),|ui| {
                    ui.vertical_centered(|ui| {
                        ui.add_space(ui.available_height() * 0.4); // 稍微向下偏移，视觉更舒服
                        ui.label(RichText::new(text.viewer_error)
                            .color(Color32::RED)
                            .size(14.0));
                    });
                });

            } else if state.is_loading {
                // 3. 如果正在加载且没图没报错，这里留白
            }
            // 情况 4：空状态
            else {
                ui.centered_and_justified(|ui| ui.label(text.viewer_drag_hint));
            }

            // --- 2. 渲染导航箭头（无论成功与否，只要列表不为空就显示） ---
            if state.has_nav {
                if let Some(nav_action) = draw_arrows(ui, rect) {
                    action = match nav_action {
                        Nav::Prev => ViewerAction::Prev,
                        Nav::Next => ViewerAction::Next,
                    };
                }
            }
        });
    });

    action
}

fn render_image_viewer(
    ui: &mut Ui,
    tex: &TextureHandle,
    zoom: f32
){
    //--- 手动计算居中逻辑 ---
    let size = tex.size_vec2() * zoom;
    // 获取当前 ScrollArea 内部可用的视口大小
    let available_size = ui.available_size();

    // 1. 判断当前图片是否大于显示区域（只要有一边大，就可以拖拽）
    let is_draggable = size.x > available_size.x || size.y > available_size.y;

    // 2. 如果可以拖拽，且鼠标正在此区域内，改变指针
    if is_draggable {
        // 3. 在闭包外部，根据状态设置图标
        if ui.rect_contains_pointer(ui.max_rect()) {
            ui.ctx().set_cursor_icon(CursorIcon::Move );
        }
    }

    ScrollArea::both()
        .scroll_source(egui::scroll_area::ScrollSource::DRAG)
        .auto_shrink([false; 2])
        .show(ui, |ui| {
            // 计算边距：如果图片比窗口小，则计算一半的差值作为偏移；否则偏移为 0
            let x_offset = (available_size.x - size.x).max(0.0) * 0.5;
            let y_offset = (available_size.y - size.y).max(0.0) * 0.5;

            ui.horizontal(|ui| {
                ui.add_space(x_offset); // 左边距
                ui.vertical(|ui| {
                    ui.add_space(y_offset); // 上边距

                    let img_widget = Image::from_texture(tex)
                        .fit_to_exact_size(size);

                    ui.add(img_widget);
                });
            });
        });
}
