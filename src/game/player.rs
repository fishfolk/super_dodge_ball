use macroquad::color::Color;
use crate::{FacingTo, HasDirection, Vec2};
use crate::game::ball::Ball;

#[derive(PartialEq, Debug)]
pub struct Player {
    pub(crate) pos: Vec2,
    pub(crate) rotation: f32,
    pub(crate) vel: Vec2,
    pub(crate) color: Color,
    pub(crate) facing_to: FacingTo,
    pub(crate) ducking: bool,
    pub(crate) jumping: bool,
    pub(crate) has_ball: bool,
    pub(crate) life: i32,
}

impl Player {
    pub fn throwing(&mut self, ball: &mut Ball) {
        let target_pos = (self.pos - ball.pos).normalize();
        ball.throwing(target_pos, self.pos, self.rotation);
    }
}


impl HasDirection for Player {
    fn get_position(&self) -> Vec2 {
        self.pos
    }
    fn get_rotation(&self) -> f32 {
        self.rotation
    }
    fn get_rotation_as_radian(&self) -> f32 {
        self.rotation.to_radians()
    }
}
