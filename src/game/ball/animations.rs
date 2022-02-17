use macroquad::prelude::*;

use serde::{Deserialize, Serialize};
use crate::game::animations::{AnimationMetadata, AnimationParams};

use crate::{json, Player};

/// This is used in stead of `AnimationParams`, as we have different data requirements, in the case
/// of a player character, compared to most other use cases. We want to have a default animation
/// set, for instance, that corresponds with the way the core game characters are animated, but
/// still have the possibility to declare custom animation sets, as well as have variation in size,
///
/// Refer to `crate::components::animation_player::AnimationParams` for detailed documentation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BallAnimationParams {
    #[serde(rename = "texture")]
    pub texture_id: String,
    #[serde(default = "json::default_scale")]
    pub scale: f32,
    #[serde(default, with = "json::vec2_def")]
    pub offset: Vec2,
    #[serde(default, with = "json::vec2_opt")]
    pub pivot: Option<Vec2>,
    #[serde(
        default,
        with = "json::uvec2_opt",
        skip_serializing_if = "Option::is_none"
    )]
    pub frame_size: Option<UVec2>,
    #[serde(
        default,
        with = "json::color_opt",
        skip_serializing_if = "Option::is_none"
    )]
    pub tint: Option<Color>,
    #[serde(default)]
    pub animations: BallAnimations,
}

impl From<BallAnimationParams> for AnimationParams {
    fn from(other: BallAnimationParams) -> Self {
        AnimationParams {
            texture_id: other.texture_id,
            scale: other.scale,
            offset: other.offset,
            pivot: other.pivot,
            frame_size: other.frame_size,
            tint: other.tint,
            animations: other.animations.into(),
            should_autoplay: true,
            is_deactivated: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BallAnimations {
    #[serde(default = "BallAnimations::default_idle_animation")]
    pub idle: AnimationMetadata,
    #[serde(rename = "move", default = "BallAnimations::default_move_animation")]
    pub moving: AnimationMetadata,
}

impl BallAnimations {
    pub fn default_idle_animation() -> AnimationMetadata {
        AnimationMetadata {
            id: Player::IDLE_ANIMATION_ID.to_string(),
            row: 1,
            frames: 1,
            fps: 1,
            is_looping: false,
        }
    }

    pub fn default_move_animation() -> AnimationMetadata {
        AnimationMetadata {
            id: Player::MOVE_ANIMATION_ID.to_string(),
            row: 1,
            frames: 5,
            fps: 8,
            is_looping: true,
        }
    }

}

impl Default for BallAnimations {
    fn default() -> Self {
        BallAnimations {
            idle: Self::default_idle_animation(),
            moving: Self::default_move_animation(),
        }
    }
}

impl From<Vec<AnimationMetadata>> for BallAnimations {
    fn from(vec: Vec<AnimationMetadata>) -> Self {
        BallAnimations {
            idle: vec
                .iter()
                .find(|anim| anim.id == Player::IDLE_ANIMATION_ID)
                .cloned()
                .unwrap(),
            moving: vec
                .iter()
                .find(|anim| anim.id == Player::MOVE_ANIMATION_ID)
                .cloned()
                .unwrap(),
        }
    }
}

impl From<BallAnimations> for Vec<AnimationMetadata> {
    fn from(params: BallAnimations) -> Self {
        vec![
            params.idle,
            params.moving,
        ]
    }
}
