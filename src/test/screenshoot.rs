use eframe::egui;
use egui::{FontData, FontDefinitions, FontFamily, TextureHandle};
use std::sync::Arc;
use xcap::Monitor;
/// 截图
fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("主控制台 (App Window)")
            .with_inner_size([400.0, 300.0])
            .with_position(egui::pos2(100.0, 100.0))
            .with_always_on_top(),
        ..Default::default()
    };

    let monitors = Monitor::all().unwrap_or_else(|err| {
        eprintln!("获取显示器信息失败: {}", err);
        vec![]
    });

    eframe::run_native(
        "Xcap Capture App",
        options,
        Box::new(move |cc| Ok(Box::new(MultiWindowApp::new(monitors, cc)))),
    )
}

struct MultiWindowApp {
    monitors: Vec<Monitor>,
    show_viewports: bool,
    // 新增：用于存储每个显示器捕获到的图像纹理
    captured_textures: Vec<Option<TextureHandle>>,
}

impl MultiWindowApp {
    pub const APP_FONT: &[u8] = include_bytes!(
        concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/assets/fonts/msyhl.ttf"
        )
    );

    fn new(monitors: Vec<Monitor>, cc: &eframe::CreationContext<'_>) -> Self {
        let mut fonts = FontDefinitions::default();
        fonts.font_data.insert("my_font".to_owned(), Arc::new(FontData::from_static(Self::APP_FONT)));
        fonts.families.get_mut(&FontFamily::Proportional).unwrap().insert(0, "my_font".to_owned());
        cc.egui_ctx.set_fonts(fonts);

        // --- 核心逻辑：在启动时捕获屏幕并转换为 egui 纹理 ---
        let mut captured_textures = Vec::new();

        for (i, monitor) in monitors.iter().enumerate() {
            if i >= 2 { break; } // 这里我们还是只处理前两个显示器

            println!("正在截取显示器 {}...", i + 1);
            // 1. xcap 捕获图像
            if let Ok(image_buffer) = monitor.capture_image() {
                let width = image_buffer.width() as usize;
                let height = image_buffer.height() as usize;
                // 获取裸像素数据 (RGBA 格式)
                let pixels = image_buffer.into_raw();

                // 2. 转换为 egui 的 ColorImage
                let color_image = egui::ColorImage::from_rgba_unmultiplied(
                    [width, height],
                    &pixels,
                );

                // 3. 加载到 egui 上下文中，生成 TextureHandle
                let texture = cc.egui_ctx.load_texture(
                    format!("screen_capture_{}", i),
                    color_image,
                    egui::TextureOptions::LINEAR, // 线性采样，图片缩放时更平滑
                );

                captured_textures.push(Some(texture));
            } else {
                eprintln!("捕获显示器 {} 失败", i + 1);
                captured_textures.push(None);
            }
        }
        println!("截图完成！");

        Self {
            monitors,
            show_viewports: true,
            captured_textures,
        }
    }

    fn hidden_viewports(&mut self, ctx: &egui::Context) {
        // ==========================================
        // 窗口 1: App 窗口 (主控制台)
        // ==========================================
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("主控制窗口");
            ui.separator();

            ui.horizontal(|ui| {
                let button_text = if self.show_viewports { "隐藏截图窗口" } else { "显示截图窗口" };
                if ui.button(button_text).clicked() {
                    self.show_viewports = !self.show_viewports;
                }
                ui.label(if self.show_viewports { "🟢 正在显示" } else { "🟡 已隐藏 (后台保活)" });
            });

            ui.separator();
            ui.label(format!("已捕获 {} 个显示器的画面", self.captured_textures.len()));
        });

        // ==========================================
        // 窗口 2 & 3: 渲染截图的 Viewport
        // ==========================================
        for i in 0..2 {
            if let Some(monitor) = self.monitors.get(i) {
                let viewport_id = egui::ViewportId::from_hash_of(format!("screen_overlay_{}", i));

                let phys_x = monitor.x().unwrap() as f32;
                let phys_y = monitor.y().unwrap() as f32;
                let self_scale = monitor.scale_factor().unwrap();
                let logic_x = phys_x / self_scale;
                let logic_y = phys_y / self_scale;

                let pos = egui::pos2(logic_x, logic_y);
                let size = egui::vec2(
                    monitor.width().unwrap() as f32 / self_scale,
                    monitor.height().unwrap() as f32 / self_scale
                );

                let builder = egui::ViewportBuilder::default()
                    .with_title(format!("Viewport {}", i + 1))
                    .with_position(pos)
                    .with_inner_size(size)
                    .with_decorations(false)
                    .with_visible(self.show_viewports);

                // 【关键改动 1】：获取当前显示器的纹理句柄，并 Clone 出来
                // TextureHandle 内部是 Arc 包裹的，Clone 只是增加引用计数，不会复制图像内存
                let texture_opt = self.captured_textures.get(i).cloned().flatten();

                ctx.show_viewport_deferred(viewport_id, builder, move |ctx, _class| {
                    // 使用 Frame::none() 去除默认的内边距(Padding)和背景色
                    // 这样图片就能 100% 无缝填满整个无边框窗口
                    egui::CentralPanel::default().frame(egui::Frame::none()).show(ctx, |ui| {

                        if let Some(texture) = &texture_opt {
                            // 【关键改动 2】：将图片绘制在 UI 上，并强制缩放至与窗口一致大小
                            let image = egui::Image::new(texture)
                                .fit_to_exact_size(ui.available_size());
                            ui.add(image);
                        } else {
                            // 如果截图失败，显示默认黑屏文字
                            ui.centered_and_justified(|ui| {
                                ui.heading("无法获取该屏幕的截图");
                            });
                        }

                    });
                });
            }
        }
    }
}

impl eframe::App for MultiWindowApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.hidden_viewports(ctx);
    }
}