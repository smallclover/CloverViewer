use eframe::egui;
use egui::{
    Context, FontDefinitions,
    FontFamily, TextureHandle,
    ViewportBuilder,FontData,
    Key
};
use std::{
    num::NonZeroUsize,
    path::PathBuf,
    sync::Arc,
    collections::HashSet,
};
use lru::LruCache;
use crate::{
    image_loader::{ImageLoader, LoadResult},
    navigator::Navigator,
    ui::{
        menu::draw_menu,
        preview::draw_preview_bar,
        loading::global_loading,
        resources::APP_FONT,
        viewer::{draw_viewer, ViewerState, ViewerAction},
        settings::render_settings_window,
        about::render_about_window,
        right_click_menu::render_context_menu,
        ui_mode::UiMode
    },
    utils::{is_image, load_icon},
    config::{load_config, save_config, Config}
};

pub fn run() -> eframe::Result<()> {
    let mut options = eframe::NativeOptions {
        viewport: ViewportBuilder::default().with_inner_size([1024.0, 768.0]),
        ..Default::default()
    };
    // 加载图标
    options.viewport = options.viewport.with_icon(load_icon());

    let start_path = std::env::args().nth(1).map(PathBuf::from);

    eframe::run_native(
        "CloverViewer",
        options,
        Box::new(|cc| {
            Ok(Box::new(MyApp::new(cc, start_path)))
        }),
    )
}


pub struct MyApp {
    loader: ImageLoader,
    nav: Navigator,
    // LRU 缓存：Key 是路径，Value 是纹理句柄
    // 当超过容量时，最久没看的 TextureHandle 会被自动丢弃，
    // 因为 TextureHandle 内部是 Arc，丢弃后 egui 会自动回收显存。
    texture_cache: LruCache<PathBuf, TextureHandle>,      // 原图缓存
    thumb_cache: LruCache<PathBuf, TextureHandle>,      // 缩略图缓存
    current_texture: Option<TextureHandle>,
    error: Option<String>, // 用于存储当前图片的错误
    zoom: f32,
    ui_mode: UiMode, // UI 状态机
    failed_thumbs: HashSet<PathBuf>, // 记录加载失败的预览图路径
    config: Config, // 配置
}

impl MyApp {
    pub fn new(cc: &eframe::CreationContext<'_>, start_path: Option<PathBuf>, ) -> Self {
        // --- 1. 字体配置 ---
        let mut fonts = FontDefinitions::default();
        fonts.font_data.insert(
            "my_font".to_owned(),
            Arc::new(FontData::from_static(APP_FONT)),
        );
        fonts.families.get_mut(&FontFamily::Proportional).unwrap().insert(
            0, "my_font".to_owned()
        );
        cc.egui_ctx.set_fonts(fonts);

        // 加载配置
        let config = load_config();

        let mut app = Self {
            loader: ImageLoader::new(),
            nav: Navigator::new(),
            // 限制缓存数量为 10 张（25MB * 10 约 250MB 显存，可以根据内存情况调整）
            texture_cache: LruCache::new(NonZeroUsize::new(10).unwrap()),
            // 缩略图缓存：建议 100 张
            // 100 * 75KB ≈ 7.5MB，内存压力极小，但体验极大提升
            thumb_cache: LruCache::new(NonZeroUsize::new(100).unwrap()),
            current_texture: None,
            error: None,
            zoom: 1.0,
            ui_mode: UiMode::Normal,
            failed_thumbs: HashSet::new(),
            config,
        };

        if let Some(path) = start_path {
            app.open_new_context(cc.egui_ctx.clone(), path);
        }

        app
    }

    /// 从外部打开（拖拽、菜单、启动参数）—— 需要重扫文件夹
    fn open_new_context(&mut self, ctx: Context, path: PathBuf) {
        // 1. 更新导航器状态（扫描文件夹等）
        if path.is_dir() {
            self.nav.f_folder(&path);
        } else {
            // 这里内部会扫描父目录并建立图片列表
            self.nav.f_image(&path);
        }

        // 2. 调用统一的加载逻辑
        self.load_current(ctx);
    }

