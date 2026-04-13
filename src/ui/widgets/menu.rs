use crate::i18n::lang::get_i18n_text;
use crate::model::config::get_context_config;
use crate::model::mode::OverlayMode;
use egui::{Button, MenuBar, Panel, Ui};

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
pub fn draw_menu(ui: &mut Ui, overlay: &mut OverlayMode) -> (bool, bool, MenuAction) {
    let mut open_file_dialog = false;
    let mut open_folder_dialog = false;
    let mut action = MenuAction::None;

    let text = get_i18n_text(ui);
    let config = get_context_config(ui);

    Panel::top("top_panel").show_inside(ui, |ui| {
        MenuBar::new().ui(ui, |ui| {
            ui.menu_button(text.menu.file, |ui| {
                if ui.button(text.menu.open_file).clicked() {
                    open_file_dialog = true;
                    ui.close();
                }
                if ui.button(text.menu.open_folder).clicked() {
                    open_folder_dialog = true;
                    ui.close();
                }

                ui.separator();

                if ui.button(text.menu.settings).clicked() {
                    let config = get_context_config(ui);
                    *overlay = OverlayMode::Settings {
                        config: (*config).clone(),
                    };
                    ui.close();
                }

                ui.separator();

                if ui.button(text.menu.exit).clicked() {
                    ui.send_viewport_cmd(egui::ViewportCommand::Close);
                }
            });

            ui.menu_button(text.menu.edit, |ui| {
                if ui
                    .add(
                        Button::new(text.menu.screenshot)
                            .shortcut_text(&config.hotkeys.show_screenshot),
                    )
                    .clicked()
                {
                    action = MenuAction::ShowScreenshot;
                    ui.close();
                }
            });

            ui.menu_button(text.menu.help, |ui| {
                if ui.button(text.menu.about).clicked() {
                    *overlay = OverlayMode::About;
                    ui.close();
                }
            });
        });
    });

    (open_file_dialog, open_folder_dialog, action)
}
