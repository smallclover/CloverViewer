use eframe::egui;
use egui::{
    CentralPanel, Color32, Context, Frame, Key
};
use rfd::FileDialog;
use crate::{
    core::business::BusinessData,
    model::{
        config::{Config,get_context_config},
        state::{ViewMode, ViewState},
        constants::SUPPORTED_IMAGE_EXTENSIONS
    },
    ui::{
        panels::{
            about::render_about_window,
            settings::render_settings_window,
        },
        menus::{
            menu::draw_menu,
            context_menu::{render_context_menu, ContextMenuAction},
            status_bar::draw_status_bar,
        },
        widgets::{
            loading::global_loading,
            modal::ModalAction,
        },
        view::{
            grid_view::draw_grid_view,
            single_view::draw_single_view
        },
        mode::UiMode,
    },
    i18n::lang::get_i18n_text,
    state::custom_window::WindowState,
    os::window::show_window_hide
};

pub fn draw_top_panel(
    ctx: &Context,
    state: &mut ViewState,
) {

    let (open_file, open_folder) = draw_menu(ctx, &mut state.ui_mode);

    if open_file {
        let sender = state.path_sender.clone();
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
        let sender = state.path_sender.clone();
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
    state: &mut ViewState,
) {
    draw_status_bar(ctx, state);
}

pub fn draw_central_panel(
    ctx: &Context,
    data: &mut BusinessData,
    state: &mut ViewState,
) {
    let background_frame = Frame::NONE.fill(Color32::from_rgb(25, 25, 25));

    CentralPanel::default().frame(background_frame).show(ctx, |ui| {
        match state.view_mode {
            ViewMode::Single => {
                draw_single_view(ctx, ui, data, state);
            }
            ViewMode::Grid => {
                draw_grid_view(ctx, ui, data, state);
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

    let text = get_i18n_text(ctx);

    match &mut state.ui_mode {
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
        state.ui_mode = new_mode;
    }

    if data.current_texture.is_none() && data.loader.is_loading {
        global_loading(ctx, text.loading_parsing.to_string());
    }

    (context_menu_action, modal_action)
}

pub fn handle_input_events(ctx: &Context, data: &mut BusinessData, window_state: &WindowState) {

    if ctx.input(|i| i.viewport().close_requested()){
        let config = get_context_config(ctx);
        let aq = window_state.allow_quit.lock().unwrap();
        let mut vis = window_state.visible.lock().unwrap();
        // 设置：允许最小化到托盘,且不是点击托盘退出
        if config.minimize_on_close && !*aq {
            ctx.send_viewport_cmd(egui::ViewportCommand::CancelClose);
            *vis = false;
            // 隐藏程序
            show_window_hide(window_state.hwnd_isize);
        }else{
            // 关闭程序
        }
    }

    if ctx.input(|i| i.key_pressed(Key::ArrowLeft)) {
        data.prev_image(ctx.clone());
    }
    if ctx.input(|i| i.key_pressed(Key::ArrowRight)) {
        data.next_image(ctx.clone());
    }

    if let Some(path) = ctx.input(|i| {
        i.raw
            .dropped_files
            .first()
            .and_then(|f| f.path.clone())
    }) {
        data.handle_dropped_file(ctx.clone(), path);
    }

    let scroll_delta = ctx.input(|i| i.smooth_scroll_delta.y);
    data.update_zoom(scroll_delta);
}
