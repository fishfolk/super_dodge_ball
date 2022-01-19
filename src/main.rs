use std::collections::HashMap;
use std::ops::Deref;
use macroquad::prelude::*;

const PLAYER_HEIGHT: f32 = 25.;
const PLAYER_BASE: f32 = 22.;

trait HasDirection {
    fn get_rot(&self) -> f32;
    fn get_rot_as_radian(&self) -> f32;
}

trait Controllable {
    fn get_keys(&self) -> HashMap<PlayerAction, KeyCode>;
}

#[derive(Eq, PartialEq, Hash, Debug)]
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

#[derive(PartialEq, Debug)]
struct Player {
    pos: Vec2,
    rot: f32,
    vel: Vec2,
    color: Color,

    life: i32,
    has_ball: bool,

    catch_radius: f32,
    keys: HashMap<PlayerAction, KeyCode>,
}

#[derive(Clone)]
struct Ball {
    pos: Vec2,
    vel: Vec2,
    rot: f32,
    color: Color,

    shot_at: f64,
    collided: bool,
}

impl HasDirection for Player {
    fn get_rot(&self) -> f32 {
        self.rot
    }

    fn get_rot_as_radian(&self) -> f32 {
        self.rot.to_radians()
    }
}

impl HasDirection for Ball {
    fn get_rot(&self) -> f32 {
        self.rot
    }

    fn get_rot_as_radian(&self) -> f32 {
        self.rot.to_radians()
    }
}

fn rotation_vector<T: HasDirection>(obj: &T) -> Vec2 {
    let rotation = obj.get_rot_as_radian();
    Vec2::new(rotation.sin(), -rotation.cos())
}

enum FromPosition {
    Top,
    Bottom,
    Right,
    Left,
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

    let color = if player.has_ball { BLUE } else { BLACK };
    draw_triangle(v1, v2, v3, player.color);
    draw_triangle_lines(v1, v2, v3, 3., color);
}

fn player_facing_ball(player: &Player, ball: &Ball) -> bool {
    let player_rot_vec = rotation_vector(player);
    let ball_rot_vec = rotation_vector(ball);
    let calc = player_rot_vec.dot(ball_rot_vec);
    calc < 0.
}

fn catch_ball(player: &mut Player, ball: &mut Ball) -> bool {
    let facing_towards_ball = player_facing_ball(&player, &ball);
    // ducking saves player from damage
    if is_key_down(player.keys[&PlayerAction::Duck]) {
        return false;
    }
    if (ball.pos - player.pos).length() < player.catch_radius {
        player.has_ball = true;
        if !facing_towards_ball {
            player.life -= 1;
            println!("player was hit! {}", player.life);
        } else {
            // need to make sure catch key is pressed
            if !is_key_down(player.keys[&PlayerAction::Catch]) {
                player.life -= 1;
                println!("player was hit! {}", player.life);
            } else {
                // now need to find if direction key is pressed or not
                println!("{} {}", player.rot, player.rot.to_radians());
            }
        }
        println!("P1: {}", player.rot);
        return true;
    }
    return false;
}

fn move_player(player: &mut Player) {
    let player_rotation = player.get_rot_as_radian();
    let mut acc = -player.vel / 10.;
    let keys = &player.keys;
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
    let rot_vec = rotation_vector(player);
    player.has_ball = false;
    Some(Ball {
        pos: player.pos + rot_vec * PLAYER_HEIGHT,
        vel: rot_vec * 1.,
        rot: player.rot,
        shot_at: frame_t,
        collided: false,
        color: DARKBROWN,
    })
}

struct Game {
    players: Vec<Player>,
    ball: Option<Ball>,
    last_shot: f64,
    last_player: i32,
}

