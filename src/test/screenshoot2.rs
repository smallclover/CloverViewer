use eframe::egui;
use egui::{FontData, FontDefinitions, FontFamily, TextureHandle};
use std::sync::Arc;
use xcap::Monitor;

fn main() -> eframe::Result<()> {
    let monitors = Monitor::all().unwrap_or_else(|err| {
        eprintln!("获取显示器信息失败: {}", err);
        vec![]
    });

    if monitors.is_empty() {
        panic!("没有检测到显示器！");
    }

    // 1. 物理边界计算
    let mut phys_min_x = i32::MAX;
    let mut phys_min_y = i32::MAX;
    let mut phys_max_x = i32::MIN;
    let mut phys_max_y = i32::MIN;

    for m in &monitors {
        let x = m.x().unwrap_or(0);
        let y = m.y().unwrap_or(0);
        let w = m.width().unwrap_or(800) as i32;
        let h = m.height().unwrap_or(600) as i32;

        if x < phys_min_x { phys_min_x = x; }
        if y < phys_min_y { phys_min_y = y; }
        if x + w > phys_max_x { phys_max_x = x + w; }
        if y + h > phys_max_y { phys_max_y = y + h; }
    }

    // 默认缩放为1.0也就是不缩放
    let startup_scale = 1.0;

    // 启动时申请的逻辑尺寸（用物理尺寸除以启动缩放率）
    let logic_total_w = (phys_max_x - phys_min_x) as f32 / startup_scale;
    let logic_total_h = (phys_max_y - phys_min_y) as f32 / startup_scale;

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("CloverViewer 大画布")
            .with_position(egui::pos2(phys_min_x as f32 / startup_scale, phys_min_y as f32 / startup_scale))
            // 确保窗口能撑开
            .with_min_inner_size(egui::vec2(logic_total_w, logic_total_h))
            .with_decorations(false)
            .with_transparent(true),
        ..Default::default()
    };

    eframe::run_native(
        "CloverViewer Canvas",
        options,
        Box::new(move |cc| {
            Ok(Box::new(SingleWindowApp::new(
                monitors,
                phys_min_x,
                phys_min_y,
                cc
            )))
        }),
    )
}

struct SingleWindowApp {
    monitors: Vec<Monitor>,
    phys_min_x: i32,
    phys_min_y: i32,
    // 新增：存储各个显示器截图的纹理句柄
    textures: Vec<Option<TextureHandle>>,
}

impl SingleWindowApp {
    pub const APP_FONT: &[u8] = include_bytes!(
        concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/assets/fonts/msyhl.ttf"
        )
    );

    fn new(
        monitors: Vec<Monitor>,
        phys_min_x: i32,
        phys_min_y: i32,
        cc: &eframe::CreationContext<'_>
    ) -> Self {
        let mut fonts = FontDefinitions::default();
        fonts.font_data.insert("my_font".to_owned(), Arc::new(FontData::from_static(Self::APP_FONT)));
        fonts.families.get_mut(&FontFamily::Proportional).unwrap().insert(0, "my_font".to_owned());
        cc.egui_ctx.set_fonts(fonts);

        // --- 新增：在程序启动时抓取屏幕并生成纹理 ---
        let mut textures = Vec::new();
        println!("正在捕获屏幕，请稍候...");
        for (i, m) in monitors.iter().enumerate() {
            if let Ok(image_buffer) = m.capture_image() {
                let width = image_buffer.width() as usize;
                let height = image_buffer.height() as usize;
                // 获取 RGBA 裸数据
                let pixels = image_buffer.into_raw();

                // 将原始像素转为 egui 可识别的颜色图像
                let color_image = egui::ColorImage::from_rgba_unmultiplied(
                    [width, height],
                    &pixels,
                );

                // 将图像加载到 egui 上下文中（也就是送入 GPU 显存）
                let texture = cc.egui_ctx.load_texture(
                    format!("screen_capture_{}", i),
                    color_image,
                    egui::TextureOptions::LINEAR,
                );
                textures.push(Some(texture));
            } else {
                eprintln!("警告：捕获显示器 {} 失败", i);
                textures.push(None);
            }
        }
        println!("捕获完成！");

        Self {
            monitors,
            phys_min_x,
            phys_min_y,
            textures,
        }
    }
}

impl eframe::App for SingleWindowApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // 获取 egui 这一帧实际使用的全局缩放率
        let egui_global_scale = ctx.pixels_per_point();

        egui::CentralPanel::default()
            .frame(egui::Frame::none().fill(egui::Color32::TRANSPARENT))
            .show(ctx, |ui| {
                let painter = ui.painter();

                for (i, m) in self.monitors.iter().enumerate() {
                    let m_x = m.x().unwrap_or(0);
                    let m_y = m.y().unwrap_or(0);
                    let m_w = m.width().unwrap_or(800) as f32;
                    let m_h = m.height().unwrap_or(600) as f32;

                    let phys_rel_x = (m_x - self.phys_min_x) as f32;
                    let phys_rel_y = (m_y - self.phys_min_y) as f32;

                    // 将物理坐标按比例“缩回”逻辑坐标，抵消系统的强制缩放
                    let logic_x = phys_rel_x / egui_global_scale;
                    let logic_y = phys_rel_y / egui_global_scale;
                    let logic_w = m_w / egui_global_scale;
                    let logic_h = m_h / egui_global_scale;

                    let rect = egui::Rect::from_min_size(
                        egui::pos2(logic_x, logic_y),
                        egui::vec2(logic_w, logic_h),
                    );

                    // --- 新增：绘制截图纹理 ---
                    if let Some(texture) = &self.textures[i] {
                        // 使用 painter.image 画出这张纹理
                        // texture.id() 获取纹理ID
                        // rect 是目标绘制区域 (我们算好的完美覆盖区域)
                        // uv 是纹理采样坐标，(0.0, 0.0) 到 (1.0, 1.0) 代表取整张图片
                        // Color32::WHITE 相当于不加任何滤镜颜色
                        painter.image(
                            texture.id(),
                            rect,
                            egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
                            egui::Color32::WHITE,
                        );
                    } else {
                        // 如果因为某种原因截图失败了，降级画一个红色警告块
                        painter.rect_filled(rect, 0.0, egui::Color32::from_rgba_premultiplied(200, 50, 50, 240));
                    }
                }
            });

        // 依然保留这个主控制台面板，你可以拖着它在两个屏幕里测试流畅度
        egui::Window::new("主控制台")
            .default_pos(egui::pos2(50.0, 50.0))
            .show(ctx, |ui| {
                ui.heading("屏幕冻结完成！");
                ui.label("现在的背景应该是你的真实桌面截图。");
                ui.label("尝试拖动这个窗口，感受大画布模式下的丝滑跨屏。");
                if ui.button("退出").clicked() {
                    ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                }
            });
    }
}