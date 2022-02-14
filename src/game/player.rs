use macroquad::color::Color;
use macroquad::math::Rect;
use crate::{AnimationPlayer, FacingTo, HasDirection, Vec2};
use crate::game::ball::Ball;
use crate::game::character::PlayerCharacterParams;

pub struct Player {
    pub(crate) id: u8,
    pub(crate) pos: Vec2,
    pub(crate) rotation: f32,
    pub(crate) vel: Vec2,
    pub(crate) color: Color,
    pub(crate) facing_to: FacingTo,
    pub(crate) ducking: bool,
    pub(crate) jumping: bool,
    pub(crate) has_ball: bool,
    pub(crate) life: i32,
    pub(crate) running: bool,
    pub(crate) ready_to_catch: bool,
    pub(crate) camera_box: Rect,
    pub(crate) animation_player: AnimationPlayer,
}

impl Player {
    pub const IDLE_ANIMATION_ID: &'static str = "idle";
    pub const MOVE_ANIMATION_ID: &'static str = "move";
    pub const JUMP_ANIMATION_ID: &'static str = "jump";
    pub const FALL_ANIMATION_ID: &'static str = "fall";
    pub const CROUCH_ANIMATION_ID: &'static str = "crouch";
    pub const DEATH_BACK_ANIMATION_ID: &'static str = "death_back";
    pub const DEATH_FACE_ANIMATION_ID: &'static str = "death_face";
    pub const PUNCH_ANIMATION_ID: &'static str = "punch";
    pub const RUN_ANIMATION_ID: &'static str = "run";
    pub const HURT_ANIMATION_ID: &'static str = "hurt";

    pub fn new(id:u8, pos: Vec2, rotation: f32,
               vel: Vec2, life: i32, has_ball: bool, color: Color,
               facing_to: FacingTo, animation_player: AnimationPlayer
    ) -> Player {
        Player {
            id,
            pos,
            rotation,
            vel,
            life,
            running: false,
            has_ball,
            color,
            facing_to,
            ducking: false,
            jumping: false,
            ready_to_catch: false,
            camera_box: Default::default(),
            animation_player,
        }
    }

    pub fn throwing(&mut self, ball: &mut Ball) {
        let target_pos = (self.pos - ball.pos).normalize();
        ball.throwing(target_pos, self.pos);
    }
}


impl HasDirection for Player {
    fn get_position(&self) -> Vec2 {
        self.pos
    }
}