impl Game {
    fn new() -> Self {
        Game {
            players: vec![
                Player {
                    pos: Vec2::new(20., screen_height() / 2.),
                    rot: 0.,
                    vel: Vec2::new(0., 0.),
                    life: 100,
                    has_ball: true,
                    catch_radius: PLAYER_HEIGHT + 2.,
                    keys: Default::default(),
                    color: BLACK,
                },
                Player {
                    pos: Vec2::new(screen_width() - PLAYER_HEIGHT - 20., screen_height() / 2.),
                    rot: 0.,
                    vel: Vec2::new(0., 0.),
                    life: 100,
                    has_ball: false,
                    catch_radius: PLAYER_HEIGHT + 2.,
                    keys: Default::default(),
                    color: DARKGRAY,
                }],
            ball: None,
            last_shot: get_time(),
            last_player: 1,
        }
    }
}

fn new_game() -> Game {
    let mut game = Game::new();
    game.players[0].keys = HashMap::from(
        [
            (PlayerAction::MoveForward, KeyCode::W),
            (PlayerAction::MoveLeft, KeyCode::A),
            (PlayerAction::MoveRight, KeyCode::D),
            (PlayerAction::Throw, KeyCode::E),
            (PlayerAction::Catch, KeyCode::Q),
            (PlayerAction::Duck, KeyCode::C),
        ]
    );

    game.players[1].keys = HashMap::from(
        [
            (PlayerAction::MoveForward, KeyCode::I),
            (PlayerAction::MoveLeft, KeyCode::J),
            (PlayerAction::MoveRight, KeyCode::L),
            (PlayerAction::Throw, KeyCode::O),
            (PlayerAction::Catch, KeyCode::U),
            (PlayerAction::Duck, KeyCode::P),
        ]
    );
    game
}

#[macroquad::main("Super Dodge Ball")]
async fn main() {
    let mut game = new_game();
    loop {
        let frame_t = get_time();
        let player_one_rotation = game.players[0].get_rot_as_radian();
        let player_two_rotation = game.players[1].get_rot_as_radian();
        if is_key_pressed(KeyCode::Enter) {
            game = new_game();
            println!("Resetting Game");
            next_frame().await;
            continue;
        }
        move_player(&mut game.players[0]);
        move_player(&mut game.players[1]);

        if is_key_down(game.players[0].keys[&PlayerAction::Throw]) && game.players[0].has_ball {
            game.ball = throw_ball(&mut game.players[0], frame_t);
            game.last_shot = frame_t;
        }
        if is_key_down(game.players[1].keys[&PlayerAction::Throw]) && game.players[1].has_ball {
            game.ball = throw_ball(&mut game.players[1], frame_t);
            game.last_shot = frame_t;
        }
        game.ball = game.ball.as_ref().and_then(|b| {
            let mut b_new = b.clone();
            b_new.pos += b_new.vel;
            Some(b_new)
        });
        game.ball = game.ball.as_mut().and_then(|b| {
            if game.last_player != 1 && catch_ball(&mut game.players[0], b) {
                game.last_player = 1;
                return None;
            }
            if game.last_player != 2 && catch_ball(&mut game.players[1], b) {
                game.last_player = 2;
                return None;
            }
            Some(b.clone())
        });
        clear_background(LIGHTGRAY);
        game.ball.as_ref().map(|b| {
            let i = if game.last_player == 2 { 0 } else { 1 };
            let txt = format!("r: {} a: {}",
                              game.players[i].rot,
                              b.pos.normalize().dot(rotation_vector(&game.players[i]).normalize()));
            draw_text(&txt, b.pos.x, b.pos.y + 20.0, 20.0, DARKGRAY);
            draw_circle(b.pos.x, b.pos.y, 5., BLACK);
        });


        let mut player = &mut game.players[0];
        let txt = format!("{}", player.life);
        draw_text(&txt, player.pos.x, player.pos.y - 20., 20.0, calculate_life_color(player.life));
        draw_player(player, player_one_rotation);

        let mut player = &mut game.players[1];
        let txt = format!("{}", player.life);
        draw_text(&txt, player.pos.x, player.pos.y - 20., 20.0, calculate_life_color(player.life));
        draw_player(player, player_two_rotation);

        next_frame().await
    }
}