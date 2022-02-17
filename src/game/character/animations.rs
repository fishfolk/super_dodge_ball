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
pub struct PlayerAnimationParams {
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
    pub animations: PlayerAnimations,
}

impl From<PlayerAnimationParams> for AnimationParams {
    fn from(other: PlayerAnimationParams) -> Self {
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
pub struct PlayerAnimations {
    #[serde(default = "PlayerAnimations::default_idle_animation")]
    pub idle: AnimationMetadata,
    #[serde(rename = "move", default = "PlayerAnimations::default_move_animation")]
    pub moving: AnimationMetadata,
    #[serde(default = "PlayerAnimations::default_jump_animation")]
    pub jump: AnimationMetadata,
    #[serde(default = "PlayerAnimations::default_fall_animation")]
    pub fall: AnimationMetadata,
    #[serde(default = "PlayerAnimations::default_crouch_animation")]
    pub crouch: AnimationMetadata,
    #[serde(default = "PlayerAnimations::default_death_back_animation")]
    pub death_back: AnimationMetadata,
    #[serde(default = "PlayerAnimations::default_death_face_animation")]
    pub death_face: AnimationMetadata,
    #[serde(default = "PlayerAnimations::default_punch_animation")]
    pub punch: AnimationMetadata,
    #[serde(default = "PlayerAnimations::default_run_animation")]
    pub run: AnimationMetadata,
    #[serde(default = "PlayerAnimations::default_hurt_animation")]
    pub hurt: AnimationMetadata,
    #[serde(default = "PlayerAnimations::default_catching_animation")]
    pub catching: AnimationMetadata,
}

impl PlayerAnimations {
    pub fn default_idle_animation() -> AnimationMetadata {
        AnimationMetadata {
            id: Player::IDLE_ANIMATION_ID.to_string(),
            row: 0,
            frames: 7,
            fps: 12,
            is_looping: true,
        }
    }

    pub fn default_move_animation() -> AnimationMetadata {
        AnimationMetadata {
            id: Player::MOVE_ANIMATION_ID.to_string(),
            row: 1,
            frames: 7,
            fps: 10,
            is_looping: true,
        }
    }

    pub fn default_jump_animation() -> AnimationMetadata {
        AnimationMetadata {
            id: Player::JUMP_ANIMATION_ID.to_string(),
            row: 2,
            frames: 1,
            fps: 5,
            is_looping: false,
        }
    }

    pub fn default_fall_animation() -> AnimationMetadata {
        AnimationMetadata {
            id: Player::FALL_ANIMATION_ID.to_string(),
            row: 3,
            frames: 1,
            fps: 8,
            is_looping: true,
        }
    }

    pub fn default_crouch_animation() -> AnimationMetadata {
        AnimationMetadata {
            id: Player::CROUCH_ANIMATION_ID.to_string(),
            row: 4,
            frames: 1,
            fps: 8,
            is_looping: false,
        }
    }

    pub fn default_death_back_animation() -> AnimationMetadata {
        AnimationMetadata {
            id: Player::DEATH_BACK_ANIMATION_ID.to_string(),
            row: 5,
            frames: 7,
            fps: 10,
            is_looping: false,
        }
    }

    pub fn default_death_face_animation() -> AnimationMetadata {
        AnimationMetadata {
            id: Player::DEATH_FACE_ANIMATION_ID.to_string(),
            row: 6,
            frames: 7,
            fps: 10,
            is_looping: false,
        }
    }

    pub fn default_punch_animation() -> AnimationMetadata {
        AnimationMetadata {
            id: Player::PUNCH_ANIMATION_ID.to_string(),
            row: 9,
            frames: 8,
            fps: 10,
            is_looping: true,
        }
    }

    pub fn default_run_animation() -> AnimationMetadata {
        AnimationMetadata {
            id: Player::RUN_ANIMATION_ID.to_string(),
            row: 8,
            frames: 4,
            fps: 10,
            is_looping: true,
        }
    }

    pub fn default_hurt_animation() -> AnimationMetadata {
        AnimationMetadata {
            id: Player::HURT_ANIMATION_ID.to_string(),
            row: 7,
            frames: 2,
            fps: 3,
            is_looping: false,
        }
    }

    pub fn default_catching_animation() -> AnimationMetadata {
        AnimationMetadata {
            id: Player::CATCH_ANIMATION_ID.to_string(),
            row: 10,
            frames: 4,
            fps: 5,
            is_looping: true,
        }
    }
}

impl Default for PlayerAnimations {
    fn default() -> Self {
        PlayerAnimations {
            idle: Self::default_idle_animation(),
            moving: Self::default_move_animation(),
            jump: Self::default_jump_animation(),
            fall: Self::default_fall_animation(),
            crouch: Self::default_crouch_animation(),
            death_back: Self::default_death_back_animation(),
            death_face: Self::default_death_face_animation(),
            punch: Self::default_punch_animation(),
            run: Self::default_run_animation(),
            hurt: Self::default_hurt_animation(),
            catching: Self::default_catching_animation(),

        }
    }
}

impl From<Vec<AnimationMetadata>> for PlayerAnimations {
    fn from(vec: Vec<AnimationMetadata>) -> Self {
        PlayerAnimations {
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
            jump: vec
                .iter()
                .find(|anim| anim.id == Player::JUMP_ANIMATION_ID)
                .cloned()
                .unwrap(),
            fall: vec
                .iter()
                .find(|anim| anim.id == Player::FALL_ANIMATION_ID)
                .cloned()
                .unwrap(),
            crouch: vec
                .iter()
                .find(|anim| anim.id == Player::CROUCH_ANIMATION_ID)
                .cloned()
                .unwrap(),
            death_back: vec
                .iter()
                .find(|anim| anim.id == Player::DEATH_BACK_ANIMATION_ID)
                .cloned()
                .unwrap(),
            death_face: vec
                .iter()
                .find(|anim| anim.id == Player::DEATH_FACE_ANIMATION_ID)
                .cloned()
                .unwrap(),
            punch: vec
                .iter()
                .find(|anim| anim.id == Player::PUNCH_ANIMATION_ID)
                .cloned()
                .unwrap(),
            run: vec
                .iter()
                .find(|anim| anim.id == Player::RUN_ANIMATION_ID)
                .cloned()
                .unwrap(),
            hurt: vec
                .iter()
                .find(|anim| anim.id == Player::HURT_ANIMATION_ID)
                .cloned()
                .unwrap(),
            catching: vec
                .iter()
                .find(|anim| anim.id == Player::CATCH_ANIMATION_ID)
                .cloned()
                .unwrap(),
        }
    }
}

impl From<PlayerAnimations> for Vec<AnimationMetadata> {
    fn from(params: PlayerAnimations) -> Self {
        vec![
            params.idle,
            params.moving,
            params.jump,
            params.fall,
            params.crouch,
            params.death_back,
            params.death_face,
            params.punch,
            params.run,
            params.hurt,
            params.catching,
        ]
    }
}
