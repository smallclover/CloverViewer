use crate::model::config::init_config_arc;
use crate::{
    core::config_manager::ConfigManager,
    feature::Feature,
    feature::screenshot::ScreenshotFeature,
    feature::viewer::ViewerFeature,
    model::{
        config::{Config, load_config, update_context_config},
        mode::AppMode,
        state::AppState,
    },
    os::current_platform,
    ui::{
        resources::APP_FONT,
        widgets::{modal::ModalAction, tray::init_tray},
    },
    utils::image::load_icon,
};
use eframe::egui;
use egui::{
    Context, FontData, FontDefinitions, FontFamily, Pos2, Ui, Vec2, ViewportBuilder,
    ViewportCommand, WindowLevel,
};
use std::{
    env,
    path::PathBuf,
    sync::{Arc, Mutex},
};
use tray_icon::TrayIcon;

pub fn run() -> eframe::Result<()> {
    // 提前加载配置
    let config = load_config();

    let mut viewport = ViewportBuilder::default()
        .with_transparent(true)
        .with_icon(load_icon());

    // 应用配置文件中的大小，否则默认 1024x768
    if let Some((w, h)) = config.window_size {
        viewport = viewport.with_inner_size([w, h]);
    } else {
        viewport = viewport.with_inner_size([1024.0, 768.0]);
    }

    // 应用配置文件中的位置，否则默认居中
    if let Some((x, y)) = config.window_pos {
        viewport = viewport.with_position([x, y]);
    }

    let options = eframe::NativeOptions {
        viewport,
        ..Default::default()
    };

    let start_path = env::args().nth(1).map(PathBuf::from);

    eframe::run_native(
        "CloverViewer",
        options,
        Box::new(move |cc| {
            Ok(Box::new(CloverApp::new(
                cc, start_path, config, // 将读取好的 config 传给 new()
            )))
        }),
    )
}

pub struct CloverApp {
    _tray: TrayIcon,
    state: AppState,
    config_manager: ConfigManager,
    viewer_feature: ViewerFeature,
    screenshot_feature: ScreenshotFeature,
}

impl CloverApp {
    pub fn new(
        cc: &eframe::CreationContext<'_>,
        start_path: Option<PathBuf>,
        config: Config,
    ) -> Self {
        Self::init_fonts(cc);

        let visible = Arc::new(Mutex::new(true));
        let allow_quit = Arc::new(Mutex::new(false));
        let platform = current_platform();
        let hwnd_usize = platform.get_window_handle(cc);

        let config_arc = Arc::new(config);
        init_config_arc(&cc.egui_ctx, &Arc::clone(&config_arc));

        let state = AppState::new(&cc.egui_ctx, visible, allow_quit, hwnd_usize);

        // 创建托盘，使用 tray_restore_requested 标志在点击时通知模式需要重置
        let tray = init_tray(
            cc,
            &state.common.window_state.visible,
            &state.common.window_state.allow_quit,
            hwnd_usize,
            &state.common.tray_restore_requested,
            &state.common.tray_screenshot_requested,
            &config_arc.hotkeys.show_screenshot,
        );

        // 创建 ConfigManager 用于防抖保存配置
        let config_manager = ConfigManager::new(Arc::clone(&config_arc));

        // 创建 ViewerFeature（持有自己的 ViewerState 副本）
        let mut viewer_feature = ViewerFeature::new();

        // 打开启动路径
        if let Some(path) = start_path {
            viewer_feature
                .state
                .open_new_context(cc.egui_ctx.clone(), path);
        }

        Self {
            _tray: tray,
            state,
            config_manager,
            viewer_feature,
            screenshot_feature: ScreenshotFeature::new(),
        }
    }

    fn init_fonts(cc: &eframe::CreationContext<'_>) {
        let mut fonts = FontDefinitions::default();
        fonts.font_data.insert(
            "my_font".to_owned(),
            Arc::new(FontData::from_static(APP_FONT)),
        );
        if let Some(family) = fonts.families.get_mut(&FontFamily::Proportional) {
            family.insert(0, "my_font".to_owned());
        }
        cc.egui_ctx.set_fonts(fonts);
    }

    /// 处理全局输入事件（窗口关闭等）
    fn handle_global_input(&mut self, ctx: &Context) {
        use crate::model::config::get_context_config;
        use crate::os::current_platform;

        if ctx.input(|i| i.viewport().close_requested()) {
            let config = get_context_config(ctx);
            let Ok(aq) = self.state.common.window_state.allow_quit.lock() else {
                return;
            };
            let Ok(mut vis) = self.state.common.window_state.visible.lock() else {
                return;
            };
            if config.minimize_on_close && !*aq {
                ctx.send_viewport_cmd(egui::ViewportCommand::CancelClose);
                *vis = false;
                current_platform().show_window_hide(self.state.common.window_state.hwnd_usize);
            }
        }

        // 1. 获取所有触发的热键动作
        let actions = self.state.process_hotkey_events();
        // 2. 遍历动作，进行全局广播
        for action in actions {
            // 截图模块热键处理
            if let Some(new_mode) = self.screenshot_feature.handle_hotkey(action.clone()) {
                self.state.mode = new_mode;
            }

            // 图片查看模块： 热键处理
            if let Some(new_mode) = self.viewer_feature.handle_hotkey(action.clone()) {
                self.state.mode = new_mode;
            }
        }
    }

