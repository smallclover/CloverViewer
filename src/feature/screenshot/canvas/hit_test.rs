use eframe::egui::{Painter, Pos2};

use crate::feature::screenshot::canvas::shape::ShapeRender;
use crate::feature::screenshot::capture::DrawnShape;

/// 获取命中的 shape 索引（倒序遍历，后绘制的优先）
pub fn get_hovered_shape_index(
    pos: Pos2,
    shapes: &[DrawnShape],
    global_offset_phys: Pos2,
    ppp: f32,
    painter: &Painter,
) -> Option<usize> {
    shapes.iter().enumerate().rev().find_map(|(index, shape)| {
        if shape.hit_test(pos, global_offset_phys, ppp, painter) {
            Some(index)
        } else {
            None
        }
    })
}
