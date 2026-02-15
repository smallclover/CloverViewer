use eframe::egui;
use egui::{Context, FontData, FontDefinitions, FontFamily, ViewportBuilder, Id};
use std::{
    path::PathBuf,
    sync::{mpsc, Arc},
    env
};
use global_hotkey::{
    {GlobalHotKeyEvent, GlobalHotKeyManager},
    hotkey::HotKey
};
use global_hotkey::hotkey::{Code, Modifiers};
use crate::{
    core::business::BusinessData,
    model::{
        config::{load_config, save_config, Config},
        state::{ViewMode, ViewState},
    },
};
use crate::ui::{
    components::{
        context_menu::handle_context_menu_action,
        modal::ModalAction,
        mouse::handle_input_events,
        properties_panel::draw_properties_panel,
        resources::APP_FONT,
        screenshot::{handle_screenshot_system},
        ui_mode::UiMode,
    },
    viewer
};

use crate::utils::image::load_icon;
pub fn run() -> eframe::Result<()> {

    // 初始化热键管理器
    let hotkeys_manager = GlobalHotKeyManager::new().unwrap();

    // 热键 1: Alt + S (唤醒/截图)
    let mut mods_s = Modifiers::empty();
    mods_s.insert(Modifiers::ALT);
    let hotkey_show = HotKey::new(Some(mods_s), Code::KeyS);

    // 热键 2: Ctrl + C (仅在截图模式下注册，这里只初始化不注册)
    let mut mods_c = Modifiers::empty();
    mods_c.insert(Modifiers::CONTROL);
    let hotkey_copy = HotKey::new(Some(mods_c), Code::KeyC);

    // 初始只注册唤醒键
    hotkeys_manager.register(hotkey_show).unwrap();


    let mut options = eframe::NativeOptions {
        viewport: ViewportBuilder::default().with_inner_size([1024.0, 768.0]),
        ..Default::default()
    };
    options.viewport = options.viewport.with_icon(load_icon());

    //当应用被设置为默认程序时，传递的图片的路径
    let start_path = env::args().nth(1).map(PathBuf::from);

    eframe::run_native(
        "CloverViewer",
        options,
        Box::new(move |cc| {
            Ok(Box::new(CloverApp::new(
                cc,
                start_path,
                hotkeys_manager,
                hotkey_show,
                hotkey_copy, // 传入定义的 Copy 热键
            )))
        }),
    )
}

pub struct CloverApp {
    data: BusinessData,
    state: ViewState,
    config: Arc<Config>,
    // [修改] 通道现在传递热键 ID (u32)
    hotkey_receiver: mpsc::Receiver<u32>,
    hotkeys_manager: GlobalHotKeyManager, // 去掉下划线，我们需要用它

    // 保存热键定义
    hotkey_show: HotKey,
    hotkey_copy: HotKey,

    // 标记 Ctrl+C 是否已注册
    is_copy_registered: bool,
}

impl CloverApp {
    pub fn new(
        cc: &eframe::CreationContext<'_>,
        start_path: Option<PathBuf>,
        hotkeys_manager: GlobalHotKeyManager,
        hotkey_show: HotKey,
        hotkey_copy: HotKey,
    ) -> Self {
        // 1. 设置字体
        let mut fonts = FontDefinitions::default();
        fonts.font_data.insert("my_font".to_owned(), Arc::new(FontData::from_static(APP_FONT)));
        fonts.families.get_mut(&FontFamily::Proportional).unwrap().insert(0, "my_font".to_owned());
        cc.egui_ctx.set_fonts(fonts);

        // 2. 建立通道并设置“带唤醒功能”的热键回调
        // [修改] 通道传递 u32
        let (tx, rx) = mpsc::channel();
        let ctx_clone = cc.egui_ctx.clone();

        GlobalHotKeyEvent::set_event_handler(Some(Box::new(move |event: GlobalHotKeyEvent| {
            // 发送触发的热键 ID
            let _ = tx.send(event.id);
            ctx_clone.request_repaint();
        })));

        let config = Arc::new(load_config());

        cc.egui_ctx
            .data_mut(|data| data.insert_temp(Id::new("config"),Arc::clone(&config)));

        let mut app = Self {
            data: BusinessData::new(),
            state: ViewState::default(),
            config,
            hotkey_receiver: rx,
            hotkeys_manager,
            hotkey_show,
            hotkey_copy,
            is_copy_registered: false,
        };

        if let Some(path) = start_path {
            app.data.open_new_context(cc.egui_ctx.clone(), path);
        }

        app
    }

