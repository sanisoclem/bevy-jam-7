use crate::{level::procgen::ChunkTileData, prelude::LevelChunk};
use bevy::{camera::visibility::NoFrustumCulling, prelude::*};

mod material;
mod mesh;

pub use material::ChunkMaterial;
pub use mesh::IsoTilemapChunkMeshCache;

#[derive(Debug, Component)]
pub struct TileShaderLevel {
  pub tiles_per_chunk: u32,
  pub tile_size_screen: Vec2,
  pub tile_size_world: Vec2,
}

pub fn render_tile_data(
  mut cmd: Commands,
  asset_server: Res<AssetServer>,
  mut cache: ResMut<IsoTilemapChunkMeshCache>,
  mut meshes: ResMut<Assets<Mesh>>,
  mut images: ResMut<Assets<Image>>,
  mut materials: ResMut<Assets<material::ChunkMaterial>>,
  qry_level: Query<(Entity, &TileShaderLevel)>,
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
      let packed_tile_data: Vec<material::PackedTileData> =
        tile_data.data.iter().cloned().map(Into::into).collect();
      let tile_data_image = material::make_chunk_tile_data_image(
        &UVec2::splat(level.tiles_per_chunk),
        &packed_tile_data,
      );

      let font = asset_server.load("fonts/FiraSans-Bold.ttf");
      let text_font = TextFont {
        font: font.clone(),
        font_size: 50.0,
        ..default()
      };
      let text_justification = Justify::Center;

      let tile_data_image_handle = images.add(tile_data_image);
      let mesh = mesh::get_chunk_mesh(
        level.tiles_per_chunk as f32 * level.tile_size_world,
        level.tile_size_screen,
        &mut cache,
        &mut meshes,
      );
      cmd
        .entity(chunk_entity)
        .insert((
          NoFrustumCulling,
          Mesh2d(mesh),
          MeshMaterial2d(materials.add(material::ChunkMaterial {
            tile_data: tile_data_image_handle,
            alpha_mode: bevy::sprite_render::AlphaMode2d::Blend,
          })),
        ))
        .with_children(|x| {
          x.spawn((
            Text2d::new(format!("{:?}", tile_data.source)),
            text_font.clone(),
            TextLayout::new_with_justify(text_justification),
            TextBackgroundColor(Color::BLACK.with_alpha(0.5)),
            Transform::default().with_translation(Vec3::new(0., 100.0, 100.)),
          ));
        });

      tile_data.loaded = true;
    }
  }
}
