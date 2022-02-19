use macroquad::prelude::*;
use macroquad::prelude::animation::{Animation, AnimatedSprite};
use std::collections::HashMap;
use crate::{AnimationPlayer, colliding_with, DEFAULT_ZOOM, FacingTo, KeyCode, Player, PLAYER_HEIGHT, PLAYER_WIDTH, PlayerAction, Team};
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

pub enum GameState {
    DroppedBall,
    BallHitsPlayer,
    PlayerCatchingBall
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
    pub(crate) fn on_player_doing_things_with_ball(&mut self, frame_t: f64) {
        // check which team has the ball
        let team_with_ball = self.which_team_has_ball();
        for i in 0..self.players.len() {
            {
                let player: &mut Player = &mut self.players[i];
                let (collided, change_x, change_y) = colliding_with(&self.ball.pos, self.ball.r, &player);
                if !collided { continue; }
                if player.life <= 0 { continue; }
                // TODO Find which direction the ball is coming from
                // TODO After that check if the direction key is pressed
                // TODO Also check if `B` Button is pressed or not
                // TODO Only then the player can pick catch the ball
                // TODO must take account of how long the key pressed, if it's more than the threshold, stop catching
                let option = self.key_sets[&team_with_ball].get(&PlayerAction::B).unwrap();
                if is_key_down(*option) {
                    player.catching(frame_t);
                }
                if is_key_released(*option) {
                    player.not_catching();
                }
                // TODO: must break down the following ugly conditions
                // FIXME: When I am checking if B button is pressed and setting ready_to_catch, the player is catching the ball, but we also need the direction key to be pressed
                if player.ready_to_catch {
                    self.ball.picked_up(i);
                    player.ready_to_catch = false;
                    player.has_ball = true;
                } else {
                    if !self.ball.stopped {
                        if self.ball.thrown && player.is_hit == false {
                            // TODO: Damage will be based on ball's velocity and distance covered
                            player.life -= 10;
                            player.is_hit = true;
                        }
                    } else {
                        player.is_hit = false;
                    }
                    self.ball.update_velocity_on_collision(change_x, change_y);
                }
            }
            self.set_zoom(None);
        }
    }
}

impl Game {
    pub(crate) fn on_player_threw_ball(&mut self) {
        self.ball.throw();
        self.set_zoom(Some([-0.0035, 0.0035]));
    }
}

impl Game {
    pub(crate) fn on_ball_hitting_player(&mut self) {
        self.ball.stop();
        self.set_zoom(None);
    }
}

impl Game {
    pub(crate) fn on_player_catching_ball(&mut self) {
        let team_with_ball = self.which_team_has_ball();
        if !self.ball.thrown && self.ball.grabbed_by.is_some() {
            match team_with_ball {
                Team::One => {
                    self.ball.pos = self.players[self.ball.grabbed_by.unwrap()].pos + Vec2::new(PLAYER_WIDTH + self.ball.r + 10., 0.);
                }
                Team::Two => {
                    self.ball.pos = self.players[self.ball.grabbed_by.unwrap()].pos - Vec2::new(self.ball.r + 10., 0.);
                }
            }
        }
    }
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
        (45., FacingTo::FacingRight, Some(Vec2::new(1., -1.)))
    } else if key_down && key_right {
        (135., FacingTo::FacingRight, Some(Vec2::new(1., 1.)))
    } else if key_up && key_left {
        (315., FacingTo::FacingLeft, Some(Vec2::new(-1., -1.)))
    } else if key_down && key_left {
        (225., FacingTo::FacingLeft, Some(Vec2::new(-1., 1.)))
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
