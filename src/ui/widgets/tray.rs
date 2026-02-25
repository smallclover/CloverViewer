use eframe::egui;
use std::sync::Mutex;
use tray_icon::{Icon, MouseButtonState, TrayIconBuilder, TrayIconEvent};
// [新增 1] 引入 global-hotkey 的相关依赖
use global_hotkey::{hotkey::{Code, HotKey, Modifiers}, GlobalHotKeyEvent, GlobalHotKeyManager, HotKeyState};

use windows::Win32::Foundation::HWND;
use windows::Win32::UI::WindowsAndMessaging::{ShowWindow, SW_HIDE, SW_RESTORE};
use winit::raw_window_handle::{HasWindowHandle, RawWindowHandle};

// 测试隐藏托盘文件
//https://github.com/emilk/egui/discussions/737
static VISIBLE: Mutex<bool> = Mutex::new(true);

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // --- 1. 初始化托盘 ---
    let mut icon_data: Vec<u8> = Vec::with_capacity(16 * 16 * 4);
    for _ in 0..256 { icon_data.extend_from_slice(&[255, 0, 0, 255]); }
    let icon = Icon::from_rgba(icon_data, 16, 16)?;

    // 必须用变量持有，保持生命周期
    let _tray_icon = TrayIconBuilder::new()
        .with_icon(icon)
        .with_tooltip("My App")
        .build()?;

    // --- 2. [新增] 初始化全局快捷键 (Alt + S) ---
    // 同样必须用变量持有，防止注册失效
    let hotkey_manager = GlobalHotKeyManager::new()?;
    let hotkey = HotKey::new(Some(Modifiers::ALT), Code::KeyS);
    hotkey_manager.register(hotkey)?;

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([320.0, 240.0]),
        ..Default::default()
    };

    let _ = eframe::run_native(
        "My egui App",
        options,
        Box::new(move |cc| {
            // 获取原生句柄并转为 isize 以支持跨线程
            let RawWindowHandle::Win32(handle) = cc.window_handle().unwrap().as_raw() else {
                panic!("Unsupported platform");
            };
            let hwnd_isize = handle.hwnd.get() as isize;

            // --- 3. 设置托盘事件处理器 ---
            TrayIconEvent::set_event_handler(Some(move |event: TrayIconEvent| {
                match event {
                    TrayIconEvent::Click { button_state: MouseButtonState::Up, .. } => {
                        let mut visible = VISIBLE.lock().unwrap();
                        let window_handle = HWND(hwnd_isize as *mut std::ffi::c_void);
                        if *visible {
                            unsafe { ShowWindow(window_handle, SW_HIDE); }
                            *visible = false;
                        } else {
                            unsafe { ShowWindow(window_handle, SW_RESTORE); }
                            *visible = true;
                        }
                    }
                    _ => return,
                }
            }));

            // --- 4. [新增] 设置全局快捷键事件处理器 ---
            GlobalHotKeyEvent::set_event_handler(Some(move |event: GlobalHotKeyEvent| {
                // 确保是我们注册的热键，并且是松开状态 (防止长按连发)
                if event.id == hotkey.id() && event.state == HotKeyState::Released {
                    let mut visible = VISIBLE.lock().unwrap();
                    let window_handle = HWND(hwnd_isize as *mut std::ffi::c_void);
                    if *visible {
                        unsafe { ShowWindow(window_handle, SW_HIDE); }
                        *visible = false;
                    } else {
                        unsafe { ShowWindow(window_handle, SW_RESTORE); }
                        *visible = true;
                    }
                }
            }));

            Ok(Box::new(MyApp::default()))
        }),
    );
    Ok(())
}

struct MyApp {}

impl Default for MyApp {
    fn default() -> Self {
        Self {}
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Hello World!");
        });
    }
}