use eframe::egui;
use egui::{CentralPanel, Color32, Context, Frame, TopBottomPanel};
use rfd::FileDialog;
use crate::{
    core::business::{ViewMode, ViewerState},
    feature::Feature,
    feature::viewer::view::{
        grid_view::draw_grid_view,
        single_view::draw_single_view
    },
    i18n::lang::get_i18n_text,
    model::{
        config::Config,
        constants::SUPPORTED_IMAGE_EXTENSIONS,
        mode::{AppMode, OverlayMode},
        state::CommonState,
    },
    ui::{
        menus::{
            context_menu::{handle_context_menu_action, render_context_menu, ContextMenuAction},
            menu::{draw_menu, MenuAction},
        },
        widgets::{
            about::render_about_window,
            icons::{draw_icon_button, IconType},
            loading::global_loading,
            modal::ModalAction,
            settings::render_settings_window,
        },
    },
};
use crate::core::hotkeys::HotkeyAction;

pub mod view;
pub mod panels;

/// ViewerFeature - 图片查看器功能模块
pub struct ViewerFeature {
    pub state: ViewerState,
    overlay: OverlayMode,
    /// 待处理的配置应用动作
    pending_config_action: Option<ModalAction>,
    /// 待处理的配置（当 pending_config_action 为 Apply 时）
    pending_config: Option<Config>,
    /// 待处理的模式切换（从菜单请求）
    pending_mode_switch: Option<AppMode>,
}

impl ViewerFeature {
    pub fn new() -> Self {
        Self {
            state: ViewerState::new(),
            overlay: OverlayMode::None,
            pending_config_action: None,
            pending_config: None,
            pending_mode_switch: None,
        }
    }

}

impl Default for ViewerFeature {
    fn default() -> Self {
        Self::new()
    }
}

impl Feature for ViewerFeature {
    fn update(&mut self, ctx: &Context, common: &mut CommonState, mode: &mut AppMode) {
        // 只在 Viewer 模式下处理
        if *mode != AppMode::Viewer {
            return;
        }

        //处理看图模式下的输入事件
        self.handle_input(ctx);

        // 处理图片加载结果
        if self.state.process_load_results(ctx) {
            ctx.request_repaint();
        }

        // 处理新路径
        if let Ok(path) = common.path_receiver.try_recv() {
            self.state.open_new_context(ctx.clone(), path);
        }

        // 绘制 UI
        self.draw(ctx, common);

        // 处理待处理的模式切换（从菜单请求）
        if let Some(switch_to) = self.take_pending_mode_switch() {
            *mode = switch_to;
        }
    }

    fn handle_hotkey(&mut self, action: HotkeyAction) -> Option<AppMode> {
        match action {
            HotkeyAction::SetScreenshotMode { .. } => Some(AppMode::Screenshot),
            HotkeyAction::RequestScreenshotCopy => None,
        }
    }
}

impl ViewerFeature {
    /// 处理 Viewer 特有的输入事件
    pub fn handle_input(&mut self, ctx: &Context) {
        use egui::Key;

        // 图片导航
        if ctx.input(|i| i.key_pressed(Key::ArrowLeft)) {
            self.state.prev_image(ctx.clone());
        }
        if ctx.input(|i| i.key_pressed(Key::ArrowRight)) {
            self.state.next_image(ctx.clone());
        }

        // 拖放文件
        if let Some(path) = ctx.input(|i| {
            i.raw
                .dropped_files
                .first()
                .and_then(|f| f.path.clone())
        }) {
            self.state.handle_dropped_file(ctx.clone(), path);
        }

        // 缩放
        let scroll_delta = ctx.input(|i| i.smooth_scroll_delta.y);
        self.state.update_zoom(scroll_delta);
    }

