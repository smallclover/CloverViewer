use std::num::NonZeroUsize;
use eframe::egui;
use egui::{CentralPanel, Context, FontDefinitions, FontFamily, Image, ScrollArea, TextureHandle};
use std::path::PathBuf;
use std::sync::Arc;
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
        loading::loading
    },
    utils::is_image
};

pub fn run() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([1024.0, 768.0]),
        ..Default::default()
    };

    let start_path = std::env::args().nth(1).map(PathBuf::from);

    eframe::run_native(
        "Light Image Viewer",
        options,
        Box::new(|cc| {
            // // 关键：安装图片加载器，这样 egui::Image 才能理解各种格式
            // egui_extras::install_image_loaders(&cc.egui_ctx);
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
    error: Option<String>, // 新增：用于存储当前图片的错误
    zoom: f32,
    show_about: bool,// 关于菜单 状态
}

impl MyApp {
    pub fn new(cc: &eframe::CreationContext<'_>, start_path: Option<PathBuf>, ) -> Self {
        // --- 1. 字体配置 ---
        let mut fonts = FontDefinitions::default();
        fonts.font_data.insert(
            "my_font".to_owned(),
            Arc::new( egui::FontData::from_static(include_bytes!("../assets/msyhl.ttf"))),
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
        while let Ok(msg) = self.loader.rx.try_recv() {
            match msg.result {
                LoadResult::Ok(tex) => {
                    if msg.is_thumbnail {
                        // 缩略图逻辑：仅存入缓存，供底部预览栏使用
                        self.thumb_cache.put(msg.path.clone(), tex.clone());
                        // 2. 如果这张缩略图正是用户当前要看的图，且原图还没出来
                        // 那么立即将其设为 current_texture 占位
                        if Some(&msg.path) == self.nav.current().as_ref() {
                            if self.current_texture.is_none() {
                                self.current_texture = Some(tex);
                            }
                        }
                        ctx.request_repaint();
                    }else{
                        // 原图逻辑：执行缓存更新与缩放计算
                        self.texture_cache.put(msg.path.clone(), tex.clone());
                        if Some(msg.path) == self.nav.current() {
                            // --- 核心逻辑：计算自适应缩放 ---
                            let tex_size = tex.size_vec2();
                            // 获取当前窗口除菜单栏外的可用空间
                            let available = ctx.available_rect().size();

                            // 方案：
                            // 1. 竖向顶满所需的缩放：available.y / tex_height
                            // 2. 横向留白所需的缩放：(available.x - 120.0) / tex_width (120是两侧留白总和)
                            let scale_v = available.y / tex_size.y;
                            let scale_h = (available.x - 120.0) / tex_size.x;

                            // 取两者中较小的一个，确保图片完整显示且满足你的留白需求
                            // 同时使用 .min(1.0) 防止小图片被强制放大导致模糊
                            self.zoom = scale_v.min(scale_h).min(1.0);

                            self.current_texture = Some(tex);
                            self.loader.is_loading = false;
                            // 原图加载完成后，触发周边图片的缩略图预加载
                            self.trigger_preloads(ctx);
                        }
                    }

                }
                LoadResult::Err(e) => {
                    // 只有在加载主图（高优先级）失败时才抛出错误提示
                    if msg.is_priority {
                        self.loader.is_loading = false;
                        self.error = Some(e);
                    }
                }
            }
        }
    }

    /// 预加载逻辑：不阻塞主流程
    fn trigger_preloads(&mut self, ctx: &egui::Context) {
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
    fn load_current(&mut self, ctx: egui::Context) {
        self.error = None; // 切换图时重置错误状态

        if let Some(path) = self.nav.current() {
            // 1. 尝试从原图缓存 (texture_cache) 获取
            if let Some(tex) = self.texture_cache.get(&path) {
                self.current_texture = Some(tex.clone());
                self.loader.is_loading = false;
                // 命中原图后，也触发一次预加载（为了更新预览栏邻居的缩略图）
                self.trigger_preloads(&ctx);
            } else {
                // 2. 【优化体验】原图没中，先检查缩略图缓存中是否已有该图
                // 如果预览栏已经加载了这张图的缩略图，我们先把它显示出来占位
                if let Some(thumb) = self.thumb_cache.get(&path) {
                    self.current_texture = Some(thumb.clone());
                } else {
                    self.current_texture = None;
                }
                // 3. 发起高优先级异步加载 (size 为 None 表示加载原图)
                self.loader.load_async(ctx, path, true, None);
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
            self.render_image_viewer(ui, &tex, rect);

            // 如果正在加载（说明现在显示的是缩略图，高清图还在路上），在右上角画个小菊花
            if self.loader.is_loading {
                loading(ui);
            }
        }
        // 情况 2：没有任何纹理可以显示，且正在加载，此时才显示中心大菊花
        else if self.loader.is_loading {
            loading(ui);
        }
        // 情况 3：加载失败
        else if let Some(err) = &self.error {
            ui.centered_and_justified(|ui| ui.colored_label(egui::Color32::RED, err));
        }
        // 情况 4：空状态
        else {
            ui.centered_and_justified(|ui| ui.label("拖拽或打开图片"));
        }
    }

    fn render_image_viewer(&mut self,
                           ui: &mut egui::Ui,
                           tex: &egui::TextureHandle,
                           rect: egui::Rect){

        ScrollArea::both()
            .scroll_source(egui::scroll_area::ScrollSource::DRAG)
            .auto_shrink([false; 2])
            .show(ui, |ui| {

                // --- 手动计算居中逻辑 ---
                let size = tex.size_vec2() * self.zoom;

                // 获取当前 ScrollArea 内部可用的视口大小
                let available_size = ui.available_size();

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

        if let Some(nav) = draw_arrows(ui, rect) {
            match nav {
                Nav::Prev => { self.nav.prev(); }
                Nav::Next => { self.nav.next(); }
            };
            // 统一调用加载方法
            self.load_current(ui.ctx().clone());
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
    }
}
