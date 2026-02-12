use eframe::egui::{self, Color32, Rect, Vec2, Ui, Painter, Layout, Align, Stroke, StrokeKind};
use egui::UiBuilder;
use super::screenshot::{ScreenshotState, ScreenshotTool, ScreenshotAction};

pub fn draw_screenshot_toolbar(
    ui: &mut Ui,
    painter: &Painter,
    state: &mut ScreenshotState,
    toolbar_rect: Rect,
) -> ScreenshotAction {
    let mut action = ScreenshotAction::None;

    // 1. 样式定义
    let rounding = 8.0; // 圆角
    let bg_color = Color32::WHITE; // 白色背景
    let border_color = Color32::from_gray(200); // 浅灰色边框
    let text_color = Color32::BLACK; // 黑色文本（图标）
    let highlight_color = Color32::from_rgb(0, 120, 215); // 选中高亮色（蓝色）
    let item_spacing = 12.0; // 选项间距

    // 2. 绘制背景
    painter.rect_filled(toolbar_rect, rounding, bg_color);
    painter.rect_stroke(
        toolbar_rect,
        rounding,
        Stroke::new(1.0, border_color),
        StrokeKind::Inside,
    );

    // 3. 配置布局
    // 使用居中对齐的水平布局
    let mut child_ui = ui.new_child(UiBuilder::new().max_rect(toolbar_rect).layout(Layout::left_to_right(Align::Center)));

    // 设置间距和文本颜色
    child_ui.style_mut().spacing.item_spacing = Vec2::new(item_spacing, 0.0);
    child_ui.style_mut().visuals.override_text_color = Some(text_color);

    // 4. 绘制按钮
    // 为了让按钮在工具栏中整体居中，我们可以使用 horizontal_centered 或者手动计算 padding
    // 这里简单起见，使用 horizontal 布局，并添加一些 padding

    child_ui.horizontal(|ui| {
        // 添加左侧 padding，使内容居中（简单估算）
        ui.add_space(10.0);

        // 矩形工具
        let rect_btn = ui.add(egui::Button::new("⬜").frame(false));
        if rect_btn.clicked() {
            state.current_tool = Some(ScreenshotTool::Rect);
        }
        if state.current_tool == Some(ScreenshotTool::Rect) {
            painter.rect_stroke(
                rect_btn.rect.expand(2.0),
                4.0,
                Stroke::new(1.5, highlight_color),
                StrokeKind::Outside
            );
        }

        // 圆形工具
        let circle_btn = ui.add(egui::Button::new("⭕").frame(false));
        if circle_btn.clicked() {
            state.current_tool = Some(ScreenshotTool::Circle);
        }
        if state.current_tool == Some(ScreenshotTool::Circle) {
            painter.rect_stroke(
                circle_btn.rect.expand(2.0),
                4.0,
                Stroke::new(1.5, highlight_color),
                StrokeKind::Outside
            );
        }

        // 分隔线
        ui.separator();

        // 取消
        if ui.add(egui::Button::new("❌").frame(false)).clicked() {
            state.selection = None;
            state.toolbar_pos = None;
            state.current_tool = None;
        }

        // 保存
        if ui.add(egui::Button::new("💾").frame(false)).clicked() {
            action = ScreenshotAction::SaveAndClose;
        }
    });

    action
}
