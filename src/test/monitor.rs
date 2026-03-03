use eframe::egui;
use egui::{FontData, FontDefinitions, FontFamily};
use xcap::Monitor;
use std::sync::Arc;

/// 单纯测试初始化两个窗口
fn main() -> eframe::Result<()> {
    // 1. 初始化主窗口 (App 窗口)
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("主控制台 (App Window)")
            .with_inner_size([400.0, 300.0])
            .with_position(egui::pos2(100.0, 100.0)), // 给主窗口一个初始位置避免被遮挡
        ..Default::default()
    };



    // 在启动 egui 之前，先用 xcap 获取所有显示器的信息
    // 这样做可以避免在每一帧 (update) 里高频调用底层 API
    let monitors = Monitor::all().unwrap_or_else(|err| {
        eprintln!("获取显示器信息失败: {}", err);
        vec![]
    });

    eframe::run_native(
        "Xcap Egui App",
        options,
        Box::new( move |cc| Ok(Box::new(MultiWindowApp::new(monitors, cc)))),
    )
}

struct MultiWindowApp {
    monitors: Vec<Monitor>,
}

impl MultiWindowApp {
    pub const APP_FONT: &[u8] = include_bytes!(
        concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/assets/fonts/msyhl.ttf"
        )
    );

    fn new(monitors: Vec<Monitor>, cc: &eframe::CreationContext<'_>,) -> Self {
        // 1. 设置字体
        let mut fonts = FontDefinitions::default();
        fonts.font_data.insert("my_font".to_owned(), Arc::new(FontData::from_static(Self::APP_FONT)));
        fonts.families.get_mut(&FontFamily::Proportional).unwrap().insert(0, "my_font".to_owned());
        cc.egui_ctx.set_fonts(fonts);
        Self { monitors }
    }


}

impl eframe::App for MultiWindowApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // ==========================================
        // 窗口 1: App 窗口 (主控制台)
        // ==========================================
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("主控制窗口");
            ui.separator();
            ui.label(format!("xcap 检测到了 {} 个显示器:", self.monitors.len()));

            for (i, m) in self.monitors.iter().enumerate() {
                ui.label(format!(
                    "显示器 {}: {}x{} 位置: ({}, {})",
                    i + 1, m.width().unwrap(), m.height().unwrap(), m.x().unwrap(), m.y().unwrap()
                ));
            }
        });

        // ==========================================
        // 窗口 2 & 3: 两个覆盖屏幕的 Viewport
        // ==========================================
        // 定义两个不同的背景色，为了不刺眼，这里用了稍暗的颜色
        let bg_colors = [
            egui::Color32::from_rgb(40, 60, 40), // 墨绿色 (显示器 1)
            egui::Color32::from_rgb(40, 40, 60), // 藏青色 (显示器 2)
        ];

        // 遍历前两个显示器（如果系统只有1个显示器，只会生成1个 viewport）
        for i in 0..2 {
            if let Some(monitor) = self.monitors.get(i) {
                // 为每个 viewport 生成唯一的 ID
                let viewport_id = egui::ViewportId::from_hash_of(format!("screen_overlay_{}", i));


                let phys_x = monitor.x().unwrap() as f32;
                let phys_y = monitor.y().unwrap() as f32;
                let self_scale = monitor.scale_factor().unwrap();
                let logic_x = phys_x / self_scale;// 必须除以缩放否则如果电脑被缩放了，将导致位置不对
                let logic_y = phys_y / self_scale;

                let pos = egui::pos2(logic_x, logic_y);
                let size = egui::vec2(monitor.width().unwrap() as f32 / self_scale, monitor.height().unwrap() as f32 / self_scale);

                let builder = egui::ViewportBuilder::default()
                    .with_title(format!("Viewport {}", i + 1))
                    .with_position(pos)
                    .with_inner_size(size)
                    .with_decorations(false); // 去除系统边框、标题栏、关闭按钮
                // .with_always_on_top(); // 取决于你是否希望它永远遮挡其他程序

                ctx.show_viewport_immediate(viewport_id, builder, move |ctx, _class| {
                    // 填充对应的背景色
                    let frame = egui::Frame::default().fill(bg_colors[i]);

                    egui::CentralPanel::default().frame(frame).show(ctx, |ui| {
                        // 在覆盖窗口中心显示一些提示信息
                        ui.centered_and_justified(|ui| {
                            let text = format!("这是 Viewport {}\n覆盖在 xcap 显示器 {} 上", i + 1, i + 1);
                            ui.heading(text);
                        });
                    });
                });
            }
        }
    }
}