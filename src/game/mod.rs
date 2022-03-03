use macroquad::prelude::*;
use macroquad::prelude::animation::{Animation, AnimatedSprite};
use std::collections::HashMap;
use macroquad::ui::Drag::No;
use macroquad_platformer::World;
use crate::{_x, _y, AnimationPlayer, BallState, colliding_with, DEFAULT_ZOOM, FacingTo, KeyCode, Player, PLAYER_HEIGHT, PLAYER_WIDTH, PlayerAction, PlayerState, Team, valid_position};
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

pub trait GameStateTrait {
    fn enter();
    fn exit();
    fn process();
}

pub enum GameState {
    BallOnGround,
    BallHitsPlayer(usize),
    PlayerCatchingBall(usize),
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
    pub(crate) time_passed: f64,
    pub(crate) gravity_line: f32,
    pub(crate) world: World,
}

impl Game {
    pub(crate) fn update_ball_state(&mut self) {
        match self.ball.state {
            BallState::OnGround => {
                //self.set_zoom(None);
                self.balls[self.ball.animation].set_animation(Ball::IDLE_ANIMATION_ID);
            }
            BallState::OnAir(facing_to) => {
                // check if hitting any players
                self.balls[self.ball.animation].set_animation(Ball::MOVE_ANIMATION_ID);
            }
            BallState::OnPlayersHand(player_index) => {
                self.balls[self.ball.animation].set_animation(Ball::IDLE_ANIMATION_ID);
            }
            BallState::AfterHittingPlayer { change_x, change_y, time_passed } => {
                self.balls[self.ball.animation].set_animation(Ball::MOVE_ANIMATION_ID);
                self.gravity_line = self.ball.pos.y + PLAYER_HEIGHT;
                self.ball.after_collision(change_x, change_y, time_passed)
            }
            BallState::AfterHittingBoundary { time_passed } => {
                self.balls[self.ball.animation].set_animation(Ball::MOVE_ANIMATION_ID);
                self.gravity_line = self.ball.pos.y + PLAYER_HEIGHT;
                self.ball.after_collision(true, true, time_passed)
            }
            BallState::Stopping => {
                self.balls[self.ball.animation].set_animation(Ball::IDLE_ANIMATION_ID);
                self.ball.stop()
            }
            BallState::BallFalling { time_passed } => {
                self.balls[self.ball.animation].set_animation(Ball::MOVE_ANIMATION_ID);
                self.ball.ball_falling(time_passed, self.gravity, self.gravity_line)
            }
        }
    }

    pub fn attach_ball_to_player(&mut self, player_index: usize) {
        let team_with_ball = self.which_team_has_ball();
        match team_with_ball {
            Team::One => {
                self.ball.pos = self.players[player_index].pos + Vec2::new(PLAYER_WIDTH, 0.);
            }
            Team::Two => {
                self.ball.pos = self.players[player_index].pos - Vec2::new(self.ball.r + 10., 0.);
            }
        }
    }

    pub fn is_ball_hitting_boundary(&mut self) {
        // check if hitting the borders
        let b_pos = &self.ball.pos;
        let outside_top_or_bottom_edge = b_pos.y + self.ball.r < self.field.top_edge + 10. || b_pos.y + self.ball.r > self.field.bottom_edge - 10.;
        let outside_left_or_right_edge = b_pos.x + self.ball.r > self.field.right_edge - 10. || b_pos.x + self.ball.r < self.field.left_edge + 10.;
        if outside_left_or_right_edge || outside_top_or_bottom_edge {
            self.ball.outside_edge(self.time_passed, outside_left_or_right_edge, outside_top_or_bottom_edge);
        }
    }

    pub fn is_the_ball_hitting_any_player(&mut self) {
        for i in 0..self.players.len() {
            let player: &mut Player = &mut self.players[i];
            let (collided, change_x, change_y) = colliding_with(
                &self.ball.pos, self.ball.r, &player.pos,
                &Vec2::new(
                    PLAYER_WIDTH - self.ball.r * 2.,
                    PLAYER_HEIGHT - self.ball.r * 2.)
            );
            if collided && player.state == PlayerState::Catching {
                self.ball.picked_up(i);
            } else if collided {
                player.state = PlayerState::Hurting;
                self.ball.state = BallState::AfterHittingPlayer { time_passed: self.time_passed, change_x, change_y };
            }
        }
    }

