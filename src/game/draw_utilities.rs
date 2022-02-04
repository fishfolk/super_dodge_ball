use macroquad::color::Color;
use macroquad::prelude::{draw_line, draw_rectangle_lines};
use crate::Vec2;

pub fn draw_line_a(p1: Vec2, p2: Vec2, thickness: f32, color: Color) {
    draw_line(p1.x, p1.y, p2.x, p2.y, thickness, color);
}

pub fn draw_rectangle_lines_a(p1: Vec2, w: f32, h: f32, thickness: f32, color: Color) {
    draw_rectangle_lines(p1.x, p1.y, w, h, thickness, color);
}
