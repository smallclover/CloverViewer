use crate::model::mode::OverlayMode;
use egui::{Button, Context, MenuBar, TopBottomPanel};
use crate::i18n::lang::{get_i18n_text};
use crate::model::config::{get_context_config};

/// 菜单动作
#[derive(Default)]
pub enum MenuAction {
    #[default]
    None,
    ShowScreenshot,
}

/// 绘制主菜单栏
///
/// ## 返回
///
/// 返回一个元组 `(bool, bool, MenuAction)`，分别表示：
/// 1. 是否点击了"打开文件"
/// 2. 是否点击了"打开文件夹"
/// 3. 菜单动作
pub fn draw_menu(
    ctx: &Context,
    overlay: &mut OverlayMode,
) -> (bool, bool, MenuAction) {

    let mut open_file_dialog = false;
    let mut open_folder_dialog = false;
    let mut action = MenuAction::None;

    let text = get_i18n_text(ctx);
    let config = get_context_config(ctx);

    TopBottomPanel::top("top_panel").show(ctx, |ui| {
        MenuBar::new().ui(ui, |ui| {
            ui.menu_button(text.menu_file, |ui| {
                if ui.button(text.menu_open_file).clicked() {
                    open_file_dialog = true;
                    ui.close();
                }
                if ui.button(text.menu_open_folder).clicked() {
                    open_folder_dialog = true;
                    ui.close();
                }

                ui.separator();

                if ui.button(text.menu_settings).clicked() {
                    let config = get_context_config(ctx);
                    *overlay = OverlayMode::Settings { config: (*config).clone() };
                    ui.close();
                }

                ui.separator();

                if ui.button(text.menu_exit).clicked() {
                    ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                }
            });

            ui.menu_button(text.menu_edit, |ui| {
                if ui.add(Button::new(text.menu_screenshot).shortcut_text(&config.hotkeys.show_screenshot)).clicked(){
                    action = MenuAction::ShowScreenshot;
                    ui.close();
                }
            });

            ui.menu_button(text.menu_help, |ui| {
                if ui.button(text.menu_about).clicked() {
                    *overlay = OverlayMode::About;
                    ui.close();
                }
            });
        });
    });

    (open_file_dialog, open_folder_dialog, action)
}