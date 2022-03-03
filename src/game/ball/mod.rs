use macroquad::color::Color;
use crate::game::ball::animations::BallAnimationParams;
use crate::{AnimationPlayer, FacingTo, Player, Vec2};
use serde::{Deserialize, Serialize};

pub mod animations;

#[derive(Clone, Debug)]
pub struct Ball {
    pub(crate) pos: Vec2,
    pub(crate) vel: Vec2,
    pub(crate) r: f32,
    pub(crate) color: Color,
    pub(crate) collided: bool,
    pub(crate) thrown: bool,
    pub(crate) dropped: bool,
    pub(crate) in_air: bool,
    pub(crate) grabbed_by: Option<usize>,
    pub(crate) animation: usize,
    pub(crate) stopped: bool,
    pub(crate) state: BallState,
    pub(crate) tick: f64,
}

impl Ball {
    pub(crate) fn outside_edge(&mut self, time_passed: f64, left_or_right_edge: bool, top_or_bottom_edge: bool) {
        self.state = BallState::AfterHittingBoundary { time_passed };
    }
}


impl Ball {
    pub const IDLE_ANIMATION_ID: &'static str = "idle";
    pub const MOVE_ANIMATION_ID: &'static str = "move";

    pub(crate) fn default() -> Ball {
        Ball {
            pos: Default::default(),
            vel: Default::default(),
            r: 0.0,
            color: Default::default(),
            collided: false,
            thrown: false,
            dropped: false,
            in_air: false,
            grabbed_by: Some(0),
            animation: 0,
            stopped: true,
            state: BallState::OnPlayersHand(0),
            tick: 0.0,
        }
    }

    // ball's state updates
    #[inline]
    pub(crate) fn picked_up(&mut self, player_index: usize) {
        self.grabbed_by = Some(player_index);
        self.collided = false;
        self.thrown = false;
        self.dropped = false;
        self.in_air = false;
        self.stopped = true;
        self.vel = Vec2::new(0., 0.);
        self.state = BallState::OnPlayersHand(player_index);
    }

    #[inline]
    pub(crate) fn throwing(&mut self, target_pos: Vec2, thrower_position: Vec2, facing_to: FacingTo) {
        self.pos = thrower_position;
        self.vel = target_pos * 5.;
        self.thrown = true;
        self.collided = false;
        self.in_air = true;
        self.grabbed_by = None;
        self.stopped = false;
        self.state = BallState::OnAir(facing_to);
    }

    #[inline]
    fn stopping(&mut self) {
        self.collided = false;
        self.stopped = true;
        self.in_air = false;
        self.vel = Vec2::default();
        self.grabbed_by = None;
        self.state = BallState::OnGround;
    }

    pub(crate) fn ball_falling(&mut self, time_passed: f64, gravity: Vec2, gravity_line: f32) {
        // dt = t' - t
        // pos' = pos + dt * v
        // v' = v + dt * G
        let dt = (time_passed - self.tick) as f32;
        self.tick = time_passed;
        self.vel -= self.vel / 5.;
        self.vel += gravity * dt;
        if self.pos.y >= gravity_line {
            self.vel.y = -self.vel.y;
        }
        self.pos += self.vel;
        if self.vel.length() < 0.1 {
            self.stopping();
        }
    }

    pub(crate) fn after_collision(&mut self, change_x: bool, change_y: bool, time_passed: f64) {
        self.collided = true;
        self.dropped = true;
        self.tick = time_passed;
        if change_y {
            self.vel.y *= -1.;
        }
        if change_x {
            self.vel.x *= -1.;
        }
        self.state = BallState::BallFalling {time_passed}
    }

    pub(crate) fn move_ball(&mut self) {
        if self.vel.length() > 5. {
            self.vel = self.vel.normalize() * 5.;
        }
        self.pos += self.vel;
    }

    pub(crate) fn stop(&mut self) {
        self.vel -= self.vel / 5.;
        if self.vel.length() < 0.1 {
            self.stopping();
        }
    }
}

impl crate::HasDirection for Ball {
    fn get_position(&self) -> Vec2 {
        self.pos
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BallParams {
    /// This is the id of the player character. This should be unique, or it will either overwrite
    /// or be overwritten, depending on load order, if not.
    pub id: String,
    /// This is the name of the player character, as shown in character selection
    pub name: String,
    /// This is the description for the player character, as shown in character selection
    #[serde(default)]
    pub description: String,
    /// This holds the animation and sprite parameters for the player character. This is flattened,
    /// meaning that, in JSON, you will declare the members of this struct directly in the
    /// `PlayerCharacterParams` entry.
    #[serde(flatten)]
    pub animation: BallAnimationParams,
}

#[derive(Clone, Debug, PartialEq)]
pub enum BallState {
    OnGround,
    OnAir(FacingTo),
    OnPlayersHand(usize),
    AfterHittingPlayer { change_x: bool, change_y: bool, time_passed: f64 },
    AfterHittingBoundary { time_passed: f64 },
    BallFalling { time_passed: f64 },
    Stopping,
}