use macroquad::prelude::*;
use std::collections::HashMap;
use crate::{Ball, FacingTo, KeyCode, Player, PLAYER_HEIGHT, PLAYER_WIDTH, PlayerAction, rotation_vector, Team};
use crate::field::Field;

pub struct Game {
    pub(crate) players: Vec<Player>,
    pub(crate) ball: Ball,
    pub(crate) last_shot: f64,
    pub(crate) team_with_ball: Team,
    pub(crate) active_chars: HashMap<Team, (usize, f64)>,
    pub(crate) field: Field,
    pub(crate) key_sets: HashMap<Team, HashMap<PlayerAction, KeyCode>>,
}


impl Game {
    pub(crate) fn mark_active_player(&mut self, which_team: Team, frame_t: f64) {
        let target_team_start_index = match which_team {
            Team::One => {
                (0, self.players.len() / 2)
            }
            Team::Two => {
                (self.players.len() / 2, self.players.len())
            }
        };
        let mut distance = 9999.;
        let mut which_player = None;
        let active_player = self.active_chars.get(&which_team).unwrap();
        println!("{:?} {} {}", active_player, frame_t - active_player.1, 0.016 * 3.);
        for i in target_team_start_index.0..target_team_start_index.1 {
            let player: &Player = &self.players[i];
            if i == active_player.0 && frame_t - active_player.1 < 0.016 * 3. { continue; }
            if (self.ball.pos - player.pos).length() < distance {
                distance = (self.ball.pos - player.pos).length();
                which_player = Some(i);
            }
        }
        self.active_chars.insert(which_team, (which_player.unwrap(), frame_t));
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
            ball: Ball {
                pos: Default::default(),
                vel: Default::default(),
                r: 0.0,
                rotation: 0.0,
                color: Default::default(),
                shot_at: 0.0,
                collided: false,
            },
            last_shot: get_time(),
            team_with_ball: Team::One,
            active_chars: HashMap::from([(Team::One, (0, 0.0)), (Team::Two, (2, 0.0))]),
            field,
            key_sets: HashMap::from([(
                Team::One, HashMap::from([
                    (PlayerAction::MoveUp, KeyCode::W),
                    (PlayerAction::MoveLeft, KeyCode::A),
                    (PlayerAction::MoveDown, KeyCode::S),
                    (PlayerAction::MoveRight, KeyCode::D),
                    (PlayerAction::A, KeyCode::N),
                    (PlayerAction::B, KeyCode::M),
                ])),
                (Team::Two, HashMap::from([
                    (PlayerAction::MoveUp, KeyCode::I),
                    (PlayerAction::MoveLeft, KeyCode::J),
                    (PlayerAction::MoveDown, KeyCode::K),
                    (PlayerAction::MoveRight, KeyCode::L),
                    (PlayerAction::A, KeyCode::Z),
                    (PlayerAction::B, KeyCode::X),
                ]))
            ]),
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
        shot_at: 0.,
        collided: false,
        color: DARKBROWN,
    };
    game.active_chars = HashMap::from([(Team::One, (0, 0.0)), (Team::Two, (game.players.len() / 2, 0.0))]);
    game
}
