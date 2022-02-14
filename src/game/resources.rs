use std::collections::HashMap;
use std::path::Path;
use macroquad::prelude::*;
use macroquad::prelude::collections::storage;
use macroquad::prelude::coroutines::start_coroutine;
use macroquad::texture;
use crate::helpers::text::ToStringHelper;
use serde::{Deserialize, Serialize};
use crate::error::{Result, Error};
use crate::game::character::PlayerCharacterParams;
use crate::json::deserialize_json_file;

#[derive(Debug, Copy, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TextureKind {
    Background,
    Tileset,
    Spritesheet,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextureMetadata {
    pub id: String,
    pub path: String,
    #[serde(default, rename = "type", skip_serializing_if = "Option::is_none")]
    pub kind: Option<TextureKind>,
    #[serde(
    default,
    with = "crate::json::uvec2_opt",
    skip_serializing_if = "Option::is_none"
    )]
    pub sprite_size: Option<UVec2>,
    #[serde(default = "crate::json::default_filter_mode", with = "crate::json::FilterModeDef")]
    pub filter_mode: FilterMode,
    #[serde(default, skip)]
    pub size: Vec2,
}

#[derive(Debug, Clone)]
pub struct TextureResource {
    pub texture: Texture2D,
    pub meta: TextureMetadata,
}

pub struct Resources {
    pub assets_dir: String,
    pub textures: HashMap<String, TextureResource>,
    pub player_characters: Vec<PlayerCharacterParams>,
}

impl Resources {
    pub const TEXTURES_FILE: &'static str = "textures";
    pub const RESOURCE_FILES_EXTENSION: &'static str = "json";
    pub const PLAYER_CHARACTERS_FILE: &'static str = "player_characters";

    pub async fn new(assets_dir: &str) -> Result<Self> {
        let assets_dir_path = Path::new(assets_dir);

        let textures = {
            let mut textures = HashMap::new();
            let textures_file_path = assets_dir_path
                .join(Self::TEXTURES_FILE)
                .with_extension(Self::RESOURCE_FILES_EXTENSION);
            let metadata: Vec<TextureMetadata> = deserialize_json_file(&textures_file_path).await?;
            for meta in metadata {
                let file_path = assets_dir_path.join(&meta.path);
                let texture = load_texture(&file_path.to_string_helper()).await?;
                texture.set_filter(meta.filter_mode);
                let sprite_size = {
                    let val = meta
                        .sprite_size
                        .unwrap_or_else(|| vec2(texture.width(), texture.height()).as_u32());

                    Some(val)
                };
                let size = vec2(texture.width(), texture.height());
                let key = meta.id.clone();
                let meta = TextureMetadata {
                    sprite_size,
                    size,
                    ..meta
                };
                let res = TextureResource { texture, meta };
                textures.insert(key, res);
            }
            textures
        };

        let player_characters = {
            let path = assets_dir_path
                .join(Self::PLAYER_CHARACTERS_FILE)
                .with_extension(Self::RESOURCE_FILES_EXTENSION);

            deserialize_json_file(&path).await?
        };

        #[allow(clippy::inconsistent_struct_constructor)]
        Ok(Resources {
            assets_dir: assets_dir.to_string(),
            textures,
            player_characters,
        })
    }
}

pub async fn load_resources(assets_dir: &str) {
    let resources_loading = start_coroutine({
        let assets_dir = assets_dir.to_string();
        async move {
            let resources = match Resources::new(&assets_dir).await {
                Ok(val) => val,
                Err(err) => panic!("{}: {}", err.kind().as_str(), err),
            };
            storage::store(resources);
        }
    });

    while !resources_loading.is_done() {
        clear_background(BLACK);
        draw_text(
            &format!(
                "Loading resources {}",
                ".".repeat(((get_time() * 2.0) as usize) % 4)
            ),
            screen_width() / 2.0 - 160.0,
            screen_height() / 2.0,
            40.,
            WHITE,
        );
        next_frame().await;
    }
}