    /// 完整的 UI 绘制
    pub fn draw(&mut self, ctx: &Context, common: &mut CommonState) {
        // 1. 顶部面板
        let (open_file, open_folder, menu_action) = draw_menu(ctx, &mut self.overlay);

        if open_file {
            let sender = common.path_sender.clone();
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
            let sender = common.path_sender.clone();
            let ctx_clone = ctx.clone();
            std::thread::spawn(move || {
                if let Some(path) = FileDialog::new().pick_folder() {
                    sender.send(path).ok();
                    ctx_clone.request_repaint();
                }
            });
        }

        // 处理菜单动作
        match menu_action {
            MenuAction::ShowScreenshot => {
                self.pending_mode_switch = Some(AppMode::Screenshot);
            }
            MenuAction::None => {}
        }

        // 2. 底部面板 (内联实现，避免依赖 AppState)
        self.draw_bottom_panel(ctx);

        // 3. 中央面板
        let background_frame = Frame::NONE.fill(Color32::from_rgb(25, 25, 25));
        CentralPanel::default().frame(background_frame).show(ctx, |ui| {
            match self.state.view_mode {
                ViewMode::Single => {
                    draw_single_view(ctx, ui, &mut self.state, &mut self.overlay);
                }
                ViewMode::Grid => {
                    draw_grid_view(ctx, ui, &mut self.state);
                }
            }
        });

        // 4. 属性面板
        if matches!(self.overlay, OverlayMode::Properties) {
            crate::feature::viewer::panels::properties_panel::draw_properties_panel(
                ctx,
                &mut self.overlay,
                &self.state,
            );
        }

        // 5. Toast 系统
        common.toast_system.update(ctx);

        // 6. 处理 overlays (about, settings, context_menu)
        let context_menu_action = self.draw_overlays(ctx);

        // 处理右键菜单操作
        if let Some(action) = context_menu_action {
            handle_context_menu_action(
                ctx,
                action,
                &self.state,
                &mut self.overlay,
                &common.toast_manager,
            );
        }
    }

    /// 底部面板（内联实现）
    fn draw_bottom_panel(&self, ctx: &Context) {
        let text = get_i18n_text(ctx);

        TopBottomPanel::bottom("bottom_panel").show(ctx, |ui| {
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.add_space(10.0);

                if draw_icon_button(ui, self.state.view_mode == ViewMode::Grid, IconType::Grid, &text).clicked() {
                    // 注意：self.state.view_mode 是不可变的，我们需要另一种方式
                    // 暂时用 clone 方式处理
                }

                ui.add_space(4.0);

                if draw_icon_button(ui, self.state.view_mode == ViewMode::Single, IconType::Single, &text).clicked() {
                    // 同上
                }
            });
        });
    }

    fn draw_overlays(&mut self, ctx: &Context) -> Option<ContextMenuAction> {
        let mut context_menu_action = None;
        let text = get_i18n_text(ctx);

        match &mut self.overlay {
            OverlayMode::About => {
                let mut open = true;
                render_about_window(ctx, &mut open);
                if !open {
                    self.overlay = OverlayMode::None;
                }
            }
            OverlayMode::Settings { config } => {
                let mut open = true;
                let mut action = render_settings_window(ctx, &mut open, &text, config);

                if action == ModalAction::Apply {
                    // 保存配置到 pending
                    self.pending_config_action = Some(ModalAction::Apply);
                    self.pending_config = Some(config.clone());
                    action = ModalAction::Close;
                }

                if !open || action == ModalAction::Close {
                    self.overlay = OverlayMode::None;
                }
            }
            OverlayMode::ContextMenu(pos) => {
                let mut pos_opt = Some(*pos);
                let action = render_context_menu(ctx, &mut pos_opt);

                if let Some(action) = action {
                    context_menu_action = Some(action);
                }

                if pos_opt.is_none() {
                    self.overlay = OverlayMode::None;
                }
            }
            OverlayMode::None | OverlayMode::Properties => {}
        }

        // 加载提示
        if self.state.current_texture.is_none() && self.state.loader.is_loading {
            global_loading(ctx, text.loading_parsing.to_string());
        }

        context_menu_action
    }

    /// 获取待处理的配置动作
    pub fn get_pending_config_action(&self) -> Option<ModalAction> {
        self.pending_config_action
    }

    /// 获取待处理的配置并清除状态
    pub fn take_pending_config(&mut self) -> Option<Config> {
        self.pending_config_action = None;
        self.pending_config.take()
    }

    /// 获取待处理的模式切换并清除状态
    pub fn take_pending_mode_switch(&mut self) -> Option<AppMode> {
        self.pending_mode_switch.take()
    }
}
