use eframe::egui;
use egui::{Context, FontData, FontDefinitions, FontFamily, ViewportBuilder, ViewportId, ViewportClass, ColorImage, ViewportCommand};
use std::{
    path::PathBuf,
    sync::Arc,
};
use image::{RgbaImage, GenericImage, GenericImageView};
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
    crop::handle_crop_mode,
};
use crate::ui::viewer;
use crate::utils::image::load_icon;
use crate::screenshot::{ScreenshotState, CapturedScreen, draw_screenshot_ui, ScreenshotAction};
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
        handle_input_events(ctx, &mut self.data, &mut self.state);
    }

    fn draw_ui(&mut self, ctx: &Context) {
        if !self.screenshot_state.is_active {
            viewer::draw_top_panel(ctx, &mut self.state, &self.config);
            viewer::draw_bottom_panel(ctx, &mut self.state, &mut self.screenshot_state.is_active);
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

            handle_crop_mode(ctx, &mut self.state, &mut self.data);
        }
    }

    fn handle_screenshot(&mut self, ctx: &Context) {
        if !self.screenshot_state.is_active {
            return;
        }

        if self.screenshot_state.captures.is_empty() {
            println!("[DEBUG] Capturing screens...");
            if let Ok(monitors) = Monitor::all() {
                for (i, monitor) in monitors.iter().enumerate() {
                     if let (Ok(image), Ok(width)) = (monitor.capture_image(), monitor.width()) {
                        if width == 0 {
                            println!("[DEBUG] Monitor {} has zero width, skipping.", i);
                            continue;
                        }
                        let scale_factor = image.width() as f32 / width as f32;
                        println!("[DEBUG] Monitor {}: name={}, physical_size=({},{}), logical_size=({},{}), scale_factor={}",
                                 i, monitor.name().unwrap_or("Unknown".parse().unwrap()), image.width(), image.height(), monitor.width().unwrap(), monitor.height().unwrap(), scale_factor);

                        let color_image = ColorImage::from_rgba_unmultiplied(
                            [image.width() as usize, image.height() as usize],
                            &image,
                        );
                        self.screenshot_state.captures.push(CapturedScreen {
                            raw_image: Arc::new(image),
                            image: color_image,
                            screen_info: monitor.clone(),
                            texture: None,
                            scale_factor,
                        });
                    } else {
                        println!("[DEBUG] Failed to capture image for monitor {}", i);
                    }
                }
            } else {
                println!("[DEBUG] Could not get monitor list.");
            }
        }

        let mut final_action = ScreenshotAction::None;
        let mut wants_to_close_viewports = false;

        for i in 0..self.screenshot_state.captures.len() {
            let screen_info = self.screenshot_state.captures[i].screen_info.clone();
            let viewport_id = ViewportId::from_hash_of(format!("screenshot_{}", screen_info.name().unwrap()));
            let pos_in_logical_pixels = egui::pos2(screen_info.x().unwrap() as f32, screen_info.y().unwrap() as f32);

            ctx.show_viewport_immediate(
                viewport_id,
                ViewportBuilder::default()
                    .with_title("Screenshot")
                    .with_fullscreen(true)
                    .with_decorations(false)
                    .with_always_on_top()
                    .with_position(pos_in_logical_pixels),
                |ctx, class| {
                    if class == ViewportClass::Immediate {
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

        if wants_to_close_viewports {
            if final_action == ScreenshotAction::SaveAndClose {
                println!("[DEBUG] SaveAndClose action triggered.");
                if let Some(selection) = self.screenshot_state.selection {
                    if !selection.is_positive() {
                        println!("[DEBUG] Selection is not positive, aborting save.");
                        return;
                    }
                    println!("[DEBUG] Logical Selection: {:?}", selection);

                    let captures_data: Vec<_> = self.screenshot_state.captures.iter().map(|c| {
                        (
                            c.raw_image.clone(),
                            egui::Rect::from_min_size(
                                egui::pos2(c.screen_info.x().unwrap() as f32 / c.scale_factor, c.screen_info.y().unwrap() as f32 / c.scale_factor),
                                egui::vec2(c.screen_info.width().unwrap() as f32 / c.scale_factor, c.screen_info.height().unwrap() as f32 / c.scale_factor),
                            ),
                            c.scale_factor,
                        )
                    }).collect();

                    let target_scale = ctx.pixels_per_point();
                    println!("[DEBUG] Target image scale (from primary window): {}", target_scale);

                    std::thread::spawn(move || {
                        println!("[THREAD] Starting image stitching process.");
                        let final_width = (selection.width() * target_scale).round() as u32;
                        let final_height = (selection.height() * target_scale).round() as u32;
                        println!("[THREAD] Final image dimensions: {}x{}", final_width, final_height);

                        if final_width == 0 || final_height == 0 {
                            eprintln!("[ERROR] Failed to save image: Final image dimensions are zero.");
                            return;
                        }

                        let mut final_image = RgbaImage::new(final_width, final_height);

                        for (i, (raw_image, monitor_rect_logical, scale_factor)) in captures_data.iter().enumerate() {
                            println!("[THREAD] Processing capture {}: monitor_rect_logical={:?}, scale_factor={}", i, monitor_rect_logical, scale_factor);
                            let intersection_logical = selection.intersect(*monitor_rect_logical);
                            println!("[THREAD]   Intersection (logical): {:?}", intersection_logical);

                            if !intersection_logical.is_positive() {
                                println!("[THREAD]   No intersection, skipping.");
                                continue;
                            }

                            let crop_x_phys = ((intersection_logical.min.x - monitor_rect_logical.min.x) * scale_factor).round() as u32;
                            let crop_y_phys = ((intersection_logical.min.y - monitor_rect_logical.min.y) * scale_factor).round() as u32;
                            let crop_w_phys = (intersection_logical.width() * scale_factor).round() as u32;
                            let crop_h_phys = (intersection_logical.height() * scale_factor).round() as u32;
                            println!("[THREAD]   Physical crop: x={}, y={}, w={}, h={}", crop_x_phys, crop_y_phys, crop_w_phys, crop_h_phys);

                            if crop_x_phys.saturating_add(crop_w_phys) > raw_image.width() || crop_y_phys.saturating_add(crop_h_phys) > raw_image.height() {
                                eprintln!("[ERROR] Failed to save image: Crop area {:?} is out of bounds for raw image size {}x{}",
                                    (crop_x_phys, crop_y_phys, crop_w_phys, crop_h_phys), raw_image.width(), raw_image.height());
                                continue;
                            }

                            let cropped_part_native = image::imageops::crop_imm(
                                &**raw_image,
                                crop_x_phys,
                                crop_y_phys,
                                crop_w_phys,
                                crop_h_phys,
                            ).to_image();
                            println!("[THREAD]   Cropped native part successfully.");

                            let paste_x_start = ((intersection_logical.min.x - selection.min.x) * target_scale).round() as u32;
                            let paste_y_start = ((intersection_logical.min.y - selection.min.y) * target_scale).round() as u32;
                            let paste_x_end = ((intersection_logical.max.x - selection.min.x) * target_scale).round() as u32;
                            let paste_y_end = ((intersection_logical.max.y - selection.min.y) * target_scale).round() as u32;

                            let target_w = paste_x_end - paste_x_start;
                            let target_h = paste_y_end - paste_y_start;
                            println!("[THREAD]   Paste target: w={}, h={}", target_w, target_h);

                            if target_w == 0 || target_h == 0 {
                                println!("[THREAD]   Paste target has zero dimension, skipping.");
                                continue;
                            }

                            let part_to_paste = if cropped_part_native.width() != target_w || cropped_part_native.height() != target_h {
                                println!("[THREAD]   Resizing part from {}x{} to {}x{}", cropped_part_native.width(), cropped_part_native.height(), target_w, target_h);
                                image::imageops::resize(
                                    &cropped_part_native,
                                    target_w,
                                    target_h,
                                    image::imageops::FilterType::Lanczos3,
                                )
                            } else {
                                println!("[THREAD]   No resize needed.");
                                cropped_part_native
                            };

                            println!("[THREAD]   Pasting at: x={}, y={}", paste_x_start, paste_y_start);
                            if let Err(e) = final_image.copy_from(&part_to_paste, paste_x_start, paste_y_start) {
                                eprintln!("[ERROR] Failed to save image: Could not copy image part: {}", e);
                            };
                        }

                        if let Ok(profile) = std::env::var("USERPROFILE") {
                            let desktop = PathBuf::from(profile).join("Desktop");
                            let path = desktop.join(format!("screenshot_{}.png", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs()));
                            println!("[THREAD] Saving final image to: {:?}", path);
                            if let Err(e) = final_image.save(&path) {
                                eprintln!("[ERROR] Failed to save image to path {:?}: {}", path, e);
                            } else {
                                println!("[SUCCESS] Image saved successfully!");
                            }
                        } else {
                            eprintln!("[ERROR] Could not find USERPROFILE to determine save location.");
                        }
                    });
                } else {
                    println!("[DEBUG] Save action triggered, but there is no selection.");
                }
            }

            println!("[DEBUG] Resetting screenshot state.");
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
