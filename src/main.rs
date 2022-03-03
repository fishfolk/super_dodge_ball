use std::collections::HashMap;
use std::ops::Deref;
use macroquad::prelude::*;
use macroquad::experimental::animation::{Animation, AnimatedSprite};
use macroquad::prelude::collections::storage;
use macroquad::telemetry::frame;
use game::has_direction;
use game::draw_utilities::{draw_line_a, draw_rectangle_lines_a};
use crate::game::{calculate_movement, Game, other_team};
use game::has_direction::{HasDirection};
use game::player::Player;
use crate::game::animations::{AnimationParams, AnimationPlayer};
use crate::game::ball::animations::BallAnimations;
use crate::game::ball::{Ball, BallState};
use crate::game::character::PlayerCharacterParams;
use crate::game::player::PlayerState;
use crate::game::resources::{load_resources, Resources};
use crate::json::is_false;

pub mod helpers;
pub mod error;
pub mod json;
pub mod game;
pub mod math;
pub mod noise;

const PLAYER_HEIGHT: f32 = 54.;
const PLAYER_WIDTH: f32 = 54.;
const RESET_KEY: usize = 12;

const TEAM_ONE_PLAYER: usize = 1;
const TEAM_TWO_PLAYER: usize = 2;
const TEAM_ONE_PLAYER_READY: usize = 3;
const TEAM_TWO_PLAYER_READY: usize = 3;
const PLAYER_ANIMATED_TEXTURES: usize = 4;
const DEFAULT_ZOOM: [f32; 2] = [-0.004, 0.004];

fn _x<T: HasDirection>(p: &T) -> f32 {
    p.get_position().x
}

fn _y<T: HasDirection>(p: &T) -> f32 {
    p.get_position().y
}

#[derive(Eq, PartialEq, Hash, Debug)]
pub enum PlayerAction {
    A,
    B,
    MoveLeft,
    MoveRight,
    MoveUp,
    MoveDown,
}

enum MovingStates {
    MovingLeft,
    MovingRight,
    MovingUp,
    MovingDown,
}

#[derive(Eq, PartialEq, Debug, Hash, Clone, Copy)]
pub enum Team {
    One,
    Two,
}


#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum FacingTo {
    FacingTop,
    FacingBottom,
    FacingRight,
    FacingLeft,
    FacingTopLeft,
    FacingTopRight,
    FacingBottomRight,
    FacingBottomLeft,
}

impl FacingTo {
    pub fn opposite_direction(dir: FacingTo) -> (FacingTo, PlayerAction) {
        match dir {
            FacingTo::FacingTop => (FacingTo::FacingBottom, PlayerAction::MoveDown),
            FacingTo::FacingBottom => (FacingTo::FacingTop, PlayerAction::MoveUp),
            FacingTo::FacingRight => (FacingTo::FacingLeft, PlayerAction::MoveLeft),
            FacingTo::FacingLeft => (FacingTo::FacingRight, PlayerAction::MoveRight),
            FacingTo::FacingTopLeft => (FacingTo::FacingRight, PlayerAction::MoveRight),
            FacingTo::FacingTopRight => (FacingTo::FacingLeft, PlayerAction::MoveLeft),
            FacingTo::FacingBottomRight => (FacingTo::FacingLeft, PlayerAction::MoveLeft),
            FacingTo::FacingBottomLeft => (FacingTo::FacingRight, PlayerAction::MoveRight)
        }
    }

}

fn calculate_life_color(life: i32) -> Color {
    if life > 70 {
        DARKGREEN
    } else if life > 40 && life < 70 {
        YELLOW
    } else {
        RED
    }
}

fn valid_position(v: &Vec2) -> bool {
    if v.x + PLAYER_HEIGHT > screen_width() {
        return false;
    }
    if v.x < 0. {
        return false;
    }
    if v.y + PLAYER_HEIGHT > screen_height() {
        return false;
    }
    if v.y < 0. {
        return false;
    }
    true
}

