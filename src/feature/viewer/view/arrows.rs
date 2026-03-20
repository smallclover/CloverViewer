use egui::{
    Color32, Rect, Response, Ui, Vec2,
    StrokeKind,Align2,FontId, CursorIcon
};

pub enum Nav {
    Prev,
    Next,
}
/// 绘制左右箭头按钮
pub fn draw_arrows(ui: &mut Ui, rect: Rect) -> Option<Nav> {
    let mut clicked = None;

    // 计算按钮的大小：和预览栏缩略图接近，例如 40x80 的长条或 60x60 的圆角矩形
    let btn_size = Vec2::new(48.0, 80.0);
    let margin = 20.0; // 距离边缘的距离
    let hover_zone_width = 100.0; // 固定感应区域宽度

    // 绘制左按钮
    let left_rect = Rect::from_center_size(
        rect.left_center() + Vec2::new(margin + btn_size.x / 2.0, 0.0),
        btn_size,
    );
    // 只有鼠标悬停在左侧区域时才显示左按钮
    // 定义左侧感应区域，固定宽度
    let left_hover_rect = Rect::from_min_max(
        rect.min,
        rect.min + Vec2::new(hover_zone_width, rect.height()),
    );

    if ui.rect_contains_pointer(left_hover_rect) || ui.rect_contains_pointer(left_rect) {
        // 覆盖底层光标，强制显示默认指针
        ui.ctx().set_cursor_icon(CursorIcon::Default);

        if draw_nav_button(ui, left_rect, "⏴").clicked() {
            clicked = Some(Nav::Prev);
        }
    }

    // 绘制右按钮
    let right_rect = Rect::from_center_size(
        rect.right_center() - Vec2::new(margin + btn_size.x / 2.0, 0.0),
        btn_size,
    );
    // 只有鼠标悬停在右侧区域时才显示右按钮
    let right_hover_rect = Rect::from_min_max(
        rect.max - Vec2::new(hover_zone_width, rect.height()),
        rect.max,
    );

    if ui.rect_contains_pointer(right_hover_rect) || ui.rect_contains_pointer(right_rect) {
        // 覆盖底层光标，强制显示默认指针
        ui.ctx().set_cursor_icon(CursorIcon::Default);

        if draw_nav_button(ui, right_rect, "⏵").clicked() {
            clicked = Some(Nav::Next);
        }
    }

    clicked
}

fn draw_nav_button(ui: &mut Ui, rect: Rect, icon: &str) -> Response {
    // 1. 定义交互区域
    let id = ui.make_persistent_id(icon);
    let response = ui.interact(rect, id, egui::Sense::click());

    // 2. 根据状态决定颜色（模仿预览栏的透明深色风格）
    let visuals = ui.style().interact(&response);

    // 背景色：默认极暗透明，悬停时变亮
    let bg_color = if response.hovered() {
        Color32::from_rgba_unmultiplied(60, 60, 60, 180)
    } else {
        Color32::from_rgba_unmultiplied(30, 30, 30, 120)
    };

    // 3. 绘制背景
    ui.painter().rect_filled(
        rect.expand(visuals.expansion), // 悬停时稍微扩大一点
        8.0, // 圆角，与预览栏一致
        bg_color,
    );

    // 4. 绘制边框
    if response.hovered() {
        ui.painter().rect_stroke(
            rect,
            8.0,                 // 圆角 (Rounding)
            (1.0, Color32::GRAY),     // 粗细与颜色 (Stroke)
            StrokeKind::Inside               // 边框类型 (StrokeKind)
        );
    }

    // 5. 绘制图标
    ui.painter().text(
        rect.center(),
        Align2::CENTER_CENTER,
        icon,
        FontId::proportional(32.0),
        Color32::WHITE,
    );

    response
}