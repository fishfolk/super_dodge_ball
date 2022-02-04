use std::collections::HashMap;
use macroquad::prelude::*;
use game::has_direction;
use game::draw_utilities::{draw_line_a, draw_rectangle_lines_a};
use crate::game::{Game, new_game};
use game::has_direction::{HasDirection, rotation_vector};
use game::player::Player;
use crate::game::ball::Ball;

const PLAYER_HEIGHT: f32 = 35.;
const PLAYER_WIDTH: f32 = 35.;

pub mod game;

#[derive(Eq, PartialEq, Hash, Debug)]
enum PlayerAction {
    A,
    B,
    MoveLeft,
    MoveRight,
    MoveUp,
    MoveDown,
}

#[derive(Eq, PartialEq, Debug, Hash)]
pub enum Team {
    One,
    Two,
}


#[derive(Debug, PartialEq, Eq)]
enum FacingTo {
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
    let player_rot_vec = has_direction::rotation_vector(player);
    let ball_rot_vec = has_direction::rotation_vector(ball);
    let calc = player_rot_vec.dot(ball_rot_vec);
    calc < 0.
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
    let key_set_index = if player_index >= game.players.len() / 2 { Team::Two } else { Team::One };
    let player = &mut game.players[player_index];
    // can this player be controlled by keyboard?
    let active_player = game.active_chars.get(&key_set_index).unwrap();
    if player_index != active_player.0 { return; }
    let keys = &game.key_sets.get(&key_set_index).unwrap();
    // player will not move while catching or ducking
    if is_key_down(keys[&PlayerAction::B]) || is_key_down(keys[&PlayerAction::A]) { return; }
    let key_right = is_key_down(keys[&PlayerAction::MoveRight]);
    let key_left = is_key_down(keys[&PlayerAction::MoveLeft]);
    let key_down = is_key_down(keys[&PlayerAction::MoveDown]);
    let key_up = is_key_down(keys[&PlayerAction::MoveUp]);
    let acc = if key_up && key_right {
        player.rotation = 45.;
        player.facing_to = FacingTo::FacingTopRight;
        Vec2::new(1., -1.)
    } else if key_down && key_right {
        player.facing_to = FacingTo::FacingBottomRight;
        player.rotation = 135.;
        Vec2::new(1., 1.)
    } else if key_up && key_left {
        player.rotation = 315.;
        player.facing_to = FacingTo::FacingTopLeft;
        Vec2::new(-1., -1.)
    } else if key_down && key_left {
        player.rotation = 225.;
        player.facing_to = FacingTo::FacingBottomLeft;
        Vec2::new(-1., 1.)
    } else if key_right {
        player.rotation = 90.;
        player.facing_to = FacingTo::FacingRight;
        Vec2::new(1., 0.)
    } else if key_left {
        player.rotation = 270.;
        player.facing_to = FacingTo::FacingLeft;
        Vec2::new(-1., 0.)
    } else if key_down {
        player.rotation = 180.;
        player.facing_to = FacingTo::FacingBottom;
        Vec2::new(0., 1.)
    } else if key_up {
        player.rotation = 0.;
        player.facing_to = FacingTo::FacingTop;
        Vec2::new(0., -1.)
    } else {
        -player.vel
    };
    player.vel += acc;
    if player.vel.length() > 5. {
        player.vel = player.vel.normalize() * 5.;
    }
    let prev_pos = player.pos;
    player.pos += player.vel;
    if !valid_position(&player.pos) {
        player.pos = prev_pos;
    }
}

fn throw_ball(game: &mut Game, frame_t: f64) {
    let (player, other) = if game.team_with_ball == Team::One {
        (&game.players[1], &game.players[0])
    } else {
        (&game.players[0], &game.players[1])
    };
    //..
    let rot_vec = rotation_vector(player);
    let target_pos = (other.pos - player.pos).normalize();
    game.ball.throwing(target_pos, player.pos + rot_vec * PLAYER_HEIGHT, player.rotation);
}

const RESET_KEY: usize = 12;

#[macroquad::main("Super Dodge Ball")]
async fn main() {
    let mut game = new_game();
    // this will allow us to remap keys
    let keys_mapped = vec![
        KeyCode::W, KeyCode::A, KeyCode::S, KeyCode::D, KeyCode::N, KeyCode::M,
        KeyCode::I, KeyCode::J, KeyCode::K, KeyCode::L, KeyCode::Z, KeyCode::X,
        KeyCode::Enter,
    ];
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
    let mut time_pressed = 0.;
    loop {
        let frame_t = get_time();

        // store initial rotation
        if is_key_pressed(keys_mapped[RESET_KEY]) {
            game = new_game();
            println!("Resetting Game");
            next_frame().await;
            continue;
        }
        let time_diff = frame_t - time_pressed;
        // if is_key_down(KeyCode::D) {
        //     let last_key = get_last_key_pressed();
        //     if last_key != None && last_key.unwrap() == KeyCode::D && time_diff < 0.5 {
        //         running_mod = true;
        //     }
        // }
        // check which team has the ball
        let team = game.which_team_has_ball();
        for i in 0..game.players.len() {
            move_player(&mut game, i);
        }

        if is_mouse_button_pressed(MouseButton::Left) { // this is for testing purpose
            let pos = Vec2::from(mouse_position());
            // reposition the ball to cursor
            game.ball.pos = pos;
            // which team has the ball? and mark target player from opposite side
            let team = game.which_team_has_ball();
            let target_player = game.get_active_player(team, false);
            let player: &Player = if target_player.is_some() {
                &game.players[target_player.unwrap()]
            } else {
                &game.players[0]
            };
            // fix target
            let target_pos = (player.pos - game.ball.pos).normalize();
            // throw it
            game.ball.throwing(target_pos, pos, 90.)
        }
        let prev_pos = &game.ball.pos;
        if prev_pos.y + game.ball.vel.y < game.field.top_edge || prev_pos.y + game.ball.vel.y > game.field.bottom_edge {
            game.ball.vel.y *= -1.;
            game.ball.collided = true;
        }
        if prev_pos.x + game.ball.vel.x > game.field.right_edge || prev_pos.x + game.ball.vel.x < game.field.left_edge {
            game.ball.vel.x *= -1.;
            game.ball.collided = true;
        }
        for (i, player) in game.players.iter_mut().enumerate() {
            let (collided, change_x, change_y) = colliding_with(&game.ball.pos, game.ball.r, &player);
            if !collided {
                continue;
            }
            // check if
            // 1. ball is dropped
            // 2. player is facing the ball
            // 3. and is ready to catch it
            if game.ball.dropped {
                game.ball.picked_up(i);
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
                game.ball.pos = game.players[game.ball.grabbed_by.unwrap()].pos;
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
        for player in &game.players {
            draw_player(player);
        }
        next_frame().await
    }
}


fn draw_ball(game: &Game) {
    let txt = format!("r: {}, p: {}", game.ball.vel, game.ball.pos);
    draw_text(&txt, game.ball.pos.x, game.ball.pos.y + 20.0, 20.0, DARKGRAY);
    draw_circle(game.ball.pos.x, game.ball.pos.y, game.ball.r, game.ball.color);
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

fn draw_player(player: &Player) {
    // draw some status
    let txt = format!("{}, {}", player.life, player.rotation);
    draw_text(&txt, player.pos.x, player.pos.y - 20., 20.0, calculate_life_color(player.life));
    let color = if player.has_ball { BLUE } else { BLACK };
    draw_rectangle_lines(player.pos.x, player.pos.y, PLAYER_WIDTH, PLAYER_HEIGHT, 4., color);
}

