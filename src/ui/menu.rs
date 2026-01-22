use egui::{Context};
use rfd::FileDialog;
use std::path::PathBuf;
use crate::constants::SUPPORTED_IMAGE_EXTENSIONS;

// 1. 让 draw_menu 返回一个结果，而不是执行回调
pub fn draw_menu(ctx: &Context, show_about: &mut bool) -> Option<PathBuf> {
    let mut picked_path = None;

    egui::TopBottomPanel::top("menu").show(ctx, |ui| {

        ui.add_enabled_ui(!*show_about, |ui| {
            egui::MenuBar::new().ui(ui, |ui| {
                // “文件”菜单
                ui.menu_button("文件", |ui| {

                    ui.set_min_width(130.0);

                    if ui.button("打开文件…").clicked() {
                        if let Some(path) = FileDialog::new()
                            .add_filter("Image", SUPPORTED_IMAGE_EXTENSIONS)
                            .pick_file() {
                            picked_path = Some(path);
                        }
                        ui.close()
                    }
                    if ui.button("打开文件夹…").clicked() {
                        if let Some(path) = FileDialog::new()
                            .pick_folder() {
                            picked_path = Some(path);
                        }
                        ui.close()
                    }
                });
                // --- 2. 追加“关于”按钮 ---
                if ui.button("关于").clicked() {
                    *show_about = true;
                    ui.close()
                }

            });

        });
    });

    picked_path
}

pub fn render_about_window(ctx: &Context, show_about: &mut bool) {
    if !*show_about {
        return;
    }

    let screen_rect = ctx.content_rect();

    // 背景遮罩（拦截主窗口点击 + 发声音）
    egui::Area::new(egui::Id::new("modal_dimmer"))
        .order(egui::Order::Background)
        .fixed_pos(screen_rect.min)
        .show(ctx, |ui| {
            // 吃掉点击
            let response = ui.allocate_rect(screen_rect, egui::Sense::click());

            // 调用各个平台的API来发声
            if response.clicked() {
                // 发声音
                play_error_beep();
            }

            // 半透明遮罩
            ui.painter().rect_filled(
                screen_rect,
                0.0,
                egui::Color32::from_rgba_unmultiplied(0, 0, 0, 150),
            );
        });

    let mut request_close = false;
    // 渲染实际的可拖动窗口
    egui::Window::new("关于项目")
        .open(show_about) // 提供 [X] 关闭按钮
        .collapsible(false)
        .resizable(false)
        .default_pos(ctx.content_rect().center())
        // 设置轴心点为中心，这样窗口的“中心”会对齐到 App 的“中心”
        .pivot(egui::Align2::CENTER_CENTER)
        .show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.heading("CloverViewer");
                ui.label("Rust 图片查看器");
                ui.add_space(8.0);
                ui.hyperlink_to("GitHub 源码地址", "https://github.com/smallclover/CloverViewer");
                ui.add_space(12.0);
                if ui.button("我知道了").clicked() {
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