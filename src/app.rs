use eframe::egui;
use egui::{
    CentralPanel, Context, FontDefinitions,
    FontFamily, Image, ScrollArea, TextureHandle,
    ViewportBuilder,FontData
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
        arrows::Nav,
        arrows::draw_arrows,
        menu::draw_menu,
        menu::render_about_window,
        preview::draw_preview_bar,
        loading::{corner_loading, global_loading},
        resources::APP_FONT
    },
    utils::{is_image, load_icon}
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
    show_about: bool,// 关于菜单 状态
    failed_thumbs: HashSet<PathBuf>, // 记录加载失败的预览图路径
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
            show_about: false, // 默认不显示
            failed_thumbs: HashSet::new(),
        };

        if let Some(path) = start_path {
            app.open_new_context(cc.egui_ctx.clone(), path);
        }

        app
    }

    /// 场景 A：从外部打开（拖拽、菜单、启动参数）—— 需要重扫文件夹
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
                                //2：主图报错也要触发预加载，否则预览栏永远是空的
                                // should_trigger_preloads = true;
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
                // is_priority = false: 预加载任务
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
                    self.error = Some("文件损坏或格式不支持".to_string());
                    self.current_texture = None;
                    self.loader.is_loading = false;
                }else{
                    // 2. 原图没中，先检查缩略图缓存中是否已有该图
                    // // 如果预览栏已经加载了这张图的缩略图，我们先把它显示出来占位
                    // if let Some(thumb) = self.thumb_cache.get(&path) {
                    //     self.current_texture = Some(thumb.clone());
                    // } else {
                    //     self.current_texture = None;
                    // }
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
            if i.key_pressed(egui::Key::ArrowLeft) {
                if self.nav.prev().is_some() {
                    self.load_current(ctx.clone());
                }
            }
            if i.key_pressed(egui::Key::ArrowRight) {
                if self.nav.next().is_some() {
                    self.load_current(ctx.clone());
                }
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
    /// 渲染主画布
    fn render_central_panel(&mut self, ctx: &Context) {
        // 创建深色背景，添加内边距 (margin)
        let background_frame = egui::Frame::NONE
            .fill(egui::Color32::from_rgb(25, 25, 25)) // 深炭灰色，能很好地衬托图片
            .inner_margin(0.0); // 如果希望图片贴边，设为 0

        CentralPanel::default().frame(background_frame).show(ctx, |ui| {
            ui.add_enabled_ui(!self.show_about, |ui|{
                self.render_main_content(ui);
            })
        });
    }

    fn render_main_content(&mut self, ui: &mut egui::Ui) {
        let rect = ui.available_rect_before_wrap();

        // 情况 1：只要当前有纹理（无论是原图还是缩略图），就先画出来
        if let Some(tex) = self.current_texture.clone() {
            self.render_image_viewer(ui, &tex);

            // 如果正在加载（说明现在显示的是缩略图，高清图还在路上），在右上角画个小菊花
            if self.loader.is_loading {
                corner_loading(ui);
            }
        }
        // 情况 2：加载失败
        else if let Some(_) = &self.error {
            // 即使主图报错，这里的渲染也要确保不阻塞后面的 UI
            ui.scope_builder(egui::UiBuilder::new().max_rect(rect),|ui| {
                ui.vertical_centered(|ui| {
                    ui.add_space(ui.available_height() * 0.4); // 稍微向下偏移，视觉更舒服
                    ui.label(egui::RichText::new("文件损坏或格式不支持")
                        .color(egui::Color32::RED)
                        .size(14.0));
                });
            });

        }else if self.loader.is_loading {
            // 3. 如果正在加载且没图没报错，这里留白
            // 这样背后的文字就不会出来了，全局菊花会覆盖在这个空白区域上
        }
        // 情况 4：空状态
        else {
            ui.centered_and_justified(|ui| ui.label("拖拽或打开图片"));
        }

        // --- 2. 渲染导航箭头（无论成功与否，只要列表不为空就显示） ---
        // 只要导航器里有图片列表，我们就应该允许翻页
        if self.nav.current().is_some() {
            if let Some(nav_action) = draw_arrows(ui, rect) {
                match nav_action {
                    Nav::Prev => { self.nav.prev(); }
                    Nav::Next => { self.nav.next(); }
                };
                self.load_current(ui.ctx().clone());
            }
        }
    }

    fn render_image_viewer(&mut self,
                           ui: &mut egui::Ui,
                           tex: &TextureHandle){
        //--- 手动计算居中逻辑 ---
        let size = tex.size_vec2() * self.zoom;
        // 获取当前 ScrollArea 内部可用的视口大小
        let available_size = ui.available_size();

        // 1. 判断当前图片是否大于显示区域（只要有一边大，就可以拖拽）
        let is_draggable = size.x > available_size.x || size.y > available_size.y;

        // 2. 如果可以拖拽，且鼠标正在此区域内，改变指针
        if is_draggable {
            // 3. 在闭包外部，根据状态设置图标
            if ui.rect_contains_pointer(ui.max_rect()) {
                ui.ctx().set_cursor_icon(egui::CursorIcon::Move );
            }
        }

        ScrollArea::both()
            .scroll_source(egui::scroll_area::ScrollSource::DRAG)
            .auto_shrink([false; 2])
            .show(ui, |ui| {
                // 计算边距：如果图片比窗口小，则计算一半的差值作为偏移；否则偏移为 0
                let x_offset = (available_size.x - size.x).max(0.0) * 0.5;
                let y_offset = (available_size.y - size.y).max(0.0) * 0.5;

                ui.horizontal(|ui| {
                    ui.add_space(x_offset); // 左边距
                    ui.vertical(|ui| {
                        ui.add_space(y_offset); // 上边距

                        let img_widget = Image::from_texture(tex)
                            .fit_to_exact_size(size);

                        ui.add(img_widget);
                    });
                });
            });
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &Context, _: &mut eframe::Frame) {
        // 1. 处理异步加载结果 (数据层)
        self.process_load_results(ctx);
        // 2. 处理全局输入 (控制层：键盘、拖拽、滚轮)
        self.handler_inputs(ctx);
        // 3. 渲染菜单栏 (UI层：顶部)
        if let Some(path) = draw_menu(ctx, &mut self.show_about) {
            self.open_new_context(ctx.clone(), path);
        }
        // 4. 渲染主画布
        self.render_central_panel(ctx);
        // 5. 预览窗口
        if !self.show_about && self.nav.current().is_some() {
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
        if self.show_about {
            // 这里调用弹窗逻辑
            render_about_window(ctx, &mut self.show_about);
        }

        // 全局状态
        if self.current_texture.is_none() && self.loader.is_loading {
            global_loading(ctx);
        }
    }
}
