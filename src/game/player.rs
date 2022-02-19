use std::collections::HashMap;
use macroquad::color::Color;
use macroquad::math::Rect;
use crate::{AnimationPlayer, calculate_movement, FacingTo, HasDirection, KeyCode, PlayerAction, valid_position, Vec2};
use crate::game::ball::Ball;
use crate::game::character::PlayerCharacterParams;
use crate::game::player::PlayerState::Walking;

pub enum PlayerState {
    Idle,
    Walking,
    Jumping,
    Ducking,
    Running,
    Catching,
    Hurting,
    Died,
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
    pub(crate) fn not_catching(&mut self) {
        self.ready_to_catch = false;
        self.catch_grace_time = 0.;
    }
}

impl Player {
    pub fn catching(&mut self, frame_t: f64) {
        if !self.ready_to_catch {
            self.ready_to_catch = true;
            self.catch_grace_time = frame_t;
        } else {
            self.catch_grace_time += frame_t;
        }
        if self.catch_grace_time > Player::CATCH_GRACE_TIME {
            self.ready_to_catch = false;
            self.catch_grace_time = 0.
        }
    }
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
        };
        self.animation_player.set_animation(state);
        self.animation_player.update();
    }

    pub(crate) fn update_player_state(&mut self, keys_pressed: [bool; 6]) {
        if self.life <= 0 {
            self.state = PlayerState::Died;
            return;
        }
        if self.is_hit {
            self.state = PlayerState::Hurting;
            return;
        }
        if keys_pressed[4] && keys_pressed.contains(&true) {
            self.state = PlayerState::Catching;
            return;
        }
        if keys_pressed[5] {
            self.state = PlayerState::Ducking;
            return;
        }
        let (rotation, facing_to, acc) = calculate_movement(keys_pressed);
        if acc.is_some() {
            self.facing_to = facing_to;
            if self.facing_to == FacingTo::FacingRight || self.facing_to == FacingTo::FacingLeft {
                self.facing_to_before = self.facing_to.clone();
            }
            self.rotation = rotation;
            self.vel += acc.unwrap_or(-self.vel);
            if self.vel.length() > 0. {
                self.state = PlayerState::Walking;
            }
            if self.vel.length() > 5. {
                self.vel = self.vel.normalize() * 5.;
            }
            let prev_pos = self.pos;
            self.pos += self.vel;
            if !valid_position(&self.pos) {
                self.state = PlayerState::Idle;
                self.pos = prev_pos;
                self.vel = Vec2::ZERO;
            }
        } else {
            self.state = PlayerState::Idle;
            self.vel = Vec2::ZERO;
        }
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
        ball.throwing(target_pos, self.pos);
    }
}


impl HasDirection for Player {
    fn get_position(&self) -> Vec2 {
        self.pos
    }
}

enum PlayerStates {
    Idle,
    Moving,
    Running,
    Catching,
    Ducking
}