    fn handle_background_tasks(&mut self, ctx: &Context) {
        if self.data.process_load_results(ctx) {
            ctx.request_repaint();
        }
        if let Ok(path) = self.state.path_receiver.try_recv() {
            if path.is_dir() {
                self.state.view_mode = ViewMode::Grid;
            } else {
                self.state.view_mode = ViewMode::Single;
            }
            self.data.open_new_context(ctx.clone(), path);
        }
    }

    // [修改] 热键处理逻辑
    fn handle_hotkeys(&mut self, _ctx: &Context) {
        // 1. 动态注册/注销 Ctrl+C
        // 如果进入截图模式且未注册 -> 注册
        if self.state.ui_mode == UiMode::Screenshot && !self.is_copy_registered {
            if let Ok(_) = self.hotkeys_manager.register(self.hotkey_copy) {
                self.is_copy_registered = true;
                println!("[Hotkey] Registered Ctrl+C for Screenshot");
            }
        }
        // 如果退出截图模式且已注册 -> 注销
        else if self.state.ui_mode != UiMode::Screenshot && self.is_copy_registered {
            if let Ok(_) = self.hotkeys_manager.unregister(self.hotkey_copy) {
                self.is_copy_registered = false;
                println!("[Hotkey] Unregistered Ctrl+C");
            }
        }

        // 2. 处理接收到的热键事件
        while let Ok(id) = self.hotkey_receiver.try_recv() {
            if id == self.hotkey_show.id() {
                // Alt + S: 激活截图
                self.state.ui_mode = UiMode::Screenshot;
            } else if id == self.hotkey_copy.id() {
                // Ctrl + C: 只有在截图模式下才会收到这个 (因为只有那时才注册)
                if self.state.ui_mode == UiMode::Screenshot {
                    // 设置标志位，通知 screenshot.rs 执行复制
                    self.state.screenshot_state.copy_requested = true;
                }
            }
        }
    }

    fn handle_input_events(&mut self, ctx: &Context) {
        handle_input_events(ctx, &mut self.data);
    }

    fn draw_ui(&mut self, ctx: &Context) {
        if self.state.ui_mode != UiMode::Screenshot {
            viewer::draw_top_panel(
                ctx,
                &mut self.state,
            );
            viewer::draw_bottom_panel(ctx, &mut self.state);
            viewer::draw_central_panel(ctx, &mut self.data, &mut self.state);
            draw_properties_panel(ctx, &mut self.state, &self.data);
            self.state.toast_system.update(ctx);
        }
    }

    fn handle_ui_interactions(&mut self, ctx: &Context) {
        if self.state.ui_mode != UiMode::Screenshot {
            let mut temp_config = (*self.config).clone();
            let (context_menu_action, modal_action) =
                viewer::draw_overlays(ctx, &self.data, &mut self.state, &mut temp_config);

            if let Some(action) = context_menu_action {
                handle_context_menu_action(ctx, action, &self.data, &mut self.state);
            }

            if let Some(ModalAction::Apply) = modal_action {
                self.config = Arc::new(temp_config);
                save_config(&self.config);
            }
        }
    }
}

impl eframe::App for CloverApp {
    fn update(&mut self, ctx: &Context, _: &mut eframe::Frame) {
        ctx.data_mut(|data| data.insert_temp(Id::NULL, Arc::clone(&self.config)));

        self.handle_hotkeys(ctx); // 传入 ctx
        self.handle_background_tasks(ctx);
        self.handle_input_events(ctx);
        self.draw_ui(ctx);
        self.handle_ui_interactions(ctx);
        handle_screenshot_system(ctx, &mut self.state);
    }
}
