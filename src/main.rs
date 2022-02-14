use std::collections::HashMap;
use std::ops::Deref;
use macroquad::prelude::*;
use macroquad::experimental::animation::{Animation, AnimatedSprite};
use macroquad::prelude::collections::storage;
use game::has_direction;
use game::draw_utilities::{draw_line_a, draw_rectangle_lines_a};
use crate::game::{calculate_movement, Game, other_team};
use game::has_direction::{HasDirection};
use game::player::Player;
use crate::game::animations::{AnimationParams, AnimationPlayer};
use crate::game::ball::Ball;
use crate::game::character::PlayerCharacterParams;
use crate::game::resources::{load_resources, Resources};

pub mod helpers;
pub mod error;
pub mod json;
pub mod game;
pub mod math;
pub mod noise;

const PLAYER_HEIGHT: f32 = 40.;
const PLAYER_WIDTH: f32 = 55.;
const RESET_KEY: usize = 12;
// frames
const IDLE_FRAMES: usize = 8;
const WALK_FRAMES: usize = 8;
const RUN_FRAMES: usize = 4;
const HURT_FRAMES: usize = 2;
const DEATH_FRAMES: usize = 7;
const THROW_FRAMES: usize = 8;


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

#[derive(Eq, PartialEq, Debug, Hash, Clone, Copy)]
pub enum Team {
    One,
    Two,
}


#[derive(Debug, PartialEq, Eq)]
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


