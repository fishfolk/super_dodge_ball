use macroquad::color::Color;
use crate::Vec2;

#[derive(Clone, Debug)]
pub struct Ball {
    pub(crate) pos: Vec2,
    pub(crate) vel: Vec2,
    pub(crate) r: f32,
    pub(crate) rotation: f32,
    pub(crate) color: Color,
    pub(crate) collided: bool,
    pub(crate) thrown: bool,
    pub(crate) dropped: bool,
    pub(crate) in_air: bool,
    pub(crate) grabbed_by: Option<usize>,
}

impl Ball {
    pub(crate) fn picked_up(&mut self, player_index: usize) {
        self.grabbed_by = Some(player_index);
        self.collided = false;
        self.thrown = false;
        self.dropped = false;
        self.in_air = false;
        self.vel = Vec2::new(0., 0.);
    }
}

impl Ball {
    pub(crate) fn throwing(&mut self, target_pos: Vec2, thrower_position: Vec2, rotation: f32) {
        self.pos = thrower_position;
        self.vel = target_pos * 5.;
        self.thrown = true;
        self.collided = false;
        self.in_air = true;
        self.rotation = rotation;
        self.grabbed_by = None;
    }
}

impl Ball {
    pub(crate) fn default() -> Ball {
        Ball {
            pos: Default::default(),
            vel: Default::default(),
            r: 0.0,
            rotation: 0.0,
            color: Default::default(),
            collided: false,
            thrown: false,
            dropped: false,
            in_air: false,
            grabbed_by: Some(0),
        }
    }
}

impl crate::HasDirection for Ball {
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