fn colliding_with(pos: &Vec2, r: f32, rect_pos: &Vec2, rect_size: &Vec2) -> (bool, bool, bool) {
    // temporary variables to set edges for testing
    let mut test_x = pos.x;
    let mut test_y = pos.y;
    // which edge is closest?
    let change_x = if pos.x < rect_pos.x {
        test_x = rect_pos.x;      // test left edge
        true
    } else if pos.x > rect_pos.x + rect_size.x {
        test_x = rect_pos.x + rect_size.x;   // right edge
        true
    } else {
        false
    };
    //
    let change_y = if pos.y < rect_pos.y {
        test_y = rect_pos.y;      // top edge
        true
    } else if pos.y > rect_pos.y + rect_size.y {
        test_y = rect_pos.y + rect_size.y;   // bottom edge
        true
    } else {
        false
    };
    // get distance from closest edges
    let dist_x = pos.x - test_x;
    let dist_y = pos.y - test_y;
    let distance = ((dist_x * dist_x) + (dist_y * dist_y)).sqrt();
    // if the distance is less than the radius, collision!
    (distance <= r, change_x, change_y)
}


fn throw_ball(game: &mut Game, player_index: usize, team_with_ball: &Team) -> Option<usize> {
    if !is_key_pressed(
        *game.key_sets.get(&team_with_ball)
            .unwrap().get(&PlayerAction::B)
            .unwrap()
    ) {
        return None;
    }
    let other_team = other_team(*team_with_ball);
    // find the opposite player
    let target_player_index = game.get_active_player_for_team(other_team).unwrap();
    let pos = game.ball.pos;
    let target = game.players[target_player_index].pos;
    let target_pos = (target - pos).normalize();
    game.ball.throwing(target_pos, pos, game.players[player_index].facing_to_before.clone());
    game.players[player_index].has_ball = false;
    game.balls[game.ball.animation].set_animation(Ball::MOVE_ANIMATION_ID);
    game.balls[game.ball.animation].update();
    Some(target_player_index)
}

async fn local_game() {
    // this should allow us to remap keys later
    let keys_mapped = vec![
        KeyCode::W, KeyCode::A, KeyCode::S, KeyCode::D, KeyCode::N, KeyCode::M,
        KeyCode::I, KeyCode::J, KeyCode::K, KeyCode::L, KeyCode::Z, KeyCode::X,
        KeyCode::Enter,
    ];
    let mut game = new_game(&keys_mapped).await;
    loop {
        game.time_passed = get_time();
        if is_key_pressed(keys_mapped[RESET_KEY]) {
            game = new_game(&keys_mapped).await;
            game.set_zoom(None);
            next_frame().await;
            continue;
        }
        game.update_ball_state();
        // move player code outside, AI player will be separated
        for i in 0..game.players.len() {
            game.update_player(i);
        }
        game.ball.move_ball();
        game.is_the_ball_hitting_any_player();
        game.is_ball_hitting_boundary();
        debug_ball_throwing(&mut game);
        //
        //
        // Drawing stuffs
        //
        //
        match mouse_wheel() {
            (_x, y) if y != 0.0 => {
                let mut zoom = game.zoom;
                zoom *= 1.1f32.powf(y);
                game.set_zoom(Some([zoom.x, zoom.y]));
            }
            _ => (),
        }
        clear_background(LIGHTGRAY);
        draw_field(&game);
        // set_camera(&Camera2D { target: game.ball.pos, rotation: 180., zoom: game.zoom, ..Default::default() });
        debug_collision(&game);
        game.balls[game.ball.animation].update();
        let bx = _x(&game.ball);
        let by = _y(&game.ball);
        draw_rectangle_lines_a(game.ball.pos, game.ball.r, game.ball.r , 2., BLACK);
        game.balls[game.ball.animation].draw(Vec2::new(bx, by), 0., false, false);

        for (i, player) in game.players.iter().enumerate() {
            let txt = format!("{}", player.life);
            draw_text(&txt, player.pos.x, player.pos.y - 20., 20.0, calculate_life_color(player.life));
            let flip_x = should_face_to(
                player.facing_to.clone(),
                if i >= game.players.len() { Team::Two } else { Team::Two },
                player.facing_to_before.clone(),
            );
            // draw_rectangle_lines_a(player.pos, PLAYER_WIDTH, PLAYER_HEIGHT, 2., BLACK);
            player.animation_player.draw(
                player.pos, 0., flip_x, false,
            );
        }
        next_frame().await
    }
}

fn check_for_collision(player: &mut Player, ball: &Ball) {
    let (collided, change_x, change_y) = colliding_with(&ball.pos, ball.r,
                                                        &player.pos,
                                                        &Vec2::new(PLAYER_WIDTH, PLAYER_HEIGHT),
    );
}

