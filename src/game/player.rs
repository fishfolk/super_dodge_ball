use std::collections::HashMap;
use macroquad::color::Color;
use macroquad::math::Rect;
use macroquad_platformer::Actor;
use crate::{AnimationPlayer, calculate_movement, FacingTo, HasDirection, KeyCode, PlayerAction, valid_position, Vec2};
use crate::game::ball::Ball;
use crate::game::character::PlayerCharacterParams;
use crate::game::player::PlayerState::Walking;

#[derive(PartialEq)]
pub enum PlayerState {
    Idle,
    Walking,
    Jumping,
    Ducking,
    Running,
    Catching,
    Hurting,
    Died,
    Throwing,
    Passing,
}

pub struct Player {
    pub(crate) id: u8,
    pub(crate) pos: Vec2,
    pub(crate) rotation: f32,
    pub(crate) vel: Vec2,
    pub(crate) color: Color,
    pub(crate) facing_to: FacingTo,
    pub(crate) facing_to_before: FacingTo,
    pub(crate) ducking: bool,
    pub(crate) jumping: bool,
    pub(crate) has_ball: bool,
    pub(crate) life: i32,
    pub(crate) running: bool,
    pub(crate) ready_to_catch: bool,
    pub(crate) camera_box: Rect,
    pub(crate) animation_player: AnimationPlayer,
    pub(crate) is_hit: bool,
    pub(crate) catch_grace_time: f64,
    pub(crate) state: PlayerState,

}

impl Player {
    pub(crate) fn set_animation(&mut self) {
        let state = match self.state {
            PlayerState::Idle => Player::IDLE_ANIMATION_ID,
            PlayerState::Walking => Player::MOVE_ANIMATION_ID,
            PlayerState::Jumping => Player::JUMP_ANIMATION_ID,
            PlayerState::Ducking => Player::CROUCH_ANIMATION_ID,
            PlayerState::Running => Player::RUN_ANIMATION_ID,
            PlayerState::Catching => Player::CATCH_ANIMATION_ID,
            PlayerState::Hurting => Player::HURT_ANIMATION_ID,
            PlayerState::Died => Player::DEATH_BACK_ANIMATION_ID,
            PlayerState::Throwing => Player::CATCH_ANIMATION_ID,
            PlayerState::Passing => Player::CATCH_ANIMATION_ID,
        };
        self.animation_player.set_animation(state);
        self.animation_player.update();
    }

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
    pub const CATCH_ANIMATION_ID: &'static str = "catching";

    pub const CATCH_GRACE_TIME: f64 = 5.;

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
            is_hit: false,
            has_ball,
            color,
            facing_to,
            facing_to_before: FacingTo::FacingLeft,
            ducking: false,
            jumping: false,
            ready_to_catch: false,
            camera_box: Default::default(),
            animation_player,
            catch_grace_time: 0.,
            state: PlayerState::Idle,
        }
    }

    pub fn throwing(&mut self, ball: &mut Ball) {
        let target_pos = (self.pos - ball.pos).normalize();
        ball.throwing(target_pos, self.pos, self.facing_to_before.clone());
    }
}


impl HasDirection for Player {
    fn get_position(&self) -> Vec2 {
        self.pos
    }
}

