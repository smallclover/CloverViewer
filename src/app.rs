use eframe::egui;
use egui::{Context, FontData, FontDefinitions, FontFamily, ViewportBuilder, ViewportId, ViewportClass, ColorImage, ViewportCommand};
use std::{
    path::PathBuf,
    sync::Arc,
};
use image::{RgbaImage, GenericImage};
use crate::{
    model::config::{load_config, save_config, Config},
    model::state::{ViewState, ViewMode},
    core::business::BusinessData,
};
use crate::ui::components::{
    context_menu::handle_context_menu_action,
    modal::ModalAction,
    mouse::handle_input_events,
    properties_panel::draw_properties_panel,
    resources::APP_FONT,
};
use crate::ui::viewer;
use crate::utils::image::load_icon;
use crate::ui::components::screenshot::{ScreenshotState, CapturedScreen, draw_screenshot_ui, ScreenshotAction};
use xcap::Monitor;

pub fn run() -> eframe::Result<()> {
    let mut options = eframe::NativeOptions {
        viewport: ViewportBuilder::default().with_inner_size([1024.0, 768.0]),
        ..Default::default()
    };
    options.viewport = options.viewport.with_icon(load_icon());

    let start_path = std::env::args().nth(1).map(PathBuf::from);

    eframe::run_native(
        "CloverViewer",
        options,
        Box::new(|cc| Ok(Box::new(CloverApp::new(cc, start_path)))),
    )
}

pub struct CloverApp {
    data: BusinessData,
    state: ViewState,
    config: Config,
    screenshot_state: ScreenshotState,
}