    /// 处理异步加载结果 (数据层)
    /// 核心：处理加载结果
    fn process_load_results(&mut self, ctx: &Context) {
        let mut processed_count = 0;
        let mut should_trigger_preloads = false;
        let mut received_any = false;

        // 1. 限制每帧处理消息的数量（例如最多 5 条），防止大批量加载时 UI 线程卡死
        while processed_count < 5 {
            match self.loader.rx.try_recv() {
                Ok(msg) => {
                    received_any = true;
                    match msg.result {
                        LoadResult::Ok(tex) => {
                            if msg.is_thumbnail {
                                self.thumb_cache.put(msg.path.clone(), tex.clone());
                                if Some(&msg.path) == self.nav.current().as_ref() {
                                    if self.current_texture.is_none() {
                                        self.current_texture = Some(tex);
                                    }
                                }
                            } else {
                                self.texture_cache.put(msg.path.clone(), tex.clone());
                                if Some(msg.path) == self.nav.current() {
                                    // 缩放计算逻辑 (计算开销极小，可以保留)
                                    let tex_size = tex.size_vec2();
                                    let available = ctx.available_rect().size();
                                    let scale_v = available.y / tex_size.y;
                                    let scale_h = (available.x - 120.0) / tex_size.x;
                                    self.zoom = scale_v.min(scale_h).min(1.0);

                                    self.current_texture = Some(tex);
                                    self.loader.is_loading = false;

                                    // 标记需要预加载，不在循环内立即执行
                                    should_trigger_preloads = true;
                                }
                            }
                        }
                        LoadResult::Err(e) => {
                            // 1：无论是不是原图，只要报错，就记录到黑名单
                            self.failed_thumbs.insert(msg.path.clone());
                            if msg.is_priority {
                                self.loader.is_loading = false;
                                self.error = Some(e);
                            }
                        }
                    }
                    processed_count += 1;
                }
                Err(_) => break, // 队列空了
            }
        }

        // 2. 统一触发副作用
        if should_trigger_preloads {
            self.trigger_preloads(ctx);
        }

        // 3. 只要有数据变动，请求一次重绘即可
        if received_any {
            ctx.request_repaint();
        }
    }

    /// 预加载逻辑：不阻塞主流程
    fn trigger_preloads(&mut self, ctx: &Context) {
        // 获取预览窗口的所有路径
        let to_load = self.nav.get_preview_window();
        // 只有缓存里没有，才发起低优先级加载
        for (_, path) in to_load {
            // 检查专门的缩略图缓存 (thumb_cache)
            if !self.thumb_cache.contains(&path) {
                // size = Some((160, 120)): 告诉 loader 只解码成小图
                self.loader.load_async(ctx.clone(), path, false, Some((160, 120)));
            }
        }
    }

    /// 核心导航逻辑：加载高清原图
    fn load_current(&mut self, ctx: Context) {
        self.error = None; // 切换图时重置错误状态

        if let Some(path) = self.nav.current() {
            // --- 无论如何，先让预览栏动起来 ---
            self.trigger_preloads(&ctx);
            // 1. 尝试从原图缓存 (texture_cache) 获取
            if let Some(tex) = self.texture_cache.get(&path) {
                self.current_texture = Some(tex.clone());
                self.loader.is_loading = false;
            } else {
                if self.failed_thumbs.contains(&path) {
                    // 这里不直接赋值字符串，而是留空，让 viewer 根据语言动态获取
                    self.error = Some("error".to_string());
                    self.current_texture = None;
                    self.loader.is_loading = false;
                }else{
                    self.current_texture = self.thumb_cache.get(&path).cloned();
                    // 3. 发起高优先级异步加载 (size 为 None 表示加载原图)
                    self.loader.load_async(ctx, path, true, None);
                }
            }
        }
    }

