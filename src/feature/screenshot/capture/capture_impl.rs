use crate::feature::screenshot::capture::{CapturedScreen, ScreenshotState};
use crate::model::device::MonitorInfo;
use crate::os::current_platform;
use eframe::egui::{ColorImage, Context, Pos2, Rect};
use std::sync::Arc;
use std::sync::mpsc::TryRecvError;
use std::sync::mpsc::channel;
use std::thread;
use std::time::Duration;
use xcap::Monitor;

/// 处理截图捕获过程
/// 返回 true 表示应该退出截图模式
pub(super) fn handle_capture_process(
    ctx: &Context,
    screenshot_state: &mut ScreenshotState,
) -> bool {
    if !screenshot_state.is_capturing {
        screenshot_state.is_capturing = true;

        ctx.request_repaint();

        let (tx, rx) = channel();
        screenshot_state.capture_receiver = Some(rx);
        let ctx_clone = ctx.clone();

        thread::spawn(move || {
            thread::sleep(Duration::from_millis(150));
            tracing::debug!("Capturing screens and windows in background...");

            let mut captures = Vec::new();
            if let Ok(monitors) = Monitor::all() {
                for monitor in monitors {
                    if let (Ok(image), Ok(width)) = (monitor.capture_image(), monitor.width()) {
                        if width == 0 {
                            continue;
                        }

                        let color_image = ColorImage::from_rgba_unmultiplied(
                            [image.width() as usize, image.height() as usize],
                            &image,
                        );

                        let info = MonitorInfo {
                            name: monitor.name().unwrap_or_default(),
                            x: monitor.x().unwrap_or(0),
                            y: monitor.y().unwrap_or(0),
                            width: monitor.width().unwrap_or(0),
                            height: monitor.height().unwrap_or(0),
                            scale_factor: monitor.scale_factor().unwrap_or(1.0),
                        };

                        captures.push(CapturedScreen {
                            raw_image: Arc::new(image),
                            image: color_image,
                            screen_info: info,
                        });
                    }
                }
            }

            let mut window_rects = Vec::new();
            if let Ok(windows) = xcap::Window::all() {
                for w in windows {
                    if !w.is_minimized().unwrap_or(true) {
                        let app_name = w.app_name().unwrap_or_default().to_lowercase();
                        if app_name.contains("cloverviewer") || app_name.contains("screenshot") {
                            continue;
                        }

                        let rect = Rect::from_min_size(
                            Pos2::new(w.x().unwrap_or(0) as f32, w.y().unwrap_or(0) as f32),
                            egui::vec2(
                                w.width().unwrap_or(0) as f32,
                                w.height().unwrap_or(0) as f32,
                            ),
                        );
                        if rect.width() > 50.0 && rect.height() > 50.0 {
                            window_rects.push(rect);
                        }
                    }
                }
            }
            // 使用系统 API 捕获任务栏
            let taskbars = current_platform().get_taskbar_rects();
            window_rects.extend(taskbars);
            let _ = tx.send((captures, window_rects));
            ctx_clone.request_repaint();
        });
    }

    if let Some(rx) = &screenshot_state.capture_receiver {
        match rx.try_recv() {
            Ok((captures, window_rects)) => {
                for cap in &captures {
                    let monitor_name = &cap.screen_info.name;
                    if let Some(texture) = screenshot_state.texture_pool.get_mut(monitor_name) {
                        texture.set(cap.image.clone(), Default::default());
                    } else {
                        let texture = ctx.load_texture(
                            format!("screenshot_{}", monitor_name),
                            cap.image.clone(),
                            Default::default(),
                        );
                        screenshot_state
                            .texture_pool
                            .insert(monitor_name.clone(), texture);
                    }
                }

                screenshot_state.captures = captures;
                screenshot_state.window_rects = window_rects;
                screenshot_state.is_capturing = false;
                screenshot_state.capture_receiver = None;
                ctx.request_repaint();
            }
            Err(TryRecvError::Empty) => {
                ctx.request_repaint_after(Duration::from_millis(16));
            }
            Err(TryRecvError::Disconnected) => {
                screenshot_state.is_capturing = false;
                screenshot_state.capture_receiver = None;
                return true; // 表示应该退出截图模式
            }
        }
    }
    false // 不需要退出
}
