use std::collections::HashMap;
use macroquad::color::Color;
use macroquad::math::Rect;
use crate::{AnimationPlayer, calculate_movement, FacingTo, HasDirection, KeyCode, PlayerAction, valid_position, Vec2};
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
    pub(crate) is_hit: bool,
    pub(crate) catch_grace_time: f64,
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
    pub(crate) fn move_(&mut self, keys_pressed: [bool; 6]) {
        if self.life <= 0 {
            self.animation_player.set_animation(Player::DEATH_BACK_ANIMATION_ID);
            self.animation_player.update();
            return;
        }
        if self.is_hit {
            self.animation_player.set_animation(Player::HURT_ANIMATION_ID);
            self.animation_player.update();
            return;
        }
        self.animation_player.set_animation(Player::IDLE_ANIMATION_ID);
        // mark the keys pressed
        if keys_pressed[4] && keys_pressed.contains(&true) {
            self.animation_player.set_animation(Player::CATCH_ANIMATION_ID);
            self.animation_player.update();
            return;
        }
        if keys_pressed[5] {
            self.animation_player.set_animation(Player::CROUCH_ANIMATION_ID);
            self.animation_player.update();
            return;
        }
        let (rotation, facing_to, acc) = calculate_movement(keys_pressed);
        self.facing_to = facing_to;
        self.rotation = rotation;
        self.vel += acc.unwrap_or(-self.vel);
        if self.vel.length() > 0. {
            self.animation_player.set_animation(Player::MOVE_ANIMATION_ID);
        }
        if self.vel.length() > 5. {
            self.vel = self.vel.normalize() * 5.;
        }
        let prev_pos = self.pos;
        self.pos += self.vel;
        if !valid_position(&self.pos) {
            self.pos = prev_pos;
            self.animation_player.set_animation(Player::HURT_ANIMATION_ID);
        }
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
            ducking: false,
            jumping: false,
            ready_to_catch: false,
            camera_box: Default::default(),
            animation_player,
            catch_grace_time: 0.,
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