    /// 输入处理
    fn handler_inputs(&mut self, ctx: &Context) {
        ctx.input(|i| {
            if i.key_pressed(Key::ArrowLeft) {
                self.prev_image(ctx.clone());
            }
            if i.key_pressed(Key::ArrowRight) {
                self.next_image(ctx.clone());
            }
        });

        if let Some(path) = ctx.input(|i| {
            i.raw.dropped_files.first()
                .and_then(|f| f.path.clone())
                .filter(|p| is_image(p))
        }){
            self.open_new_context(ctx.clone(), path);
        }

        let scroll_delta = ctx.input(|i| i.smooth_scroll_delta.y);
        if scroll_delta != 0.0 {
            self.zoom = (self.zoom + scroll_delta * 0.001).clamp(0.1, 10.0);
        }
    }

    // 封装上一张图片的逻辑
    fn prev_image(&mut self, ctx: Context) {
        if self.nav.prev().is_some() {
            self.load_current(ctx);
        }
    }

    // 封装下一张图片的逻辑
    fn next_image(&mut self, ctx: Context) {
        if self.nav.next().is_some() {
            self.load_current(ctx);
        }
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &Context, _: &mut eframe::Frame) {
        // 1. 处理异步加载结果 (数据层)
        self.process_load_results(ctx);
        // 2. 处理全局输入 (控制层：键盘、拖拽、滚轮)
        self.handler_inputs(ctx);
        // 3. 渲染菜单栏 (UI层：顶部)
        if let Some(path) = draw_menu(ctx, &mut self.ui_mode, self.config.language) {
            self.open_new_context(ctx.clone(), path);
        }
        // 4. 渲染主画布
        let viewer_state = ViewerState {
            texture: self.current_texture.as_ref(),
            is_loading: self.loader.is_loading,
            error: self.error.as_ref(),
            zoom: self.zoom,
            has_nav: self.nav.current().is_some(),
        };

        // 检查是否处于模态窗口状态
        let is_modal_open = matches!(self.ui_mode, UiMode::About | UiMode::Settings);

        match draw_viewer(ctx, viewer_state, is_modal_open, self.config.language) {
            ViewerAction::Prev => {
                self.prev_image(ctx.clone());
            }
            ViewerAction::Next => {
                self.next_image(ctx.clone());
            }
            ViewerAction::ContextMenu(pos) => {
                self.ui_mode = UiMode::ContextMenu(pos);
            }
            ViewerAction::None => {}
        }

        // 5. 预览窗口
        if !is_modal_open && self.nav.current().is_some() {
            let previews = self.nav.get_preview_window();
            if let Some(new_idx) = draw_preview_bar(
                ctx,
                &previews,
                &mut self.thumb_cache,
                &self.failed_thumbs,
                self.nav.get_index()
            ) {
                self.nav.set_index(new_idx);
                self.load_current(ctx.clone());
            }
        }

        // 6. 渲染弹窗 (独立层)
        match self.ui_mode {
            UiMode::About => {
                let mut open = true;
                render_about_window(ctx, &mut open, self.config.language);
                if !open {
                    self.ui_mode = UiMode::Normal;
                }
            }
            UiMode::Settings => {
                let mut open = true;
                let old_lang = self.config.language;
                render_settings_window(ctx, &mut open, &mut self.config.language);

                if old_lang != self.config.language {
                    save_config(&self.config);
                }

                if !open {
                    save_config(&self.config);
                    self.ui_mode = UiMode::Normal;
                }
            }
            UiMode::ContextMenu(pos) => {
                let mut pos_opt = Some(pos);
                render_context_menu(ctx, &mut pos_opt, self.config.language);
                if let Some(new_pos) = pos_opt {
                    // 如果位置变了（通常不会），更新它
                    self.ui_mode = UiMode::ContextMenu(new_pos);
                } else {
                    // 菜单关闭
                    self.ui_mode = UiMode::Normal;
                }
            }
            UiMode::Normal => {}
        }

        // 全局状态
        if self.current_texture.is_none() && self.loader.is_loading {
            global_loading(ctx, self.config.language);
        }
    }
}
