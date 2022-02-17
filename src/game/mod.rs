use macroquad::prelude::*;
use macroquad::prelude::animation::{Animation, AnimatedSprite};
use std::collections::HashMap;
use crate::{AnimationPlayer, DEFAULT_ZOOM, FacingTo, KeyCode, Player, PLAYER_HEIGHT, PLAYER_WIDTH, PlayerAction, Team};
use crate::game::ball::animations::BallAnimationParams;

pub(crate) mod camera;
pub(crate) mod ball;
pub(crate) mod field;
pub mod player;
pub mod has_direction;
pub mod draw_utilities;
pub mod animations;
pub mod resources;
pub mod character;

use crate::game::ball::Ball;
use crate::game::field::Field;

#[derive(Eq, PartialEq)]
pub enum Sideline {
    Top,
    Bottom,
    Back,
    Inside,
}

pub struct Game {
    pub(crate) players: Vec<Player>,
    pub(crate) balls: Vec<AnimationPlayer>,
    pub(crate) ball: Ball,
    pub(crate) team_with_ball: Team,
    pub(crate) field: Field,
    pub(crate) key_sets: HashMap<Team, HashMap<PlayerAction, KeyCode>>,
    pub(crate) gravity: Vec2,
    pub(crate) keys_pressed: Vec<KeyCode>,
    pub(crate) textures: Vec<Texture2D>,
    pub(crate) zoom: Vec2,
}


impl Game {

    pub(crate) fn get_active_player_for_team(&self, which_team: Team) -> Option<usize> {
        let team_one = (0, self.players.len() / 2);
        let team_two = (self.players.len() / 2, self.players.len());
        let target_team_start_index = match which_team {
            Team::One => team_one,
            Team::Two => team_two
        };
        let mut distance = 9999.;
        let mut which_player = None;
        for i in target_team_start_index.0..target_team_start_index.1 {
            let player: &Player = &self.players[i];
            if (self.ball.pos - player.pos).length() < distance {
                distance = (self.ball.pos - player.pos).length();
                which_player = Some(i);
            }
        }
        which_player
    }

    pub(crate) fn is_on_sideline(&self) -> Sideline {
        if self.ball.pos.y < self.field.top_edge {
            Sideline::Top
        } else if self.ball.pos.y > self.field.bottom_edge {
            Sideline::Bottom
        } else if self.ball.pos.x > self.field.right_edge || self.ball.pos.x < self.field.left_edge {
            Sideline::Back
        } else {
            Sideline::Inside
        }
    }

    pub fn which_team_has_ball(&self) -> Team {
        let on_sideline = self.is_on_sideline();
        if self.ball.pos.x > self.field.mid_section {
            if on_sideline != Sideline::Inside {
                Team::One
            } else {
                Team::Two
            }
        } else {
            if on_sideline != Sideline::Inside {
                Team::Two
            } else {
                Team::One
            }
        }
    }

    pub fn default() -> Self {
        Game {
            players: vec![],
            balls: vec![],
            ball: Ball::default(),
            team_with_ball: Team::One,
            field: Field::default(),
            key_sets: HashMap::default(),
            gravity: Default::default(),
            keys_pressed: vec![],
            textures: vec![],
            zoom: Vec2::from(DEFAULT_ZOOM),
        }
    }

    pub fn set_zoom(&mut self, zoom: Option<[f32;2]>) {
        self.zoom = Vec2::from(zoom.unwrap_or(DEFAULT_ZOOM));
    }
}


pub(crate) fn calculate_movement(keys: [bool;6]) -> (f32, FacingTo, Option<Vec2>) {
    let (key_up, key_right, key_down, key_left) = (keys[0], keys[1], keys[2], keys[3]);
    if key_up && key_right {
        (45., FacingTo::FacingTopRight, Some(Vec2::new(1., -1.)))
    } else if key_down && key_right {
        (135., FacingTo::FacingBottomRight, Some(Vec2::new(1., 1.)))
    } else if key_up && key_left {
        (315., FacingTo::FacingTopLeft, Some(Vec2::new(-1., -1.)))
    } else if key_down && key_left {
        (225., FacingTo::FacingBottomLeft, Some(Vec2::new(-1., 1.)))
    } else if key_right {
        (90., FacingTo::FacingRight, Some(Vec2::new(1., 0.)))
    } else if key_left {
        (270., FacingTo::FacingLeft, Some(Vec2::new(-1., 0.)))
    } else if key_down {
        (180., FacingTo::FacingBottom, Some(Vec2::new(0., 1.)))
    } else if key_up {
        (0., FacingTo::FacingTop, Some(Vec2::new(0., -1.)))
    } else {
        (90., FacingTo::FacingRight, None)
    }
}

pub(crate) fn other_team(team: Team) -> Team {
    match team {
        Team::One => { Team::Two }
        Team::Two => { Team::One }
    }
}
