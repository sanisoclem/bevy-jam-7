use bevy::prelude::*;
use utils::iso::IsoWorldCoords;

use crate::{
  level::{
    asset::TileDefinition,
    procgen::{ChunkTileData, TileData},
  },
  prelude::LevelChunk,
};

#[derive(Debug, Component)]
pub struct TileSpriteLevel {
  pub tileset: Handle<Image>,
  pub layout: Handle<TextureAtlasLayout>,
  pub tile_size_screen: Vec2,
  pub tile_size_world: Vec2,
  pub tiles: Vec<TileDefinition>,
}
impl TileSpriteLevel {
  pub fn get_tile(&self, d: &TileData) -> TileDefinition {
    if d.moisture < 0.5 {
      self.tiles.first().unwrap().clone()
    } else {
      self.tiles.get(1).unwrap().clone()
    }
  }
}

pub fn render_tile_data(
  mut cmd: Commands,
  qry_level: Query<(Entity, &TileSpriteLevel)>,
  mut qry_chunk: Query<(Entity, &mut ChunkTileData), With<LevelChunk>>,
  qry_children: Query<&Children>,
) {
  for (level_entity, level) in qry_level {
    let Some(children) = qry_children.get(level_entity).ok() else {
      continue;
    };

    for child in children.into_iter() {
      let Some((chunk_entity, mut tile_data)) = qry_chunk.get_mut(*child).ok() else {
        continue;
      };
      if tile_data.loaded {
        continue;
      }
      let tiles: Vec<_> = tile_data
        .data
        .iter()
        .map(|td| {
          let tile_coords: IsoWorldCoords = (td.coords.as_vec2() * level.tile_size_world).into();
          let tile = level.get_tile(td);
          let aspect_ratio = level.tile_size_screen.y / level.tile_size_screen.x;
          cmd
            .spawn((
              Transform::default().with_translation(
                (tile_coords.to_screen(aspect_ratio)
                  + Vec2::new(0., -(tile.surface_height as f32)))
                .extend((-((td.coords.x + td.coords.y) as f32)) / 10000.),
              ),
              Sprite::from_atlas_image(
                level.tileset.clone(),
                TextureAtlas {
                  layout: level.layout.clone(),
                  index: tile.index,
                },
              ),
            ))
            .id()
        })
        .collect();

      cmd.entity(chunk_entity).replace_children(tiles.as_slice());

      tile_data.loaded = true;
    }
  }
}
