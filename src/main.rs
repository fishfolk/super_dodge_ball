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
use crate::game::ball::Ball;
use crate::game::character::PlayerCharacterParams;
use crate::game::resources::{load_resources, Resources};
use crate::json::is_false;

pub mod helpers;
pub mod error;
pub mod json;
pub mod game;
pub mod math;
pub mod noise;

const PLAYER_HEIGHT: f32 = 40.;
const PLAYER_WIDTH: f32 = 55.;
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
enum PlayerAction {
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


#[derive(Debug, PartialEq, Eq, Clone)]
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

fn colliding_with(pos: &Vec2, r: f32, player: &Player) -> (bool, bool, bool) {
    // temporary variables to set edges for testing
    let mut test_x = pos.x;
    let mut test_y = pos.y;
    // which edge is closest?
    let change_x = if pos.x < player.pos.x {
        test_x = player.pos.x;      // test left edge
        true
    } else if pos.x > player.pos.x + PLAYER_WIDTH {
        test_x = player.pos.x + PLAYER_WIDTH;   // right edge
        true
    } else {
        false
    };
    //
    let change_y = if pos.y < player.pos.y {
        test_y = player.pos.y;      // top edge
        true
    } else if pos.y > player.pos.y + PLAYER_HEIGHT {
        test_y = player.pos.y + PLAYER_HEIGHT;   // bottom edge
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

fn update_player(game: &mut Game, player_index: usize) {
    let current_team = if player_index >= game.players.len() / 2 { Team::Two } else { Team::One };
    let active_player = game.deref().get_active_player_for_team(current_team);
    if Some(player_index) != active_player { return; }
    let keys = &game.key_sets.get(&current_team).unwrap();
    let keys_pressed = [
        is_key_down(keys[&PlayerAction::MoveUp]),
        is_key_down(keys[&PlayerAction::MoveRight]),
        is_key_down(keys[&PlayerAction::MoveDown]),
        is_key_down(keys[&PlayerAction::MoveLeft]),
        is_key_down(keys[&PlayerAction::B]),
        is_key_down(keys[&PlayerAction::A])
    ];
    let player: &mut Player = &mut game.players[player_index];
    player.update_player_state(keys_pressed);
    player.set_animation();
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
    game.ball.throwing(target_pos, pos);
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
    // let mut time_pressed = 0.;
    loop {
        let frame_t = get_time();
        if is_key_pressed(keys_mapped[RESET_KEY]) {
            game = new_game(&keys_mapped).await;
            game.set_zoom(None);
            next_frame().await;
            continue;
        }
        game.on_player_doing_things_with_ball(frame_t);
        game.on_player_catching_ball();
        game.on_ball_hitting_player();
        game.on_player_threw_ball();
        for i in 0..game.players.len() {
            update_player(&mut game, i);
        }
        let team = game.which_team_has_ball();
        if let Some(p) = game.ball.grabbed_by {
            throw_ball(&mut game, p, &team);
        }
        debug_ball_throwing(&mut game);
        is_ball_outside_boundary(&mut game);
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
        set_camera(&Camera2D {
            target: game.ball.pos,
            rotation: 180.,
            zoom: game.zoom,
            ..Default::default()
        });
        debug_collision(&game);
        {
            let bx = _x(&game.ball);
            let by = _y(&game.ball);
            let which_animation = if game.ball.in_air {
                Ball::MOVE_ANIMATION_ID
            } else {
                Ball::IDLE_ANIMATION_ID
            };
            game.balls[game.ball.animation].set_animation(which_animation);
            game.balls[game.ball.animation].update();
            game.balls[game.ball.animation].draw(Vec2::new(bx, by), 0., false, false);
        }
        for (i, player) in game.players.iter().enumerate() {
            let txt = format!("{}", player.life);
            draw_text(&txt, player.pos.x, player.pos.y - 20., 20.0, calculate_life_color(player.life));
            let flip_x = should_face_to(
                player.facing_to.clone(),
                if i >= game.players.len() { Team::Two} else { Team::Two },
                player.facing_to_before.clone()
            );
            player.animation_player.draw(
                player.pos, 0., flip_x, false,
            );
        }
        next_frame().await
    }
}

fn should_face_to(facing_to: FacingTo, which_team: Team, facing_to_before: FacingTo) -> bool {
    let which = || {
        if facing_to_before == FacingTo::FacingLeft {
            if which_team == Team::One { false }
            else { true }
        } else {
            if which_team == Team::One { true }
            else { false }
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

fn is_ball_outside_boundary(game: &mut Game) {
    let prev_pos = &game.ball.pos;
    if prev_pos.y + game.ball.vel.y < game.field.top_edge || prev_pos.y + game.ball.vel.y > game.field.bottom_edge {
        game.ball.vel.y *= -1.;
        game.ball.collided = true;
    }
    if prev_pos.x + game.ball.vel.x > game.field.right_edge || prev_pos.x + game.ball.vel.x < game.field.left_edge {
        game.ball.vel.x *= -1.;
        game.ball.collided = true;
    }
}

fn debug_ball_throwing(game: &mut Game) {
    if is_mouse_button_pressed(MouseButton::Left) { // this is for testing purpose
        let pos = Vec2::from(mouse_position());
        // reposition the ball to cursor
        game.ball.pos = pos;
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
        game.ball.throwing(target_pos, pos)
    }
}

fn debug_collision(game: &Game) {
    let position = Vec2::from(mouse_position());
    for player in &game.players {
        let (collided, _, _) = colliding_with(&position, 10., &player);
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
    game.gravity = Vec2::new(-2., -2.);
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
        players.push(Player::new(
            0,
            Vec2::new(game.field.mid_section - PLAYER_WIDTH - 80., screen_height() / 2.),
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
            Vec2::new(game.field.mid_section + 80., screen_height() / 2.),
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
        r: 10.,
        collided: false,
        thrown: false,
        dropped: false,
        color: BLACK,
        in_air: false,
        grabbed_by: Some(0),
        animation: 0,
        stopped: true,
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


fn short_angle_dist(a0: f32, a1: f32) -> f32 {
    let max = 360.0;
    let da = (a1 - a0) % max;
    2.0 * da % max - da
}

fn angle_lerp(a0: f32, a1: f32, t: f32) -> f32 {
    a0 + short_angle_dist(a0, a1) * t
}

fn draw_cross(x: f32, y: f32, color: Color) {
    let size = 0.1;
    let thickness = 0.005;
    draw_line(x - size, y, x + size, y, thickness, color);
    draw_line(x, y - size, x, y + size, thickness, color);
}

async fn camera_test() {
    let mut target = (0., 0.);
    let mut zoom = 1.0;
    let mut rotation = 0.0;
    let mut smooth_rotation: f32 = 0.0;
    let mut offset = (0., 0.);

    loop {
        if is_key_down(KeyCode::W) {
            target.1 -= 0.1;
        }
        if is_key_down(KeyCode::S) {
            target.1 += 0.1;
        }
        if is_key_down(KeyCode::A) {
            target.0 += 0.1;
        }
        if is_key_down(KeyCode::D) {
            target.0 -= 0.1;
        }
        if is_key_down(KeyCode::Left) {
            offset.0 -= 0.1;
        }
        if is_key_down(KeyCode::Right) {
            offset.0 += 0.1;
        }
        if is_key_down(KeyCode::Up) {
            offset.1 += 0.1;
        }
        if is_key_down(KeyCode::Down) {
            offset.1 -= 0.1;
        }
        #[cfg(not(target_arch = "wasm32"))]
        if is_key_down(KeyCode::Q) | is_key_down(KeyCode::Escape) {
            break;
        }

        match mouse_wheel() {
            (_x, y) if y != 0.0 => {
                // Normalize mouse wheel values is browser (chromium: 53, firefox: 3)
                #[cfg(target_arch = "wasm32")]
                    let y = if y < 0.0 {
                    -1.0
                } else if y > 0.0 {
                    1.0
                } else {
                    0.0
                };
                if is_key_down(KeyCode::LeftControl) {
                    zoom *= 1.1f32.powf(y);
                } else {
                    rotation += 10.0 * y;
                    rotation = match rotation {
                        angle if angle >= 360.0 => angle - 360.0,
                        angle if angle < 0.0 => angle + 360.0,
                        angle => angle,
                    }
                }
            }
            _ => (),
        }

        smooth_rotation = angle_lerp(smooth_rotation, rotation, 0.1);

        clear_background(LIGHTGRAY);

        set_camera(&Camera2D {
            target: vec2(target.0, target.1),
            ..Default::default()
        });
        draw_cross(0., 0., RED);

        set_camera(&Camera2D {
            target: vec2(target.0, target.1),
            rotation: smooth_rotation,
            ..Default::default()
        });
        draw_cross(0., 0., GREEN);

        set_camera(&Camera2D {
            target: vec2(target.0, target.1),
            rotation: smooth_rotation,
            zoom: vec2(zoom, zoom * screen_width() / screen_height()),
            ..Default::default()
        });
        draw_cross(0., 0., BLUE);

        set_camera(&Camera2D {
            target: vec2(target.0, target.1),
            rotation: smooth_rotation,
            zoom: vec2(zoom, zoom * screen_width() / screen_height()),
            offset: vec2(offset.0, offset.1),
            ..Default::default()
        });

        // Render some primitives in camera space
        draw_line(-0.4, 0.4, -0.8, 0.9, 0.05, BLUE);
        draw_rectangle(-0.3, 0.3, 0.2, 0.2, GREEN);
        draw_circle(0., 0., 0.1, YELLOW);

        // Back to screen space, render some text
        set_default_camera();
        draw_text(
            format!("target (WASD keys) = ({:+.2}, {:+.2})", target.0, target.1).as_str(),
            10.0,
            10.0,
            15.0,
            BLACK,
        );
        draw_text(
            format!("rotation (mouse wheel) = {} degrees", rotation).as_str(),
            10.0,
            25.0,
            15.0,
            BLACK,
        );
        draw_text(
            format!("zoom (ctrl + mouse wheel) = {:.2}", zoom).as_str(),
            10.0,
            40.0,
            15.0,
            BLACK,
        );
        draw_text(
            format!("offset (arrow keys) = ({:+.2}, {:+.2})", offset.0, offset.1).as_str(),
            10.0,
            55.0,
            15.0,
            BLACK,
        );
        draw_text("HELLO", 30.0, 200.0, 30.0, BLACK);

        next_frame().await
    }
}