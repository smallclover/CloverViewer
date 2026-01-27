use egui::{
    Context, TopBottomPanel, MenuBar,
};
use rfd::FileDialog;
use std::path::PathBuf;
use crate::{constants::SUPPORTED_IMAGE_EXTENSIONS, i18n::{get_text, Language}};

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
                }

            });

        });
    });

    picked_path
}
