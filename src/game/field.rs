use macroquad::prelude::*;
use crate::{Player, PLAYER_HEIGHT, PLAYER_WIDTH};

pub struct Field {
    pub(crate) top_left: Vec2,
    pub(crate) bottom_left: Vec2,
    pub(crate) top_right: Vec2,
    pub(crate) bottom_right: Vec2,
    pub(crate) mid_section_top: Vec2,
    pub(crate) mid_section_bottom: Vec2,
    pub(crate) right_edge: f32,
    pub(crate) left_edge: f32,
    pub(crate) bottom_edge: f32,
    pub(crate) top_edge: f32,
    pub(crate) mid_section: f32,
}

impl Field {
    pub(crate) fn default() -> Self {
        let top_edge = screen_height() / 3.;
        let left_edge = screen_width() / 10.;
        let bottom_edge = screen_height() - screen_height() / 4.;
        let right_edge = screen_width() - screen_width() / 10.;

        let top_left = Vec2::new(left_edge, top_edge);
        let bottom_left = Vec2::new(left_edge, bottom_edge);
        let top_right = Vec2::new(right_edge, top_edge);
        let bottom_right = Vec2::new(right_edge, bottom_edge);
        let mid_section = (right_edge - left_edge) / 2. + left_edge;
        let mid_section_top = Vec2::new(mid_section, top_edge);
        let mid_section_bottom = Vec2::new(mid_section, bottom_left.y);
        Field {
            top_left,
            bottom_left,
            top_right,
            bottom_right,
            mid_section_top,
            mid_section_bottom,
            right_edge,
            left_edge,
            bottom_edge,
            top_edge,
            mid_section,
        }
    }

    pub(crate) fn point_outside_field(&self, position: Vec2, r: f32) -> bool {
        if position.x > self.right_edge { return true; }
        if position.x + r < self.left_edge { return true; }
        if position.y - r > self.bottom_edge { return true; }
        if position.y + r < self.top_edge { return true; }
        false
    }

    pub(crate) fn player_outside_field(&self, player: &Player) -> bool {
        if player.pos.x > self.right_edge { return true; }
        if player.pos.x + PLAYER_WIDTH < self.left_edge { return true; }
        if player.pos.y > self.bottom_edge { return true; }
        if player.pos.y + PLAYER_HEIGHT < self.top_edge { return true; }
        false
    }
}
