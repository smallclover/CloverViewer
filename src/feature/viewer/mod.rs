use eframe::egui;
use egui::{
    CentralPanel, Color32, Context, Frame
};
use rfd::FileDialog;
use crate::{
    core::business::{ViewMode, ViewerState},
    feature::viewer::view::{
        grid_view::draw_grid_view,
        single_view::draw_single_view
    },
    i18n::lang::get_i18n_text,
    model::{
        config::Config,
        constants::SUPPORTED_IMAGE_EXTENSIONS,
        mode::UiMode,
        state::AppState,
    },
    ui::{
        menus::{
            context_menu::{render_context_menu, ContextMenuAction},
            menu::draw_menu,
            status_bar::draw_status_bar,
        },
        widgets::{
            loading::global_loading,
            modal::ModalAction,
        },
    },
};
use crate::ui::widgets::about::render_about_window;
use crate::ui::widgets::settings::render_settings_window;

pub mod view;
pub mod panels;

pub fn draw_top_panel(
    ctx: &Context,
    state: &mut AppState,
) {

    let (open_file, open_folder) = draw_menu(ctx, &mut state.ui_mode);

    if open_file {
        let sender = state.common.path_sender.clone();
        let ctx_clone = ctx.clone();
        std::thread::spawn(move || {
            if let Some(path) = FileDialog::new()
                .add_filter("Image", SUPPORTED_IMAGE_EXTENSIONS)
                .pick_file()
            {
                sender.send(path).ok();
                ctx_clone.request_repaint();
            }
        });
    }

    if open_folder {
        let sender = state.common.path_sender.clone();
        let ctx_clone = ctx.clone();
        std::thread::spawn(move || {
            if let Some(path) = FileDialog::new().pick_folder() {
                sender.send(path).ok();
                ctx_clone.request_repaint();
            }
        });
    }
}

pub fn draw_bottom_panel(
    ctx: &Context,
    state: &mut AppState,
) {
    draw_status_bar(ctx, state);
}

pub fn draw_central_panel(
    ctx: &Context,
    state: &mut AppState,
) {
    let background_frame = Frame::NONE.fill(Color32::from_rgb(25, 25, 25));

    CentralPanel::default().frame(background_frame).show(ctx, |ui| {
        match state.viewer.view_mode {
            ViewMode::Single => {
                draw_single_view(ctx, ui, &mut state.viewer, &mut state.ui_mode);
            }
            ViewMode::Grid => {
                draw_grid_view(ctx, ui, &mut state.viewer);
            }
        }
    });
}

pub fn draw_overlays(
    ctx: &Context,
    viewer: &ViewerState,
    ui_mode: &mut UiMode,
    temp_config: &mut Config,
) -> (Option<ContextMenuAction>, Option<ModalAction>) {
    let mut context_menu_action = None;
    let mut modal_action = None;
    let mut new_ui_mode = None;

    let text = get_i18n_text(ctx);

    match ui_mode {
        UiMode::About => {
            let mut open = true;
            render_about_window(ctx, &mut open);
            if !open {
                new_ui_mode = Some(UiMode::Normal);
            }
        }
        UiMode::Settings(cfg) => {
            let mut open = true;
            let mut action = render_settings_window(ctx, &mut open, &text, cfg);

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
            let action = render_context_menu(ctx, &mut pos_opt);

            if let Some(action) = action {
                context_menu_action = Some(action);
            }

            if pos_opt.is_none() {
                new_ui_mode = Some(UiMode::Normal);
            }
        }
        UiMode::Normal => {}
        UiMode::Properties => {}
        UiMode::Screenshot => {}
    }

    if let Some(new_mode) = new_ui_mode {
        *ui_mode = new_mode;
    }

    if viewer.current_texture.is_none() && viewer.loader.is_loading {
        global_loading(ctx, text.loading_parsing.to_string());
    }

    (context_menu_action, modal_action)
}