    pub fn update_player(&mut self, player_index: usize) {
        let current_team = if player_index >= self.players.len() / 2 { Team::Two } else { Team::One };
        let active_player = self.get_active_player_for_team(current_team);
        if Some(player_index) != active_player { return; }
        let target_pos = self.find_target_pos(&current_team);
        let keys = self.key_sets.get(&current_team).unwrap();
        let keys_pressed = [
            is_key_down(keys[&PlayerAction::MoveUp]),
            is_key_down(keys[&PlayerAction::MoveRight]),
            is_key_down(keys[&PlayerAction::MoveDown]),
            is_key_down(keys[&PlayerAction::MoveLeft]),
            is_key_down(keys[&PlayerAction::B]),
            is_key_down(keys[&PlayerAction::A])
        ];
        {
            let player: &mut Player = &mut self.players[player_index];
            if player.life <= 0 {
                player.state = PlayerState::Died;
                return;
            }
            if keys_pressed[4] {
                if self.ball.state == BallState::OnPlayersHand(player_index) {
                    self.ball.throwing(target_pos, self.ball.pos, player.facing_to.clone());
                    player.state = PlayerState::Throwing;
                } else {
                    let (facing_to, player_action) = FacingTo::opposite_direction(player.facing_to);
                    let key_code = keys.get(&player_action).unwrap();
                    if self.ball.state == BallState::OnAir(facing_to) && is_key_down(*key_code){
                        player.state = PlayerState::Catching;
                        // code to handle catching of ball
                    } else if self.ball.state == BallState::OnGround {
                        player.state = PlayerState::Catching;
                    }
                }
            } else if keys_pressed[5] {
                player.state = PlayerState::Ducking;
            } else {
                let (rotation, facing_to, acc) = calculate_movement([keys_pressed[0], keys_pressed[1], keys_pressed[2], keys_pressed[3]]);
                if acc.is_some() {
                    player.facing_to = facing_to;
                    if player.facing_to == FacingTo::FacingRight || player.facing_to == FacingTo::FacingLeft {
                        player.facing_to_before = player.facing_to.clone();
                    }
                    player.rotation = rotation;
                    player.vel += acc.unwrap_or(-player.vel);
                    if player.vel.length() > 0. {
                        player.state = PlayerState::Walking;
                    }
                    if player.vel.length() > 5. {
                        player.vel = player.vel.normalize() * 5.;
                    }
                    let prev_pos = player.pos;
                    player.pos += player.vel;
                    if !valid_position(&player.pos) {
                        player.state = PlayerState::Idle;
                        player.pos = prev_pos;
                        player.vel = Vec2::ZERO;
                    }
                } else {
                    player.state = PlayerState::Idle;
                    player.vel = Vec2::ZERO;
                }
            }
            player.set_animation();
        }
    }

    fn find_target_pos(&self, &current_team: &Team) -> Vec2 {
        let other_team = other_team(current_team);
        let target_player_index = self.get_active_player_for_team(other_team).unwrap();
        let pos = self.ball.pos;
        let target = self.players[target_player_index].pos;
        (target - pos).normalize()
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
            gravity: Vec2::new(-2., -2.),
            keys_pressed: vec![],
            textures: vec![],
            zoom: Vec2::from(DEFAULT_ZOOM),
            time_passed: get_time(),
            gravity_line: screen_height() / 2.,
            world: World::new(),
        }
    }

    pub fn set_zoom(&mut self, zoom: Option<[f32; 2]>) {
        self.zoom = Vec2::from(zoom.unwrap_or(DEFAULT_ZOOM));
    }
}


pub(crate) fn calculate_movement(keys: [bool; 4]) -> (f32, FacingTo, Option<Vec2>) {
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

