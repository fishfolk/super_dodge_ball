use std::collections::HashMap;
use std::ops::Deref;
use macroquad::prelude::*;
use macroquad::telemetry::frame;

const PLAYER_HEIGHT: f32 = 25.;
const PLAYER_BASE: f32 = 22.;

#[derive(Eq, PartialEq, Hash)]
enum PlayerAction {
    Throw,
    Catch,
    Duck,
    MoveLeft,
    MoveRight,
    MoveForward,
}

enum FieldSide {
    OnLeft,
    OnRight,
}

#[derive(PartialEq)]
struct Player {
    pos: Vec2,
    rot: f32,
    vel: Vec2,
    life: i32,
    has_ball: bool,
    catch_radius: f32,
}


#[derive(Clone)]
struct Ball {
    pos: Vec2,
    vel: Vec2,
    shot_at: f64,
    collided: bool,
}

enum FromPosition {
    Top,
    Bottom,
    Right,
    Left,
}

fn valid_position(v: &Vec2) -> bool {
    let mut vr = Vec2::new(v.x, v.y);
    if vr.x + PLAYER_HEIGHT > screen_width() {
        return false;
    }
    if vr.x < 0. {
        return false;
    }
    if vr.y + PLAYER_HEIGHT > screen_height() {
        return false;
    }
    if vr.y < 0. {
        return false;
    }
    true
}

fn draw_player(player: &mut Player, rotation: f32) {
    let v1 = Vec2::new(
        player.pos.x + rotation.sin() * PLAYER_HEIGHT / 2.,
        player.pos.y - rotation.cos() * PLAYER_HEIGHT / 2.,
    );
    let v2 = Vec2::new(
        player.pos.x - rotation.cos() * PLAYER_BASE / 2. - rotation.sin() * PLAYER_HEIGHT / 2.,
        player.pos.y - rotation.sin() * PLAYER_BASE / 2. + rotation.cos() * PLAYER_HEIGHT / 2.,
    );
    let v3 = Vec2::new(
        player.pos.x + rotation.cos() * PLAYER_BASE / 2. - rotation.sin() * PLAYER_HEIGHT / 2.,
        player.pos.y + rotation.sin() * PLAYER_BASE / 2. + rotation.cos() * PLAYER_HEIGHT / 2.,
    );

    draw_triangle_lines(v1, v2, v3, 2., if player.has_ball { BLUE } else { BLACK });
}

fn find_rotation_vec(player: &Player) -> Vec2 {
    let player_rotation = player.rot.to_radians();
    Vec2::new(player_rotation.sin(), -player_rotation.cos())
}

fn player_facing_ball(player: &Player, ball: &Ball) -> bool {
    let rot_vec = find_rotation_vec(&player);
    let calc = rot_vec.normalize().dot(ball.vel.normalize());
    calc < 0.
}

fn catch_ball(player: &mut Player, ball: &mut Ball, last_player: i32) -> bool {
    let facing_towards_ball = player_facing_ball(&player, &ball);
    if (ball.pos - player.pos).length() < player.catch_radius {
        player.has_ball = true;
        if !facing_towards_ball {
            player.life -= 1;
            println!("player was hit! {}", player.life);
        }
        println!("P1: {}", player.rot);
        return true;
    }
    return false;
}

fn player_movement(player: &mut Player, keys: &HashMap<PlayerAction, KeyCode>) {
    let player_rotation = player.rot.to_radians();
    let mut acc = -player.vel / 10.;

    if is_key_down(keys[&PlayerAction::MoveForward]) {
        acc = Vec2::new(player_rotation.sin(), -player_rotation.cos()) / 3.;
    }
    if is_key_down(keys[&PlayerAction::MoveRight]) {
        player.rot += 5.;
    } else if is_key_down(keys[&PlayerAction::MoveLeft]) {
        player.rot -= 5.;
    }
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

fn throw_ball(player: &mut Player, frame_t: f64) -> Option<Ball> {
    let rot_vec = find_rotation_vec(&player);
    player.has_ball = false;
    Some(Ball {
        pos: player.pos + rot_vec * PLAYER_HEIGHT,
        vel: rot_vec * 3.,
        shot_at: frame_t,
        collided: false,
    })
}

#[macroquad::main("Super Dodge Ball")]
async fn main() {
    let mut player_one = Player {
        pos: Vec2::new(20., screen_height() / 2.),
        rot: 90.,
        vel: Vec2::new(0., 0.),
        life: 100,
        has_ball: true,
        catch_radius: PLAYER_HEIGHT + 2.,
    };

    let mut player_two = Player {
        pos: Vec2::new(screen_width() - PLAYER_HEIGHT - 20., screen_height() / 2.),
        rot: -90.,
        vel: Vec2::new(0., 0.),
        life: 100,
        has_ball: false,
        catch_radius: PLAYER_HEIGHT + 2.,
    };

    let mut ball: Option<Ball> = None;
    let mut last_shot = get_time();
    let mut game_over = false;
    let mut last_player = 1;

    // let mut screen_center = Vec2::new(screen_width() / 2., screen_height() / 2.);
    let player_one_keys = HashMap::from(
        [
            (PlayerAction::MoveForward, KeyCode::W),
            (PlayerAction::MoveLeft, KeyCode::A),
            (PlayerAction::MoveRight, KeyCode::D),
            (PlayerAction::Throw, KeyCode::E)
        ]
    );

    let player_two_keys = HashMap::from(
        [
            (PlayerAction::MoveForward, KeyCode::I),
            (PlayerAction::MoveLeft, KeyCode::J),
            (PlayerAction::MoveRight, KeyCode::L),
            (PlayerAction::Throw, KeyCode::O)
        ]
    );

    loop {
        let frame_t = get_time();
        let player_one_rotation = player_one.rot.to_radians();
        let player_two_rotation = player_two.rot.to_radians();

        player_movement(&mut player_one, &player_one_keys);
        player_movement(&mut player_two, &player_two_keys);

        if is_key_down(player_one_keys[&PlayerAction::Throw]) && player_one.has_ball {
            ball = throw_ball(&mut player_one, frame_t);
            last_shot = frame_t;
        }

        if is_key_down(KeyCode::O) && player_two.has_ball {
            ball = throw_ball(&mut player_two, frame_t);
            last_shot = frame_t;
        }


        ball = ball.as_ref().and_then(|b| {
            let mut b_new = b.clone();
            b_new.pos += b_new.vel;
            Some(b_new)
        });

        ball = ball.as_mut().and_then(|b| {
            if last_player != 1 && catch_ball(&mut player_one, b, last_player) {
                last_player = 1;
                return None;
            }
            if last_player != 2 && catch_ball(&mut player_two, b, last_player) {
                last_player = 2;
                return None;
            }
            Some(b.clone())
        });

        if player_one.life == 0 {
            game_over = true;
        }

        if game_over {
            continue;
        }

        clear_background(LIGHTGRAY);
        ball.as_ref().map(|b| {
            draw_circle(b.pos.x, b.pos.y, 5., BLACK);
        });

        draw_player(&mut player_one, player_one_rotation);
        draw_player(&mut player_two, player_two_rotation);

        next_frame().await
    }
}