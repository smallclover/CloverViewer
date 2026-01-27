use egui::{
    Context, TopBottomPanel, MenuBar,
};
use rfd::FileDialog;
use std::path::PathBuf;
use crate::{constants::SUPPORTED_IMAGE_EXTENSIONS, i18n::{get_text, Language}};
use crate::ui::ui_mode::UiMode;

pub fn draw_menu(
    ctx: &Context,
    ui_mode: &mut UiMode,
    lang: Language
) -> Option<PathBuf> {
    let mut picked_path = None;
    let text = get_text(lang);

    TopBottomPanel::top("menu").show(ctx, |ui| {
        // Only enable menu if in Normal mode (or ContextMenu, though usually menu is disabled then too)
        // Actually, if a modal is open (About/Settings), we usually disable the menu.
        let enabled = matches!(ui_mode, UiMode::Normal | UiMode::ContextMenu(_));

        ui.add_enabled_ui(enabled, |ui| {
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
                        *ui_mode = UiMode::Settings;
                        ui.close();
                    }
                });

                // --- 2. 追加“关于”按钮 ---
                if ui.button(text.menu_about).clicked() {
                    *ui_mode = UiMode::About;
                }

            });

        });
    });

    picked_path
}
