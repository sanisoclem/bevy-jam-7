use crate::level::render::material::PackedTileData;
use bevy::{camera::visibility::NoFrustumCulling, prelude::*};

pub use material::ChunkMaterial;
pub use mesh::IsoTilemapChunkMeshCache;
use sys_chonker::{ChunkId, LevelChunk};
use sys_move::IsoWorldCoords;
use sys_procgen::ProceduralLevel;

mod material;
mod mesh;

#[derive(Debug, Component)]
pub struct ChunkMeshGenerator {
  pub tiles_per_chunk: u32,
  pub tile_size_screen: Vec2,
  pub tile_size_world: Vec2,
}

fn generate_tile_data(
  chunk: ChunkId,
  level: &ProceduralLevel,
  meshgen: &ChunkMeshGenerator,
) -> Vec<material::PackedTileData> {
  let range = 0..meshgen.tiles_per_chunk;
  range
    .clone()
    .flat_map(|x| range.clone().map(move |y| (x, y)))
    .map(|(x, y)| {
      let coords = chunk.origin_world(meshgen.tile_size_world * meshgen.tiles_per_chunk as f32)
        + IsoWorldCoords::new(
          x as f32 * meshgen.tile_size_world.x,
          y as f32 * meshgen.tile_size_world.y,
        );
      let layers: [usize; 8] = std::array::from_fn(|i| i);
      PackedTileData {
        data: layers.map(|l| (level.sample(&coords, l) * u8::MAX as f32) as u8),
      }
    })
    .collect()
}

pub fn render_tile_data(
  mut cmd: Commands,
  // asset_server: Res<AssetServer>,
  mut cache: ResMut<IsoTilemapChunkMeshCache>,
  mut meshes: ResMut<Assets<Mesh>>,
  mut images: ResMut<Assets<Image>>,
  mut materials: ResMut<Assets<material::ChunkMaterial>>,
  qry_level: Query<(Entity, &ProceduralLevel)>,
  qry_chunk_parent: Query<(Entity, &ChunkMeshGenerator, &ChildOf)>,
  qry: Query<(Entity, &LevelChunk, &ChildOf), Without<Mesh2d>>,
) {
  for (level_entity, proc_level) in qry_level {
    for (chunk_parent_entity, meshgen, chunk_parent_child_of) in qry_chunk_parent {
      if chunk_parent_child_of.0 != level_entity {
        continue;
      }

      for (chunk_entity, chunk, chunk_child_of) in qry {
        if chunk_child_of.0 != chunk_parent_entity {
          continue;
        }

        let packed_tile_data = generate_tile_data(chunk.id, proc_level, meshgen);
        let tile_data_image = material::make_chunk_tile_data_image(
          &UVec2::splat(meshgen.tiles_per_chunk),
          &packed_tile_data,
        );

        // let font = asset_server.load("fonts/FiraSans-Bold.ttf");
        // let text_font = TextFont {
        //   font: font.clone(),
        //   font_size: 50.0,
        //   ..default()
        // };
        // let text_justification = Justify::Center;

        let tile_data_image_handle = images.add(tile_data_image);
        let mesh = mesh::get_chunk_mesh(
          meshgen.tiles_per_chunk as f32 * meshgen.tile_size_world,
          meshgen.tile_size_screen,
          &mut cache,
          &mut meshes,
        );
        cmd.entity(chunk_entity).insert((
          NoFrustumCulling,
          Mesh2d(mesh),
          MeshMaterial2d(materials.add(material::ChunkMaterial {
            tile_data: tile_data_image_handle,
            alpha_mode: bevy::sprite_render::AlphaMode2d::Blend,
          })),
        ));
        // .with_children(|x| {
        //   // x.spawn((
        //   //   Text2d::new(format!("{:?}", chunk.id)),
        //   //   text_font.clone(),
        //   //   TextLayout::new_with_justify(text_justification),
        //   //   TextBackgroundColor(Color::BLACK.with_alpha(0.5)),
        //   //   Transform::default().with_translation(Vec3::new(0., 100.0, 100.)),
        //   // ));
        // });
      }
    }
  }
}
