use egui::{Color32, Context, Id, RichText, Align2};
use std::sync::mpsc::{Receiver, Sender, channel};
/// 通知气泡
#[derive(Clone, Copy, PartialEq)]
pub enum ToastLevel {
    Info,
    Success,
    Error,
}

pub struct ToastConfig {
    pub message: String,
    pub level: ToastLevel,
    pub duration: f32,
    pub show_progress: bool,
}

/// 发送给 Toast 系统的指令
pub enum ToastCommand {
    Show(ToastConfig),
    Dismiss,
}

/// 内部状态：用于管理动画和显示
struct ToastState {
    config: ToastConfig,
    start_time: f64,
    appearing: bool,
}

pub struct ToastSystem {
    state: Option<ToastState>,
    receiver: Receiver<ToastCommand>,
    sender: Sender<ToastCommand>,
}
// 定义一些固定尺寸常数
const TOAST_WIDTH: f32 = 100.0;
const TOAST_MIN_HEIGHT: f32 = 20.0;

impl ToastSystem {


    pub fn new() -> Self {
        let (sender, receiver) = channel();
        Self {
            state: None,
            receiver,
            sender,
        }
    }

    /// 获取一个发送句柄，可以克隆到任何线程
    pub fn manager(&self) -> ToastManager {
        ToastManager {
            sender: self.sender.clone(),
        }
    }

    pub fn update(&mut self, ctx: &Context) {
        // 1. 处理新指令
        while let Ok(cmd) = self.receiver.try_recv() {
            match cmd {
                ToastCommand::Show(config) => {
                    // 如果当前已有 Toast，直接替换 config 但重置时间，实现原地切换
                    self.state = Some(ToastState {
                        config,
                        start_time: ctx.input(|i| i.time),
                        appearing: true,
                    });
                }
                ToastCommand::Dismiss => self.state = None,
            }
        }

        // 2. 渲染逻辑
        if let Some(state) = &mut self.state {
            let now = ctx.input(|i| i.time);
            let elapsed = (now - state.start_time) as f32;
            let remaining = state.config.duration - elapsed;

            if remaining <= 0.0 {
                self.state = None;
                return;
            }

            // 动画常数
            let fade_in_time = 0.2;
            let fade_out_time = 0.4;

            // 计算透明度 (Alpha)
            let alpha = if elapsed < fade_in_time {
                (elapsed / fade_in_time).min(1.0)
            } else if remaining < fade_out_time {
                (remaining / fade_out_time).min(1.0)
            } else {
                1.0
            };
            // --- 单层 Area + Pivot 居中 ---
            egui::Area::new(Id::new("global_toast"))
                .anchor(Align2::CENTER_TOP, [0.0, 60.0]) // 锚点在屏幕顶部中心
                .pivot(Align2::CENTER_TOP)              // 将 Toast 自身的顶部中心对准锚点
                .interactable(false)
                .order(egui::Order::Foreground)
                .show(ctx, |ui| {
                    // 获取对应状态的颜色
                    // 图标
                    let (bg_color, icon, icon_color) = match state.config.level {
                        ToastLevel::Info => (Color32::from_rgb(45, 45, 50), "ℹ", Color32::from_rgb(100, 150, 255)),
                        ToastLevel::Success => (Color32::from_rgb(40, 50, 40), "✔", Color32::from_rgb(100, 255, 150)),
                        ToastLevel::Error => (Color32::from_rgb(50, 40, 40), "✖", Color32::from_rgb(255, 100, 100)),
                    };

                    // 应用透明度
                    let bg_with_alpha = bg_color.gamma_multiply(alpha);
                    let text_color = Color32::WHITE.gamma_multiply(alpha);
                    let icon_color_with_alpha = icon_color.gamma_multiply(alpha);
                    // 计算边框和进度条背景（变亮效果）
                    let border_color = lighten_color(bg_with_alpha, 0.1);

                    // 使用 Frame 包裹内容
                    egui::Frame::NONE
                        .fill(bg_with_alpha)
                        .corner_radius(8.0)
                        .stroke((1.0, border_color))
                        .inner_margin(egui::Margin::symmetric(15, 8))
                        .show(ui, |ui| {
                            ui.set_width(TOAST_WIDTH);
                            ui.set_min_height(TOAST_MIN_HEIGHT);

                            ui.vertical(|ui| {
                                ui.horizontal(|ui| {
                                    ui.add_space(2.0);
                                    ui.label(RichText::new(icon).color(icon_color_with_alpha).size(18.0).strong());
                                    ui.add_space(8.0);

                                    // 允许文字换行，确保不会撑爆容器宽度
                                    ui.label(
                                        RichText::new(&state.config.message)
                                            .color(text_color)
                                            .size(14.0)
                                    );
                                });

                                // 如果有进度条，它会被放置在垂直布局的最底部
                                if state.config.show_progress {
                                    ui.add_space(6.0);
                                    let progress = (remaining / state.config.duration).clamp(0.0, 1.0);
                                    let height = 2.0;
                                    let width = ui.available_width();
                                    let (rect, _) = ui.allocate_at_least(egui::vec2(width, height), egui::Sense::hover());
                                    // 进度条底色
                                    ui.painter().rect_filled(rect, 1.0, border_color);
                                    // 进度条前景
                                    let mut progress_rect = rect;
                                    progress_rect.set_width(width * progress);
                                    ui.painter().rect_filled(progress_rect, 1.0, icon_color_with_alpha);
                                }
                            });
                        });
                });
            // 只要有 Toast 在显示，就持续请求重绘，保证动画和进度条丝滑
            ctx.request_repaint();
        }
    }
}

/// 一个简单的辅助函数，让 Color32 变亮
fn lighten_color(color: Color32, amount: f32) -> Color32 {
    let mut hsva = egui::ecolor::Hsva::from(color);
    hsva.v = (hsva.v + amount).clamp(0.0, 1.0); // 增加 Value (明度)
    Color32::from(hsva)
}

/// 外部调用的管理器
#[derive(Clone)]
pub struct ToastManager {
    sender: Sender<ToastCommand>,
}

impl ToastManager {
    pub fn show(&self, message: impl Into<String>, level: ToastLevel, duration: f32, show_progress: bool) {
        let _ = self.sender.send(ToastCommand::Show(ToastConfig {
            message: message.into(),
            level,
            duration,
            show_progress,
        }));
    }

    pub fn success(&self, message: impl Into<String>) {
        self.show(message, ToastLevel::Success, 3.0, false);
    }

    pub fn error(&self, message: impl Into<String>) {
        self.show(message, ToastLevel::Error, 4.0, false);
    }

    pub fn info(&self, message: impl Into<String>) {
        self.show(message, ToastLevel::Info, 2.5, true);
    }
}
