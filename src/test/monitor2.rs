use eframe::egui;
use egui::{FontData, FontDefinitions, FontFamily};
use xcap::Monitor;
use std::sync::Arc;
/// 使用三种方式来初始化
/// drop 销毁重建
/// show_viewport_immediate 立即隐藏
/// show_viewport_deferred 隐藏
/// TODO当前问题，主显示器的viewport不是1
fn main() -> eframe::Result<()> {
    // 1. 初始化主窗口 (App 窗口)
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("主控制台 (App Window)")
            .with_inner_size([400.0, 300.0])
            .with_position(egui::pos2(100.0, 100.0))
            .with_always_on_top(), // 给主窗口一个初始位置避免被遮挡
        ..Default::default()
    };

    // 在启动 egui 之前，先用 xcap 获取所有显示器的信息
    let monitors = Monitor::all().unwrap_or_else(|err| {
        eprintln!("获取显示器信息失败: {}", err);
        vec![]
    });

    eframe::run_native(
        "Xcap Egui App",
        options,
        Box::new(move |cc| Ok(Box::new(MultiWindowApp::new(monitors, cc)))),
    )
}

struct MultiWindowApp {
    monitors: Vec<Monitor>,
    // 新增：用于控制是否显示副窗口的状态
    show_viewports: bool,
}

impl MultiWindowApp {
    pub const APP_FONT: &[u8] = include_bytes!(
        concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/assets/fonts/msyhl.ttf"
        )
    );

    fn new(monitors: Vec<Monitor>, cc: &eframe::CreationContext<'_>) -> Self {
        // 1. 设置字体
        let mut fonts = FontDefinitions::default();
        fonts.font_data.insert("my_font".to_owned(), Arc::new(FontData::from_static(Self::APP_FONT)));
        fonts.families.get_mut(&FontFamily::Proportional).unwrap().insert(0, "my_font".to_owned());
        cc.egui_ctx.set_fonts(fonts);

        Self {
            monitors,
            // 初始化为 true，启动时默认显示这两个 viewport
            show_viewports: true,
        }
    }

    fn hidden_viewports(&mut self, ctx: &egui::Context) {
        // ==========================================
        // 窗口 1: App 窗口 (主控制台)
        // ==========================================
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("主控制窗口");
            ui.separator();

            // 新增：控制 viewport 显示/隐藏的按钮
            ui.horizontal(|ui| {
                let button_text = if self.show_viewports {
                    "隐藏覆盖窗口"
                } else {
                    "显示覆盖窗口"
                };

                // 点击按钮时，对状态值取反
                if ui.button(button_text).clicked() {
                    self.show_viewports = !self.show_viewports;
                }

                // 给出一个小提示状态
                ui.label(if self.show_viewports { "🟢 正在显示" } else { "🟡 已隐藏 (后台保活)" });
            });

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
        // 新增：只有当 show_viewports 为 true 时，才执行生成 viewport 的逻辑
        let bg_colors = [
            egui::Color32::from_rgb(40, 60, 40), // 墨绿色 (显示器 1)
            egui::Color32::from_rgb(40, 40, 60), // 藏青色 (显示器 2)
        ];

        // 遍历前两个显示器
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

                ctx.show_viewport_immediate(viewport_id, builder, move |ctx, _class| {
                    let frame = egui::Frame::default().fill(bg_colors[i]);

                    egui::CentralPanel::default().frame(frame).show(ctx, |ui| {
                        ui.centered_and_justified(|ui| {
                            let text = format!("这是 Viewport {}\n覆盖在 xcap 显示器 {} 上", i + 1, i + 1);
                            ui.heading(text);
                        });
                    });
                });
            }
        }
    }

    fn drop_viewports(&mut self, ctx: &egui::Context) {
        // ==========================================
        // 窗口 1: App 窗口 (主控制台)
        // ==========================================
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("主控制窗口");
            ui.separator();

            // 新增：控制 viewport 显示/隐藏的按钮
            ui.horizontal(|ui| {
                let button_text = if self.show_viewports {
                    "隐藏覆盖窗口"
                } else {
                    "显示覆盖窗口"
                };

                // 点击按钮时，对状态值取反
                if ui.button(button_text).clicked() {
                    self.show_viewports = !self.show_viewports;
                }

                // 给出一个小提示状态
                ui.label(if self.show_viewports { "🟢 正在显示" } else { "🔴 已隐藏" });
            });

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
        // 新增：只有当 show_viewports 为 true 时，才执行生成 viewport 的逻辑
        if self.show_viewports {
            let bg_colors = [
                egui::Color32::from_rgb(40, 60, 40), // 墨绿色 (显示器 1)
                egui::Color32::from_rgb(40, 40, 60), // 藏青色 (显示器 2)
            ];

            // 遍历前两个显示器
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
                        .with_decorations(false);

                    ctx.show_viewport_immediate(viewport_id, builder, move |ctx, _class| {
                        let frame = egui::Frame::default().fill(bg_colors[i]);

                        egui::CentralPanel::default().frame(frame).show(ctx, |ui| {
                            ui.centered_and_justified(|ui| {
                                let text = format!("这是 Viewport {}\n覆盖在 xcap 显示器 {} 上", i + 1, i + 1);
                                ui.heading(text);
                            });
                        });
                    });
                }
            }
        } // if self.show_viewports 结束
    }

    fn hidden_viewports_2(&mut self, ctx: &egui::Context) {
        // ==========================================
        // 窗口 1: App 窗口 (主控制台)
        // ==========================================
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("主控制窗口");
            ui.separator();

            // 控制 viewport 显示/隐藏的按钮
            ui.horizontal(|ui| {
                let button_text = if self.show_viewports {
                    "隐藏覆盖窗口"
                } else {
                    "显示覆盖窗口"
                };

                // 点击按钮时，对状态值取反
                if ui.button(button_text).clicked() {
                    self.show_viewports = !self.show_viewports;
                }

                // 给出一个小提示状态
                ui.label(if self.show_viewports { "🟢 正在显示" } else { "🟡 已隐藏 (后台保活)" });
            });

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
        let bg_colors = [
            egui::Color32::from_rgb(40, 60, 40), // 墨绿色 (显示器 1)
            egui::Color32::from_rgb(40, 40, 60), // 藏青色 (显示器 2)
        ];

        // 遍历前两个显示器
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
                    .with_visible(self.show_viewports); // 这里依然由主循环的 self 控制可见性

                // 【关键改动】：提前把闭包里需要用到的数据复制出来
                let bg_color = bg_colors[i];
                let display_index = i + 1;

                // 替换为 deferred，并将独立的数据 move 进去
                ctx.show_viewport_deferred(viewport_id, builder, move |ctx, _class| {
                    let frame = egui::Frame::default().fill(bg_color);

                    egui::CentralPanel::default().frame(frame).show(ctx, |ui| {
                        ui.centered_and_justified(|ui| {
                            let text = format!("这是 Viewport {}\n覆盖在 xcap 显示器 {} 上", display_index, display_index);
                            ui.heading(text);
                        });
                    });
                });
            }
        }
    }
}

// TODO 当前显示器是viewport2，不是1？

impl eframe::App for MultiWindowApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // self.hidden_viewports(ctx);
        self.hidden_viewports_2(ctx);
        // self.drop_viewports(ctx);
    }
}