fn player_facing_ball(player: &Player, ball: &Ball) -> bool {
    // let player_rot_vec = has_direction::rotation_vector(player);
    // let ball_rot_vec = has_direction::rotation_vector(ball);
    // let calc = player_rot_vec.dot(ball_rot_vec);
    // calc < 0.
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

fn catch_ball(player: &mut Player, ball: &mut Ball, keys: &HashMap<PlayerAction, KeyCode>) -> bool {
    let facing_towards_ball = player_facing_ball(&player, &ball);
    // ducking saves player from damage
    if is_key_down(keys[&PlayerAction::A]) {
        return false;
    }
    // if colliding_with(&ball.pos, ball.r, &player) {
    //     player.has_ball = true;
    //     let facing_to = &player.facing_to;
    //     if !facing_towards_ball {
    //         player.life -= 1;
    //     } else {
    //         // need to make sure catch key is pressed
    //         let key_pressed = match facing_to {
    //             FacingTo::FacingTop | FacingTo::FacingTopRight | FacingTo::FacingTopLeft => keys[&PlayerAction::MoveUp],
    //             FacingTo::FacingBottom | FacingTo::FacingBottomRight | FacingTo::FacingBottomLeft => keys[&PlayerAction::MoveDown],
    //             FacingTo::FacingRight => keys[&PlayerAction::MoveRight],
    //             FacingTo::FacingLeft => keys[&PlayerAction::MoveLeft],
    //         };
    //         if !(is_key_down(keys[&PlayerAction::B]) && is_key_down(key_pressed)) {
    //             player.life -= 1;
    //             println!("player was hit! {}", player.life);
    //         }
    //         return true;
    //     }
    // }
    return false;
}


fn move_player(game: &mut Game, player_index: usize) {
    let current_team = if player_index >= game.players.len() / 2 { Team::Two } else { Team::One };
    let active_player = game.deref().get_active_player_for_team(current_team);
    let keys = &game.key_sets.get(&current_team).unwrap();
    // player will not move while catching or ducking
    if is_key_down(keys[&PlayerAction::B]) || is_key_down(keys[&PlayerAction::A]) {
        return;
    }
    if Some(player_index) != active_player { return; }
    let player: &mut Player = &mut game.players[player_index];
    player.animation_player.set_animation(Player::IDLE_ANIMATION_ID);
    // mark the keys pressed
    let keys_pressed = (
        is_key_down(keys[&PlayerAction::MoveUp]),
        is_key_down(keys[&PlayerAction::MoveRight]),
        is_key_down(keys[&PlayerAction::MoveDown]),
        is_key_down(keys[&PlayerAction::MoveLeft]),
    );
    let (rotation, facing_to, acc) = calculate_movement(keys_pressed);
    player.facing_to = facing_to;
    player.rotation = rotation;
    player.vel += acc.unwrap_or(-player.vel);
    if player.vel.length() > 0. {
        player.animation_player.set_animation(Player::MOVE_ANIMATION_ID);
    }
    if player.vel.length() > 5. {
        player.vel = player.vel.normalize() * 5.;
    }
    let prev_pos = player.pos;
    player.pos += player.vel;
    if !valid_position(&player.pos) {
        player.pos = prev_pos;
        player.animation_player.set_animation(Player::HURT_ANIMATION_ID);
    }
    player.animation_player.update();
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
    Some(target_player_index)
}

const TEAM_ONE_PLAYER: usize = 1;
const TEAM_TWO_PLAYER: usize = 2;
const TEAM_ONE_PLAYER_READY: usize = 3;
const TEAM_TWO_PLAYER_READY: usize = 3;
const PLAYER_ANIMATED_TEXTURES: usize = 4;


async fn player_animation_demo() {
    let player_characters = {
        let resources = storage::get::<Resources>();
        resources.player_characters.clone()
    };

    let mut animation_players = Vec::new();
    for player_character in player_characters.iter() {
        let mut animation_params: AnimationParams = player_character.animation.clone().into();
        let mut player = AnimationPlayer::new(animation_params);
        player.set_animation(Player::MOVE_ANIMATION_ID);
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
    let mut game = Game::new();
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
    game.textures = vec![
        load_texture("resources/textures/soccer-ball.png").await.unwrap(),
        load_texture("resources/textures/grin_64.png").await.unwrap(),
        load_texture("resources/textures/grin_64_flipped.png").await.unwrap(),
        load_texture("resources/textures/scared.png").await.unwrap(),
        load_texture("resources/textures/player/bandit_64x64.png").await.unwrap(),
    ];
    let animation_player = || {
        let mut animation_params: AnimationParams = resources.player_characters[0].animation.clone().into();
        let mut animation_player = AnimationPlayer::new(animation_params);
        animation_player.set_animation(Player::IDLE_ANIMATION_ID);
        animation_player.set_scale(1.5);
        animation_player
    };
    let mut players = vec![];
    players.push(
        Player::new(
            0,
            Vec2::new(game.field.mid_section - PLAYER_WIDTH - 80., screen_height() / 2.),
            90.,
            Vec2::new(0., 0.),
            100,
            false,
            BLACK,
            FacingTo::FacingRight,
            animation_player(),
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
        animation_player(),
    ));
    game.players = players;
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
    };
    game
}

fn window_conf() -> Conf {
    Conf {
        window_title: "Super dodge ball".to_owned(),
        fullscreen: true,
        window_resizable: false,
        window_width: 1080,
        window_height: 860,
        ..Default::default()
    }
}

async fn local_game() {
    // this should allow us to remap keys later
    let keys_mapped = vec![
        KeyCode::W, KeyCode::A, KeyCode::S, KeyCode::D, KeyCode::N, KeyCode::M,
        KeyCode::I, KeyCode::J, KeyCode::K, KeyCode::L, KeyCode::Z, KeyCode::X,
        KeyCode::Enter,
    ];
    let mut game = new_game(&keys_mapped).await;
    let mut time_pressed = 0.;
    //
    //
    loop {
        let frame_t = get_time();
        if is_key_pressed(keys_mapped[RESET_KEY]) {
            game = new_game(&keys_mapped).await;
            println!("Resetting Game");
            next_frame().await;
            continue;
        }
        // check which team has the ball
        let team_with_ball = game.which_team_has_ball();
        for i in 0..game.players.len() {
            move_player(&mut game, i);
        }
        if let Some(p) = game.ball.grabbed_by {
            throw_ball(&mut game, p, &team_with_ball);
        }

        debug_ball_throwing(&mut game);
        is_ball_outside_boundary(&mut game);

        for (i, player) in game.players.iter_mut().enumerate() {
            let (collided, change_x, change_y) = colliding_with(&game.ball.pos, game.ball.r, &player);
            if !collided {
                continue;
            }
            let option = game.key_sets[&team_with_ball].get(&PlayerAction::B).unwrap();
            if is_key_down(*option) {
                player.ready_to_catch = true;
            }
            if is_key_released(*option) {
                player.ready_to_catch = false;
            }
            if player.ready_to_catch {
                game.ball.picked_up(i);
                player.ready_to_catch = false;
                player.has_ball = true;
            } else {
                // ball is not captured, so ball has hit the player
                game.ball.collided = true;
                game.ball.dropped = true;
                if change_y {
                    game.ball.vel.y *= -1.;
                }
                if change_x {
                    game.ball.vel.x *= -1.;
                }
                game.ball.vel -= game.ball.vel / 10.;
            }
        }
        if !game.ball.thrown {
            if game.ball.grabbed_by.is_some() {
                match team_with_ball {
                    Team::One => {
                        game.ball.pos = game.players[game.ball.grabbed_by.unwrap()].pos + Vec2::new(PLAYER_WIDTH + game.ball.r + 10., 0.);
                    }
                    Team::Two => {
                        game.ball.pos = game.players[game.ball.grabbed_by.unwrap()].pos - Vec2::new(game.ball.r + 10., 0.);
                    }
                }
            }
        } else {
            game.ball.pos += game.ball.vel;
            if game.ball.vel.length() > 5. {
                game.ball.vel = game.ball.vel.normalize() * 5.;
            }

            if game.ball.collided {
                game.ball.vel -= game.ball.vel / 5.;
            }

            if game.ball.vel.length() < 0.1 {
                game.ball.collided = false;
                game.ball.vel = Vec2::default();
                game.ball.grabbed_by = None;
            }
        }
        clear_background(LIGHTGRAY);
        draw_field(&game);
        debug_collision(&game);
        draw_ball(&game);
        // draw_players(&game, frame_t);
        for (i, player) in game.players.iter().enumerate() {
            // draw some status
            let txt = format!("{}", player.life);
            draw_text(&txt, player.pos.x, player.pos.y - 20., 20.0, calculate_life_color(player.life));
            let flip_x = i >= game.players.len() / 2;
            player.animation_player.draw(
                player.pos, 0., flip_x, false,
            );
        }
        next_frame().await
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    load_resources("resources").await;
    player_animation_demo().await;
    local_game().await;
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


fn draw_ball(game: &Game) {
    let bx = _x(&game.ball);
    let by = _y(&game.ball);
    draw_texture(game.textures[0], bx - game.ball.r, by - game.ball.r, WHITE);
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