impl CloverApp {
    pub fn new(cc: &eframe::CreationContext<'_>, start_path: Option<PathBuf>) -> Self {
        let mut fonts = FontDefinitions::default();
        fonts.font_data.insert(
            "my_font".to_owned(),
            Arc::new(FontData::from_static(APP_FONT)),
        );
        fonts
            .families
            .get_mut(&FontFamily::Proportional)
            .unwrap()
            .insert(0, "my_font".to_owned());
        cc.egui_ctx.set_fonts(fonts);

        let config = load_config();

        let mut app = Self {
            data: BusinessData::new(),
            state: ViewState::default(),
            config,
            screenshot_state: ScreenshotState::default(),
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

    fn handle_input_events(&mut self, ctx: &Context) {
        handle_input_events(ctx, &mut self.data);
    }

    fn draw_ui(&mut self, ctx: &Context) {
        if !self.screenshot_state.is_active {
            viewer::draw_top_panel(ctx, &mut self.state, &self.config,&mut self.screenshot_state.is_active);
            viewer::draw_bottom_panel(ctx, &mut self.state);
            viewer::draw_central_panel(ctx, &mut self.data, &mut self.state, &self.config);
            draw_properties_panel(ctx, &mut self.state, &self.data, &self.config);
            self.state.toast_system.update(ctx);
        }
    }

    fn handle_ui_interactions(&mut self, ctx: &Context) {
        if !self.screenshot_state.is_active {
            let mut temp_config = self.config.clone();
            let (context_menu_action, modal_action) =
                viewer::draw_overlays(ctx, &self.data, &mut self.state, &mut temp_config);

            if let Some(action) = context_menu_action {
                handle_context_menu_action(action, &self.data, &mut self.state, &self.config);
            }

            if let Some(ModalAction::Apply) = modal_action {
                self.config = temp_config;
                save_config(&self.config);
            }
        }
    }

    fn handle_screenshot(&mut self, ctx: &Context) {
        if !self.screenshot_state.is_active {
            return;
        }

        // --- 1. 初始化捕获 (只在开始时执行一次) ---
        if self.screenshot_state.captures.is_empty() {
            println!("[DEBUG] Capturing screens...");
            if let Ok(monitors) = Monitor::all() {
                for (i, monitor) in monitors.iter().enumerate() {
                    // 尝试捕获图像
                    if let (Ok(image), Ok(width)) = (monitor.capture_image(), monitor.width()) {
                        if width == 0 { continue; }

                        let color_image = ColorImage::from_rgba_unmultiplied(
                            [image.width() as usize, image.height() as usize],
                            &image,
                        );

                        self.screenshot_state.captures.push(CapturedScreen {
                            raw_image: Arc::new(image),
                            image: color_image,
                            screen_info: monitor.clone(),
                            texture: None,
                        });
                    } else {
                        eprintln!("[ERROR] Failed to capture monitor {}", i);
                    }
                }
            }
        }

        let mut final_action = ScreenshotAction::None;
        let mut wants_to_close_viewports = false;

        // --- 2. 渲染所有视口 ---
        for i in 0..self.screenshot_state.captures.len() {
            let screen_info = self.screenshot_state.captures[i].screen_info.clone();
            let viewport_id = ViewportId::from_hash_of(format!("screenshot_{}", screen_info.name().unwrap_or_default()));
            // 获取物理参数
            let phys_x = screen_info.x().unwrap_or(0) as f32;
            let phys_y = screen_info.y().unwrap_or(0) as f32;
            // 当前屏幕的缩放，如果不设置会导致变形
            let self_scale = screen_info.scale_factor().unwrap().clone();
            // 需要将物理坐标除去缩放
            let logic_x = phys_x/self_scale;
            let logic_y = phys_y/self_scale;

            let pos = egui::pos2(logic_x,logic_y);

            ctx.show_viewport_immediate(
                viewport_id,
                ViewportBuilder::default()
                    .with_title("Screenshot")
                    .with_fullscreen(true)
                    .with_decorations(false)
                    // 下面这三个注解可能有用，目前顶部偶尔存在闪烁的情况，出发条件不明
                    // .with_resizable(false)
                    // .with_maximized(false)
                    // .with_drag_and_drop(false)
                    // .with_always_on_top() // 0.33 无参数
                    //点击其中一个，Windows 可能会尝试重新计算 Z 轴顺序，导致另一个短暂闪烁。
                    // 就是这个问题，导致了截图和save的时候疯狂闪烁
                    .with_position(pos),
                |ctx, class| {
                    if class == ViewportClass::Immediate {
                        // 传入 index，UI 内部会处理坐标转换
                        let action = draw_screenshot_ui(ctx, &mut self.screenshot_state, i);
                        if action != ScreenshotAction::None {
                            final_action = action;
                            wants_to_close_viewports = true;
                        }
                    }
                    if wants_to_close_viewports {
                        ctx.send_viewport_cmd(ViewportCommand::Close);
                    }
                },
            );
        }

        // --- 3. 处理关闭与保存逻辑 ---
        if wants_to_close_viewports {
            if final_action == ScreenshotAction::SaveAndClose {
                println!("[DEBUG] SaveAndClose triggered.");

                // 获取全局物理坐标选区
                if let Some(selection_phys) = self.screenshot_state.selection {
                    if !selection_phys.is_positive() {
                        return; // 选区无效
                    }

                    println!("[DEBUG] Physical Selection: {:?}", selection_phys);

                    // 准备线程所需数据
                    // 这里只需要原始图片和该屏幕在全局空间中的物理位置矩形
                    let captures_data: Vec<_> = self.screenshot_state.captures.iter().map(|c| {
                        (
                            c.raw_image.clone(),
                            // 构建显示器的物理矩形
                            egui::Rect::from_min_size(
                                egui::pos2(c.screen_info.x().unwrap_or(0) as f32, c.screen_info.y().unwrap_or(0) as f32),
                                egui::vec2(c.screen_info.width().unwrap_or(0) as f32, c.screen_info.height().unwrap_or(0) as f32),
                            )
                        )
                    }).collect();

                    // 启动保存线程
                    std::thread::spawn(move || {
                        println!("[THREAD] Starting stitching...");

                        // 1. 确定画布大小
                        // 因为 selection_phys 已经是物理像素，直接取整即可，不需要再乘 scale
                        let final_width = selection_phys.width().round() as u32;
                        let final_height = selection_phys.height().round() as u32;

                        if final_width == 0 || final_height == 0 { return; }

                        let mut final_image = RgbaImage::new(final_width, final_height);

                        for (i, (raw_image, monitor_rect_phys)) in captures_data.iter().enumerate() {
                            // 2. 计算 选区 与 当前屏幕 的交集 (物理空间)
                            let intersection = selection_phys.intersect(*monitor_rect_phys);

                            if !intersection.is_positive() {
                                continue; // 选区没有覆盖这个屏幕
                            }

                            // 3. 计算裁切区域 (相对于当前屏幕左上角)
                            // crop_x = 交集左边 - 屏幕左边
                            let crop_x = (intersection.min.x - monitor_rect_phys.min.x).max(0.0).round() as u32;
                            let crop_y = (intersection.min.y - monitor_rect_phys.min.y).max(0.0).round() as u32;
                            let crop_w = intersection.width().round() as u32;
                            let crop_h = intersection.height().round() as u32;

                            // 边界检查，防止 panic
                            if crop_x + crop_w > raw_image.width() || crop_y + crop_h > raw_image.height() {
                                eprintln!("[ERROR] Monitor {} crop out of bounds.", i);
                                continue;
                            }

                            // 4. 执行裁切
                            let cropped_part = image::imageops::crop_imm(
                                &**raw_image,
                                crop_x,
                                crop_y,
                                crop_w,
                                crop_h,
                            ).to_image();

                            // 5. 计算粘贴位置 (相对于最终画布左上角)
                            // paste_x = 交集左边 - 选区左边
                            let paste_x = (intersection.min.x - selection_phys.min.x).max(0.0).round() as u32;
                            let paste_y = (intersection.min.y - selection_phys.min.y).max(0.0).round() as u32;

                            // 6. 粘贴到大图
                            // 因为全是物理像素，直接 1:1 拷贝，不需要 resize
                            if let Err(e) = final_image.copy_from(&cropped_part, paste_x, paste_y) {
                                eprintln!("[ERROR] Failed to copy part: {}", e);
                            }
                        }

                        // 7. 保存文件
                        if let Ok(profile) = std::env::var("USERPROFILE") {
                            let desktop = PathBuf::from(profile).join("Desktop");
                            let timestamp = std::time::SystemTime::now()
                                .duration_since(std::time::UNIX_EPOCH)
                                .unwrap_or_default()
                                .as_secs();
                            let path = desktop.join(format!("screenshot_{}.png", timestamp));

                            if let Err(e) = final_image.save(&path) {
                                eprintln!("[ERROR] Save failed: {}", e);
                            } else {
                                println!("[SUCCESS] Saved to {:?}", path);
                            }
                        }
                    });
                }
            }

            // --- 4. 重置状态 ---
            self.screenshot_state.is_active = false;
            self.screenshot_state.captures.clear();
            self.screenshot_state.selection = None;
            self.screenshot_state.drag_start = None;
            self.screenshot_state.save_button_pos = None;
        }
    }
}

impl eframe::App for CloverApp {
    fn update(&mut self, ctx: &Context, _: &mut eframe::Frame) {
        self.handle_background_tasks(ctx);
        self.handle_input_events(ctx);
        self.draw_ui(ctx);
        self.handle_ui_interactions(ctx);
        self.handle_screenshot(ctx);
    }
}
