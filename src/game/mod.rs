use macroquad::prelude::*;
use std::collections::HashMap;
use crate::{FacingTo, KeyCode, Player, PLAYER_HEIGHT, PLAYER_WIDTH, PlayerAction, rotation_vector, Team};
pub(crate) mod ball;
pub(crate) mod field;
pub mod player;
pub mod has_direction;
pub mod draw_utilities;

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
    pub(crate) ball: Ball,
    pub(crate) team_with_ball: Team,
    pub(crate) active_chars: HashMap<Team, (usize, f64)>,
    pub(crate) field: Field,
    pub(crate) key_sets: HashMap<Team, HashMap<PlayerAction, KeyCode>>,
    pub(crate) gravity: Vec2,
    pub(crate) keys_pressed: Vec<KeyCode>,
}


impl Game {
    pub(crate) fn get_active_player(&self, which_team: Team, take_ball: bool) -> Option<usize> {
        let team_two = (self.players.len() / 2, self.players.len());
        let team_one = (0, self.players.len() / 2);
        let target_team_start_index = match which_team {
            Team::One => {
                if take_ball { team_one } else { team_two}
            }
            Team::Two => {
                if take_ball { team_two } else { team_one}
            }
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

    pub fn new() -> Self {
        let field = Field::default();
        let game = Game {
            players: vec![
                Player {
                    pos: Vec2::new(field.mid_section - PLAYER_WIDTH - 80., screen_height() / 2.),
                    rotation: 90.,
                    vel: Vec2::new(0., 0.),
                    life: 100,
                    has_ball: true,
                    color: BLACK,
                    facing_to: FacingTo::FacingRight,
                    ducking: false,
                    jumping: false,
                },
                Player {
                    pos: Vec2::new(field.left_edge + PLAYER_WIDTH + 80., screen_height() / 2.),
                    rotation: 90.,
                    vel: Vec2::new(0., 0.),
                    life: 100,
                    has_ball: false,
                    color: Color::new(10., 233., 134., 1.),
                    facing_to: FacingTo::FacingRight,
                    ducking: false,
                    jumping: false,
                },
                Player {
                    pos: Vec2::new(field.mid_section + 80., screen_height() / 2.),
                    rotation: -90.,
                    vel: Vec2::new(0., 0.),
                    life: 100,
                    has_ball: false,
                    color: DARKGRAY,
                    facing_to: FacingTo::FacingLeft,
                    ducking: false,
                    jumping: false,
                },
                Player {
                    pos: Vec2::new(field.right_edge - 80., screen_height() / 2.),
                    rotation: -90.,
                    vel: Vec2::new(0., 0.),
                    life: 100,
                    has_ball: false,
                    color: Color::new(0., 10., 10., 1.),
                    facing_to: FacingTo::FacingLeft,
                    ducking: false,
                    jumping: false,
                },
            ],
            ball: Ball::default(),
            team_with_ball: Team::One,
            active_chars: HashMap::from([(Team::One, (0, 0.0)), (Team::Two, (2, 0.0))]),
            field,
            key_sets: HashMap::default(),
            gravity: Default::default(),
            keys_pressed: vec![]
        };
        game
    }
}

pub fn new_game() -> Game {
    let mut game = Game::new();
    let rot_vec = rotation_vector(&game.players[0]);
    game.ball = Ball {
        pos: game.players[0].pos + rot_vec * PLAYER_HEIGHT,
        vel: Vec2::default(),
        r: 10.,
        rotation: game.players[0].rotation,
        collided: false,
        thrown: false,
        dropped: false,
        color: BLACK,
        in_air: false,
        grabbed_by: Some(0),
    };
    game.active_chars = HashMap::from([(Team::One, (0, 0.0)), (Team::Two, (game.players.len() / 2, 0.0))]);
    game.gravity = Vec2::new(-2., -2.);
    game
}
