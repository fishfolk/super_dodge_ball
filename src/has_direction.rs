use crate::Vec2;

pub trait HasDirection {
    fn get_position(&self) -> Vec2;
    fn get_rotation(&self) -> f32;
    fn get_rotation_as_radian(&self) -> f32;
}

pub fn rotation_vector<T: HasDirection>(obj: &T) -> Vec2 {
    let rotation = obj.get_rotation_as_radian();
    Vec2::new(rotation.sin(), -rotation.cos())
}
