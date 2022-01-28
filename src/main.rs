use std::collections::HashMap;
use macroquad::prelude::*;
use crate::draw_utilities::{draw_line_a, draw_rectangle_lines_a};
use crate::game::{Game, new_game};
use crate::has_direction::{HasDirection, rotation_vector};

const PLAYER_HEIGHT: f32 = 35.;
const PLAYER_WIDTH: f32 = 35.;

pub mod has_direction;
pub mod game;
pub mod draw_utilities;
pub mod field;

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
enum Team {
    One,
    Two,
}


#[derive(PartialEq, Debug)]
struct Player {
    pos: Vec2,
    rotation: f32,
    vel: Vec2,
    color: Color,
    facing_to: FacingTo,
    ducking: bool,
    jumping: bool,
    has_ball: bool,
    life: i32,
}

#[derive(Clone)]
struct Ball {
    pos: Vec2,
    vel: Vec2,
    r: f32,
    rotation: f32,
    color: Color,
    shot_at: f64,
    collided: bool,
}

impl HasDirection for Player {
    fn get_position(&self) -> Vec2 {
        self.pos
    }
    fn get_rotation(&self) -> f32 {
        self.rotation
    }
    fn get_rotation_as_radian(&self) -> f32 {
        self.rotation.to_radians()
    }
}

impl HasDirection for Ball {
    fn get_position(&self) -> Vec2 {
        self.pos
    }
    fn get_rotation(&self) -> f32 {
        self.rotation
    }
    fn get_rotation_as_radian(&self) -> f32 {
        self.rotation.to_radians()
    }
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
    // pos.x + r > player.pos.x && pos.x + r < player.pos.x + PLAYER_WIDTH
    // &&
    // pos.y + r > player.pos.y && pos.y + r < player.pos.y + PLAYER_HEIGHT
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

fn throw_ball(game: &mut Game, frame_t: f64) -> Option<Ball> {
    let (player, other) = if game.team_with_ball == Team::One {
        (&game.players[1], &game.players[0])
    } else {
        (&game.players[0], &game.players[1])
    };
    let rot_vec = rotation_vector(player);
    let target_pos = (other.pos - player.pos).normalize();
    Some(Ball {
        pos: player.pos + rot_vec * PLAYER_HEIGHT,
        vel: target_pos * 5.,
        r: 10.,
        rotation: player.rotation,
        shot_at: frame_t,
        collided: false,
        color: DARKBROWN,
    })
}

#[macroquad::main("Super Dodge Ball")]
async fn main() {
    let mut game = new_game();
    loop {
        let frame_t = get_time();
        // store initial rotation
        if is_key_pressed(KeyCode::Enter) {
            game = new_game();
            println!("Resetting Game");
            next_frame().await;
            continue;
        }

        // let target: &Player = find_closest_player(&ball, &game.players);
        // check which team has the ball

        // game.mark_active_player(Team::One, frame_t);
        // game.mark_active_player(Team::Two, frame_t);

        for i in 0..game.players.len() {
            move_player(&mut game, i);
            // if is_key_down(player.keys[&PlayerAction::B]) {
            //     throw_ball(&mut game, frame_t);
            // }
        }

        if is_key_down(KeyCode::Key0) {
            println!("{:?}", game.players);
            // game.ball.
        }

        if is_mouse_button_pressed(MouseButton::Left) {
            game.ball.pos = Vec2::from(mouse_position());
            let target_pos = (game.players[0].pos - game.ball.pos).normalize();
            game.ball.vel = target_pos * 5.;
        }

        // find closest other team's player

        // set him as target

        // if game.players[0].has_ball {
        //     if is_key_down(game.players[0].keys[&PlayerAction::B]) {
        //         game.ball = throw_ball(&game.players[0], frame_t, &game.players[1]);
        //         game.last_shot = frame_t;
        //         game.players[0].has_ball = false;
        //     }
        // }

        // if game.players[1].has_ball {
        //     if is_key_down(game.players[1].keys[&PlayerAction::B]) {
        //         game.ball = throw_ball(&game.players[1], frame_t, &game.players[0]);
        //         game.last_shot = frame_t;
        //         game.players[1].has_ball = false;
        //     }
        // }

        let prev_pos = &game.ball.pos;
        if prev_pos.y + game.ball.vel.y < game.field.top_edge || prev_pos.y + game.ball.vel.y > game.field.bottom_edge {
            game.ball.vel.y *= -1.;
        }
        if prev_pos.x + game.ball.vel.x > game.field.right_edge || prev_pos.x + game.ball.vel.x < game.field.left_edge {
            game.ball.vel.x *= -1.;
        }

        for player in &game.players {
            let (collided, change_x, change_y) = colliding_with(&game.ball.pos, game.ball.r, &player);
            if !collided {
                continue;
            }
            if change_y {
                game.ball.vel.y *= -1.;
            }
            if change_x {
                game.ball.vel.x *= -1.;
            }
        }

        game.ball.pos += game.ball.vel;
        if game.ball.vel.length() > 5. {
            game.ball.vel = game.ball.vel.normalize() * 5.;
        }

        // game.ball.pos += game.ball.vel;


        // game.ball = game.ball.as_mut().and_then(|b| {
        //     if game.last_player != 1 && catch_ball(&mut game.players[0], b) {
        //         game.last_player = 1;
        //         return None;
        //     }
        //     if game.last_player != 2 && catch_ball(&mut game.players[1], b) {
        //         game.last_player = 2;
        //         return None;
        //     }
        //     Some(b.clone())
        // });
        clear_background(LIGHTGRAY);
        draw_field(&game);
        debug_collision(&game);
        draw_ball(&mut game);
        for player in &game.players {
            draw_player(player);
        }
        next_frame().await
    }
}


fn draw_ball(game: &mut Game) {
    let i = if game.team_with_ball == Team::Two { 0 } else { 1 };
    let txt = format!("r: {}, p: {}", game.ball.vel, game.ball.pos);
    draw_text(&txt, game.ball.pos.x, game.ball.pos.y + 20.0, 20.0, DARKGRAY);
    draw_circle(game.ball.pos.x, game.ball.pos.y, game.ball.r, BLACK);
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