    fn handle_cache_win_pos(&mut self, ctx: &Context, _frame: &mut eframe::Frame) {
        if self.state.mode != AppMode::Viewer {
            return;
        }

        if let Ok(visible) = self.state.common.window_state.visible.lock()
            && !*visible
        {
            return;
        }

        let viewport = ctx.input(|i| i.viewport().clone());

        // 检测最小化状态变化
        let is_minimized = viewport.minimized == Some(true);
        let was_minimized = {
            if let Ok(minimized) = self.state.common.window_state.minimized.lock() {
                *minimized
            } else {
                false
            }
        };

        // 更新最小化状态
        if let Ok(mut minimized) = self.state.common.window_state.minimized.lock() {
            *minimized = is_minimized;
        }

        // 从最小化恢复
        if was_minimized && !is_minimized {
            let current_config = self.config_manager.config();
            tracing::debug!(
                "从最小化恢复，config 中窗口位置: {:?}, 尺寸: {:?}",
                current_config.window_pos,
                current_config.window_size
            );
            ctx.send_viewport_cmd(ViewportCommand::Decorations(true));
            ctx.send_viewport_cmd(ViewportCommand::Transparent(false));
            ctx.send_viewport_cmd(ViewportCommand::WindowLevel(WindowLevel::Normal));

            // 移回截图前的原始位置和尺寸
            if let Some((x, y)) = current_config.window_pos {
                ctx.send_viewport_cmd(ViewportCommand::OuterPosition(Pos2::new(x, y)));
            }
            if let Some((w, h)) = current_config.window_size {
                ctx.send_viewport_cmd(ViewportCommand::InnerSize(Vec2::new(w, h)));
            }
        }

        if is_minimized || viewport.maximized == Some(true) || viewport.fullscreen == Some(true) {
            return;
        }

        if let (Some(outer), Some(inner)) = (viewport.outer_rect, viewport.inner_rect)
            && outer.min.x > -10000.0
            && outer.min.y > -10000.0
            && inner.width() < 4000.0
            && inner.height() < 3000.0
        {
            let current_pos = (outer.min.x, outer.min.y);
            let current_size = (inner.width(), inner.height());

            // 检查是否发生变化
            let current_config = self.config_manager.config();
            let pos_changed = current_config.window_pos != Some(current_pos);
            let size_changed = current_config.window_size != Some(current_size);

            // 鼠标没有任何按键被按下，说明用户的拖拽或缩放动作已经结束
            let no_mouse_down = !ctx.input(|i| i.pointer.any_down());

            if (pos_changed || size_changed) && no_mouse_down {
                // 更新内存配置
                let mut new_config = (*current_config).clone();
                new_config.window_pos = Some(current_pos);
                new_config.window_size = Some(current_size);
                let new_config_arc = Arc::new(new_config);

                // 更新并触发保存
                self.config_manager
                    .update_and_save(Arc::clone(&new_config_arc));

                // 更新 Context 里的配置（保证全局同步）
                update_context_config(ctx, &new_config_arc);
            }
        }
    }

    /// 更新应用配置
    fn handle_update_config(&mut self, ctx: &Context) {
        if let Some(ModalAction::Apply) = self.viewer_feature.get_pending_config_action()
            && let Some(config) = self.viewer_feature.take_pending_config()
        {
            let new_config_arc = Arc::new(config);

            // 直接立刻保存（因为这是手动点确认修改的，无需防抖）
            self.config_manager
                .update_and_save(Arc::clone(&new_config_arc));
            self.config_manager.save_now();

            self.state.reload_hotkeys(&new_config_arc);
            update_context_config(ctx, &new_config_arc)
        }
    }
}

impl eframe::App for CloverApp {
    fn logic(&mut self, ctx: &Context, frame: &mut eframe::Frame) {
        // 处理窗口缓存的位置
        self.handle_cache_win_pos(ctx, frame);
        // 全局输入处理
        self.handle_global_input(ctx);

        // 每帧检查是否需要保存配置（防抖）
        self.config_manager.update(ctx);

        // 检查是否从托盘恢复，若是则重置模式为 Viewer
        if let Ok(mut flag) = self.state.common.tray_restore_requested.lock()
            && *flag
        {
            *flag = false;
            self.state.mode = AppMode::Viewer;
        }

        // 检查是否从托盘请求截图
        if let Ok(mut flag) = self.state.common.tray_screenshot_requested.lock()
            && *flag
        {
            *flag = false;
            use crate::feature::screenshot::state::WindowPrevState;
            self.screenshot_feature
                .enter_screenshot_mode(WindowPrevState::Tray);
            self.state.mode = AppMode::Screenshot;
        }

        let common = &mut self.state.common;
        match self.state.mode {
            AppMode::Viewer => {
                self.viewer_feature
                    .logic(ctx, common, &mut self.state.mode);
            }
            AppMode::Screenshot => {
                self.screenshot_feature
                    .logic(ctx, common, &mut self.state.mode);
            }
        }
    }

    fn ui(&mut self, ui: &mut Ui, _frame: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();
        let common = &mut self.state.common;

        match self.state.mode {
            AppMode::Viewer => {
                self.viewer_feature.ui(ui, common);
                // 处理配置应用（从 overlay 状态）
                self.handle_update_config(&ctx);
            }
            AppMode::Screenshot => {
                self.screenshot_feature
                    .ui(ui, common, &mut self.state.mode);
            }
        }
    }

    fn on_exit(&mut self) {
        // 应用退出时强制保存配置
        self.config_manager.save_now();
    }
}
