use egui::{
    Context, TopBottomPanel, MenuBar,
    Area,Id,Order,Sense,Window,Align2,
    Color32, Pos2, Frame
};
use rfd::FileDialog;
use std::path::PathBuf;
use crate::{constants::SUPPORTED_IMAGE_EXTENSIONS, dev_info, i18n::{get_text, Language}};

pub fn draw_menu(
    ctx: &Context,
    show_about: &mut bool,
    show_settings: &mut bool,
    lang: Language
) -> Option<PathBuf> {
    let mut picked_path = None;
    let text = get_text(lang);

    TopBottomPanel::top("menu").show(ctx, |ui| {
        ui.add_enabled_ui(!*show_about && !*show_settings, |ui| {
            MenuBar::new().ui(ui, |ui| {
                // “文件”菜单
                ui.menu_button(text.menu_file, |ui| {

                    ui.set_min_width(130.0);

                    if ui.button(text.menu_open_file).clicked() {
                        if let Some(path) = FileDialog::new()
                            .add_filter("Image", SUPPORTED_IMAGE_EXTENSIONS)
                            .pick_file() {
                            picked_path = Some(path);
                        }
                        ui.close()
                    }
                    if ui.button(text.menu_open_folder).clicked() {
                        if let Some(path) = FileDialog::new()
                            .pick_folder() {
                            picked_path = Some(path);
                        }
                        ui.close()
                    }

                    ui.separator();

                    // 设置
                    if ui.button(text.menu_settings).clicked() {
                        *show_settings = true;
                        ui.close();
                    }
                });

                // --- 2. 追加“关于”按钮 ---
                if ui.button(text.menu_about).clicked() {
                    *show_about = true;
                    ui.close()
                }

            });

        });
    });

    picked_path
}

pub fn render_context_menu(
    ctx: &Context,
    pos: &mut Option<Pos2>,
    lang: Language
) {
    if let Some(position) = pos {
        let text = get_text(lang);
        let mut close_menu = false;

        // 1. 绘制一个全屏的透明遮罩层，用于捕获点击并关闭菜单
        // 它的 Order 必须比菜单低，但比主界面高
        // 菜单通常在 Foreground，我们可以把遮罩放在 Middle 或者 Foreground-1
        // 但 egui 的 Order 比较简单。
        // 我们可以先画遮罩，再画菜单。因为它们都是 Area，后画的在上面（如果 Order 相同）。

        // 使用一个覆盖全屏的 Area
        Area::new(Id::new("context_menu_mask"))
            .order(Order::Foreground) // 和菜单同一层级，但先画，所以在下面
            .fixed_pos(Pos2::ZERO)
            .show(ctx, |ui| {
                // 分配整个屏幕的空间
                let screen_rect = ctx.input(|i| i.content_rect());
                let response = ui.allocate_rect(screen_rect, Sense::click());
                if response.clicked_by(egui::PointerButton::Primary) {
                    close_menu = true;
                }
            });

        // 2. 绘制实际的菜单
        Area::new(Id::new("context_menu"))
            .order(Order::Foreground) // 也是 Foreground，后画，所以在遮罩上面
            .fixed_pos(*position)
            .show(ctx, |ui| {
                Frame::menu(ui.style()).show(ui, |ui| {
                    ui.set_min_width(120.0);
                    if ui.button(text.context_copy_image).clicked() {
                        dev_info!("Copy Image clicked");
                        close_menu = true;
                    }
                    if ui.button(text.context_copy_path).clicked() {
                        dev_info!("Copy Image Path clicked");
                        close_menu = true;
                    }
                });
            });

        if close_menu {
            *pos = None;
        }
    }
}

pub fn render_about_window(ctx: &Context, show_about: &mut bool, lang: Language) {
    if !*show_about {
        return;
    }

    let text = get_text(lang);
    let screen_rect = ctx.content_rect();

    // 背景遮罩（拦截主窗口点击 + 发声音）
    Area::new(Id::new("modal_dimmer"))
        .order(Order::Background)
        .fixed_pos(screen_rect.min)
        .show(ctx, |ui| {
            // 吃掉点击
            let response = ui.allocate_rect(screen_rect, Sense::click());

            // 调用各个平台的API来发声
            if response.clicked() {
                // 发声音
                play_error_beep();
            }

            // 半透明遮罩
            ui.painter().rect_filled(
                screen_rect,
                0.0,
                Color32::from_rgba_unmultiplied(0, 0, 0, 150),
            );
        });

    let mut request_close = false;
    // 渲染实际的可拖动窗口
    Window::new(text.about_title)
        .open(show_about) // 提供 [X] 关闭按钮
        .collapsible(false)
        .resizable(false)
        .default_pos(ctx.content_rect().center())
        // 设置轴心点为中心，这样窗口的“中心”会对齐到 App 的“中心”
        .pivot(Align2::CENTER_CENTER)
        .show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.heading("CloverViewer");
                ui.label(text.about_desc);
                ui.add_space(8.0);
                ui.hyperlink_to(text.about_github, "https://github.com/smallclover/CloverViewer");
                ui.add_space(12.0);
                if ui.button(text.about_close).clicked() {
                    request_close = true; // 只记录意图
                }
            });
        });

    // ===== 统一处理关闭逻辑 =====
    if request_close {
        *show_about = false;
    }
}

fn play_error_beep() {
    #[cfg(target_os = "windows")]
    unsafe {
        #[link(name = "user32")]
        unsafe  extern "system" { fn MessageBeep(u: u32) -> i32; }
        // uType 为 0xFFFFFFFF 是标准的提示声
        MessageBeep(0xFFFFFFFF);
    }

    // #[cfg(target_os = "macos")]
    // let _ = std::process::Command::new("osascript").args(&["-e", "beep"]).spawn();
    //
    // #[cfg(target_os = "linux")]
    // let _ = std::process::Command::new("bash").args(&["-c", "echo -e '\\a'"]).spawn();
}