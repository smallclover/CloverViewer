use eframe::egui;
use egui::{FontData, FontDefinitions, FontFamily};
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

        Self {
            monitors,
            phys_min_x,
            phys_min_y,
        }
    }
}

impl eframe::App for SingleWindowApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let bg_colors = [
            egui::Color32::from_rgba_premultiplied(40, 60, 40, 240),
            egui::Color32::from_rgba_premultiplied(40, 40, 60, 240),
        ];

        // 获取 egui 这一帧实际使用的全局缩放率 (在你的本子上应该是 1.25)
        let egui_global_scale = ctx.pixels_per_point();

        egui::CentralPanel::default()
            .frame(egui::Frame::none().fill(egui::Color32::TRANSPARENT))
            .show(ctx, |ui| {
                let painter = ui.painter();

                for (i, m) in self.monitors.iter().enumerate() {
                    // 获取绝对的纯物理像素信息
                    let m_x = m.x().unwrap_or(0);
                    let m_y = m.y().unwrap_or(0);
                    let m_w = m.width().unwrap_or(800) as f32;
                    let m_h = m.height().unwrap_or(600) as f32;

                    // 计算该显示器在大画布中的相对【物理】偏移量
                    let phys_rel_x = (m_x - self.phys_min_x) as f32;
                    let phys_rel_y = (m_y - self.phys_min_y) as f32;

                    // 【把物理尺寸强行除以全局缩放率，计算出“缩回”后的逻辑坐标！
                    let logic_x = phys_rel_x / egui_global_scale;
                    let logic_y = phys_rel_y / egui_global_scale;
                    let logic_w = m_w / egui_global_scale;
                    let logic_h = m_h / egui_global_scale;

                    let rect = egui::Rect::from_min_size(
                        egui::pos2(logic_x, logic_y),
                        egui::vec2(logic_w, logic_h),
                    );

                    let color = bg_colors[i % bg_colors.len()];
                    painter.rect_filled(rect, 0.0, color);

                    let text = format!(
                        "显示器 {}\n物理: {}x{}\n逻辑缩回: {:.1}x{:.1}\n当前 egui 缩放率: {}",
                        i + 1, m_w, m_h, logic_w, logic_h, egui_global_scale
                    );

                    painter.text(
                        rect.center(),
                        egui::Align2::CENTER_CENTER,
                        text,
                        egui::FontId::proportional(20.0),
                        egui::Color32::WHITE,
                    );
                }
            });

        egui::Window::new("主控制台")
            .default_pos(egui::pos2(50.0, 50.0))
            .show(ctx, |ui| {
                ui.heading("物理像素除以缩放率映射");
                if ui.button("关闭").clicked() {
                    ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                }
            });
    }
}