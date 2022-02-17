use macroquad::color::Color;
use crate::game::ball::animations::BallAnimationParams;
use crate::{AnimationPlayer, Player, Vec2};
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
}


impl Ball {
    pub const IDLE_ANIMATION_ID: &'static str = "idle";
    pub const MOVE_ANIMATION_ID: &'static str = "move";

    pub(crate) fn update_velocity_on_collision(&mut self, change_x: bool, change_y: bool) {
        self.collided = true;
        self.dropped = true;
        // TODO will have to consider gravity
        if change_y {
            self.vel.y *= -1.;
        }
        if change_x {
            self.vel.x *= -1.;
        }
        self.vel -= self.vel / 10.;
    }

    pub(crate) fn throw(&mut self) {
        self.pos += self.vel;
        if self.vel.length() > 5. {
            self.vel = self.vel.normalize() * 5.;
        }
        if self.collided {
            self.vel -= self.vel / 5.;
        }
        if self.vel.length() < 0.1 {
            self.stopping();
        }
    }

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
    }

    #[inline]
    pub(crate) fn throwing(&mut self, target_pos: Vec2, thrower_position: Vec2) {
        self.pos = thrower_position;
        self.vel = target_pos * 5.;
        self.thrown = true;
        self.collided = false;
        self.in_air = true;
        self.grabbed_by = None;
        self.stopped = false;
    }

    #[inline]
    fn stopping(&mut self) {
        self.collided = false;
        self.stopped = true;
        self.in_air = false;
        self.vel = Vec2::default();
        self.grabbed_by = None;
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