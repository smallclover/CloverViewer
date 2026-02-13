use crate::{
    ui::components::ui_mode::{UiMode},
};
use egui::{Context, MenuBar, TopBottomPanel};
use crate::i18n::lang::TextBundle;
use crate::model::config::Config;

/// 绘制主菜单栏
///
/// ## 返回
///
/// 返回一个元组 `(bool, bool)`，分别表示：
/// 1. 是否点击了“打开文件”
/// 2. 是否点击了“打开文件夹”
pub fn draw_menu(
    ctx: &Context,
    ui_mode: &mut UiMode,
    text: &TextBundle,
    config: &Config,
) -> (bool, bool) {

    let mut open_file_dialog = false;
    let mut open_folder_dialog = false;

    TopBottomPanel::top("menu").show(ctx, |ui| {
        MenuBar::new().ui(ui, |ui| {
            // “文件”菜单
            ui.menu_button(text.menu_file, |ui| {
                ui.set_min_width(130.0);

                if ui.button(text.menu_open_file).clicked() {
                    open_file_dialog = true;
                    ui.close();
                }
                if ui.button(text.menu_open_folder).clicked() {
                    open_folder_dialog = true;
                    ui.close();
                }

                ui.separator();

                // 设置
                if ui.button(text.menu_settings).clicked() {
                    *ui_mode = UiMode::Settings(config.clone());
                    ui.close();
                }
            });

            // “编辑”菜单
            ui.menu_button(text.menu_edit, |ui| {
                ui.set_min_width(130.0);

                if ui.add(egui::Button::new(text.menu_screenshot).shortcut_text("Alt+S")).clicked() {
                    *ui_mode = UiMode::Screenshot;
                    ui.close();
                }
            });

            // --- 2. 追加“关于”按钮 ---
            if ui.button(text.menu_about).clicked() {
                *ui_mode = UiMode::About;
            }
        });
    });

    (open_file_dialog, open_folder_dialog)
}
