use eframe::egui;
use egui::{
    CentralPanel, Color32, Context, CursorIcon, Frame,
    Image, RichText, ScrollArea, TextureHandle, Ui, UiBuilder, Pos2, Rect
};
use crate::ui::arrows::{draw_arrows, Nav};
use crate::ui::loading::corner_loading;
use crate::i18n::{get_text, Language};

pub enum ViewerAction {
    Prev,
    Next,
    None,
    ContextMenu(Pos2), // 新增：右键菜单请求，携带点击位置
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

    // 创建深色背景
    let background_frame = Frame::NONE.fill(Color32::from_rgb(25, 25, 25));

    CentralPanel::default().frame(background_frame).show(ctx, |ui| {
        ui.add_enabled_ui(!show_about, |ui| {
            let rect = ui.available_rect_before_wrap();

            // --- 1. 渲染图片内容 ---
            if let Some(tex) = state.texture {
                // 渲染图片，不返回交互响应
                render_image_viewer(ui, tex, state.zoom);

                // --- 2. 全局右键检测 ---
                // 核心逻辑：不使用 interact 捕获，而是直接检查输入状态
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
                                action = ViewerAction::ContextMenu(pos);
                            }
                        }
                    }
                }

                if state.is_loading {
                    corner_loading(ui);
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
                if let Some(nav_action) = draw_arrows(ui, rect) {
                    // 只有在没有其他动作时才响应箭头
                    if matches!(action, ViewerAction::None) {
                        action = match nav_action {
                            Nav::Prev => ViewerAction::Prev,
                            Nav::Next => ViewerAction::Next,
                        };
                    }
                }
            }
        });
    });

    action
}

/// 渲染图片查看器
fn render_image_viewer(
    ui: &mut Ui,
    tex: &TextureHandle,
    zoom: f32
) {
    let size = tex.size_vec2() * zoom;
    let available_size = ui.available_size();
    let is_draggable = size.x > available_size.x || size.y > available_size.y;

    if is_draggable {
        if ui.rect_contains_pointer(ui.max_rect()) {
            ui.ctx().set_cursor_icon(CursorIcon::Move);
        }
    }

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
                    let img_widget = Image::from_texture(tex).fit_to_exact_size(size);
                    ui.add(img_widget);
                });
            });
        });
}