fn should_face_to(facing_to: FacingTo, which_team: Team, facing_to_before: FacingTo) -> bool {
    let which = || {
        if facing_to_before == FacingTo::FacingLeft {
            if which_team == Team::One { false } else { true }
        } else {
            if which_team == Team::One { true } else { false }
        }
    };
    match facing_to {
        FacingTo::FacingTop => which(),
        FacingTo::FacingBottom => which(),
        FacingTo::FacingRight => false,
        FacingTo::FacingLeft => true,
        FacingTo::FacingTopLeft => true,
        FacingTo::FacingTopRight => false,
        FacingTo::FacingBottomRight => false,
        FacingTo::FacingBottomLeft => true,
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    load_resources("resources").await;
    // player_animation_demo().await;
    local_game().await;
    //camera_test().await;
}


fn debug_ball_throwing(game: &mut Game) {
    if is_mouse_button_pressed(MouseButton::Left) { // this is for testing purpose
        let pos = Vec2::from(mouse_position());
        // reposition the ball to cursor
        game.ball.pos = pos;
        game.gravity_line = game.ball.pos.y + PLAYER_HEIGHT;
        // which team has the ball? and mark target player from opposite side
        let m_team = game.which_team_has_ball();
        let other_team = other_team(m_team);
        let target_player = game.get_active_player_for_team(other_team);
        let player: &Player = if target_player.is_some() {
            &game.players[target_player.unwrap()]
        } else {
            &game.players[0]
        };
        // fix target
        let target_pos = (player.pos - game.ball.pos).normalize();
        // throw it
        game.ball.throwing(target_pos, pos, player.facing_to_before.clone())
    }

    if is_mouse_button_pressed(MouseButton::Right) { // this is for testing purpose
        let pos = Vec2::from(mouse_position());
        // reposition the ball to cursor
        game.ball.pos = pos;
        // which team has the ball? and mark target player from opposite side
        // fix target
        let target_pos = (Vec2::new(game.ball.pos.x, game.ball.pos.y + 10.) - game.ball.pos).normalize();
        game.gravity_line = game.ball.pos.y + PLAYER_HEIGHT;
        // throw it
        game.ball.vel = Vec2::new(-3., -1.);
        game.ball.state = BallState::AfterHittingPlayer {
            time_passed:game.time_passed,
            change_y: true,
            change_x: true
        };
        game.ball.throwing(target_pos, pos, FacingTo::FacingBottom)
    }
}

fn debug_collision(game: &Game) {
    let position = Vec2::from(mouse_position());
    for player in &game.players {
        let (collided, _, _) = colliding_with(&position, 10., &player.pos,
                                              &Vec2::new(PLAYER_WIDTH, PLAYER_HEIGHT));
        if collided {
            draw_rectangle_lines_a(position, 20., 20., 3., RED);
        }
    }
    // checking if out of the field
    let outside = game.field.point_outside_field(position, 10.);
    draw_circle_lines(position.x, position.y, 11., 2., if position.x + 5. > game.field.mid_section { BLUE } else { BROWN });
    draw_circle_lines(position.x, position.y, 5., 3., if outside { RED } else { GREEN });
}

fn draw_field(game: &Game) {
    draw_line_a(game.field.top_left, game.field.top_right, 3., BLACK);
    draw_line_a(game.field.top_left, game.field.bottom_left, 3., DARKGREEN);
    draw_line_a(game.field.bottom_left, game.field.bottom_right, 3., RED);
    draw_line_a(game.field.top_right, game.field.bottom_right, 3., GREEN);
    draw_line_a(game.field.mid_section_top, game.field.mid_section_bottom, 3., YELLOW);
}

async fn player_animation_demo() {
    let (player_characters, balls) = {
        let resources = storage::get::<Resources>();
        (resources.player_characters.clone(), resources.balls.clone())
    };

    let mut animation_players = Vec::new();
    for player_character in player_characters.iter() {
        let mut animation_params: AnimationParams = player_character.animation.clone().into();
        let mut player = AnimationPlayer::new(animation_params);
        player.set_animation(Player::MOVE_ANIMATION_ID);
        animation_players.push(player);
    }
    for ball in balls {
        let mut animation_params: AnimationParams = ball.animation.clone().into();
        let mut player = AnimationPlayer::new(animation_params);
        player.set_animation(Ball::MOVE_ANIMATION_ID);
        animation_players.push(player);
    }
    let animation_list = HashMap::from([
        (KeyCode::Key0, Player::IDLE_ANIMATION_ID),
        (KeyCode::Key1, Player::MOVE_ANIMATION_ID),
        (KeyCode::Key2, Player::DEATH_BACK_ANIMATION_ID),
        (KeyCode::Key3, Player::DEATH_FACE_ANIMATION_ID),
        (KeyCode::Key4, Player::PUNCH_ANIMATION_ID),
        (KeyCode::Key5, Player::RUN_ANIMATION_ID),
        (KeyCode::Key6, Player::CROUCH_ANIMATION_ID),
        (KeyCode::Key7, Player::JUMP_ANIMATION_ID),
        (KeyCode::Key8, Player::FALL_ANIMATION_ID),
        (KeyCode::Key9, Player::HURT_ANIMATION_ID),
    ]);

    loop {
        let offset = Vec2::new(80., 0.);
        let mut position = Vec2::new((screen_width() / 2.) - 64., (screen_height() / 2.) - 64.);
        for animation_player in animation_players.iter_mut() {
            for keys in &animation_list {
                if is_key_pressed(*keys.0) {
                    animation_player.set_animation(*keys.1);
                }
            }
            animation_player.update();
            animation_player.set_scale(2.0);
            animation_player.draw(position, 0.0, false, false);
            position += offset;
        }
        if is_key_pressed(KeyCode::Enter) {
            break;
        }
        next_frame().await;
    }
}

async fn new_game(keys_mapped: &Vec<KeyCode>) -> Game {
    let resources = storage::get::<Resources>();
    let mut game = Game::default();
    game.key_sets = HashMap::from([(
        Team::One, HashMap::from([
            (PlayerAction::MoveUp, keys_mapped[0]),
            (PlayerAction::MoveLeft, keys_mapped[1]),
            (PlayerAction::MoveDown, keys_mapped[2]),
            (PlayerAction::MoveRight, keys_mapped[3]),
            (PlayerAction::A, keys_mapped[4]),
            (PlayerAction::B, keys_mapped[5]),
        ])),
        (Team::Two, HashMap::from([
            (PlayerAction::MoveUp, keys_mapped[6]),
            (PlayerAction::MoveLeft, keys_mapped[7]),
            (PlayerAction::MoveDown, keys_mapped[8]),
            (PlayerAction::MoveRight, keys_mapped[9]),
            (PlayerAction::A, keys_mapped[10]),
            (PlayerAction::B, keys_mapped[11]),
        ]))
    ]);
    game.players = { // Player animations
        let player_animations = || {
            let mut animation_params: AnimationParams = resources.player_characters[0].animation.clone().into();
            let mut animation_player = AnimationPlayer::new(animation_params);
            animation_player.set_animation(Player::IDLE_ANIMATION_ID);
            animation_player.set_scale(1.);
            animation_player
        };
        let mut players = vec![];
        let p1_pos = Vec2::new(game.field.mid_section - PLAYER_WIDTH - 80., screen_height() / 2.);
        let p2_pos = Vec2::new(game.field.mid_section + 80., screen_height() / 2.);
        players.push(Player::new(
            0,
            p1_pos,
            90.,
            Vec2::new(0., 0.),
            100,
            false,
            BLACK,
            FacingTo::FacingRight,
            player_animations(),
        ));

        players.push(Player::new(
            1,
            p2_pos,
            -90.,
            Vec2::new(0., 0.),
            100,
            false,
            DARKGRAY,
            FacingTo::FacingLeft,
            player_animations(),
        ));
        players
    };
    game.balls = {
        let mut animation_params: AnimationParams = resources.balls[0].animation.clone().into();
        let mut animation_player = AnimationPlayer::new(animation_params);
        animation_player.set_animation(Ball::IDLE_ANIMATION_ID);
        vec![animation_player]
    };
    game.ball = Ball {
        pos: game.players[0].pos,
        vel: Vec2::default(),
        r: 16.,
        collided: false,
        thrown: false,
        dropped: false,
        color: BLACK,
        in_air: false,
        grabbed_by: Some(0),
        animation: 0,
        stopped: true,
        state: BallState::OnPlayersHand(0),
        tick: 0.0
    };
    game
}

fn window_conf() -> Conf {
    Conf {
        window_title: "Super dodge ball".to_owned(),
        fullscreen: false,
        window_resizable: true,
        window_width: 1080,
        window_height: 860,
        ..Default::default()
    }
}