use eframe::egui;
use egui::{
    CentralPanel, Color32, Context, Frame,
};
use rfd::FileDialog;
use crate::{
    core::business::BusinessData,
    i18n::lang::get_text,
    model::{
        config::Config,
        state::{ViewMode, ViewState},
        constants::SUPPORTED_IMAGE_EXTENSIONS
    },
    ui::components::{
        about::render_about_window,
        context_menu::{render_context_menu, ContextMenuAction},
        loading::global_loading,
        menu::draw_menu,
        modal::ModalAction,
        settings::render_settings_window,
        status_bar::draw_status_bar,
        ui_mode::UiMode,
    },
};
use crate::ui::components::{
    grid_view::draw_grid_view,
    single_view::draw_single_view
};

pub fn draw_top_panel(
    ctx: &Context,
    state: &mut ViewState,
    config: &Config,
    screenshot_active: &mut bool,
) {
    let texts = get_text(config.language);
    let (open_file, open_folder) = draw_menu(ctx, &mut state.ui_mode, texts, config, screenshot_active);

    if open_file {
        let sender = state.path_sender.clone();
        std::thread::spawn(move || {
            if let Some(path) = FileDialog::new()
                .add_filter("Image", SUPPORTED_IMAGE_EXTENSIONS)
                .pick_file()
            {
                sender.send(path).ok();
            }
        });
    }

    if open_folder {
        let sender = state.path_sender.clone();
        std::thread::spawn(move || {
            if let Some(path) = FileDialog::new().pick_folder() {
                sender.send(path).ok();
            }
        });
    }
}

pub fn draw_bottom_panel(
    ctx: &Context,
    state: &mut ViewState,
    config: &Config,
) {
    draw_status_bar(ctx, state,config);
}

pub fn draw_central_panel(
    ctx: &Context,
    data: &mut BusinessData,
    state: &mut ViewState,
    config: &Config,
) {
    let texts = get_text(config.language);
    let background_frame = Frame::NONE.fill(Color32::from_rgb(25, 25, 25));

    CentralPanel::default().frame(background_frame).show(ctx, |ui| {
        match state.view_mode {
            ViewMode::Single => {
                draw_single_view(ctx, ui, data, state, texts);
            }
            ViewMode::Grid => {
                draw_grid_view(ctx, ui, data, state, config);
            }
        }
    });
}

pub fn draw_overlays(
    ctx: &Context,
    data: &BusinessData,
    state: &mut ViewState,
    temp_config: &mut Config,
) -> (Option<ContextMenuAction>, Option<ModalAction>) {
    let mut context_menu_action = None;
    let mut modal_action = None;
    let mut new_ui_mode = None;
    let texts = get_text(temp_config.language);

    match &mut state.ui_mode {
        UiMode::About => {
            let mut open = true;
            render_about_window(ctx, &mut open, texts);
            if !open {
                new_ui_mode = Some(UiMode::Normal);
            }
        }
        UiMode::Settings(cfg) => {
            let mut open = true;
            let mut action = render_settings_window(ctx, &mut open, texts, cfg);

            if action == ModalAction::Apply {
                *temp_config = cfg.clone();
                modal_action = Some(ModalAction::Apply);
                action = ModalAction::Close;
            }

            if !open || action == ModalAction::Close {
                new_ui_mode = Some(UiMode::Normal);
            }
        }
        UiMode::ContextMenu(pos) => {
            let mut pos_opt = Some(*pos);
            let action = render_context_menu(ctx, &mut pos_opt, texts);

            if let Some(action) = action {
                context_menu_action = Some(action);
            }

            if pos_opt.is_none() {
                new_ui_mode = Some(UiMode::Normal);
            }
        }
        UiMode::Normal => {}
        UiMode::Properties => {}
    }

    if let Some(new_mode) = new_ui_mode {
        state.ui_mode = new_mode;
    }

    if data.current_texture.is_none() && data.loader.is_loading {
        global_loading(ctx, texts.loading_parsing.to_string());
    }

    (context_menu_action, modal_action)
}
