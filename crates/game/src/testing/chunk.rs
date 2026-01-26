use bevy::{
  camera::visibility::NoFrustumCulling,
  prelude::*,
  render::render_resource::AsBindGroup,
  shader::ShaderRef,
  sprite::Text2dShadow,
  sprite_render::{AlphaMode2d, Material2d},
};
use jam7::{level::LevelChunk, player::Player};

pub fn generate_level_chunk_mesh(
  mut cmd: Commands,
  mut meshes: ResMut<Assets<Mesh>>,
  mut materials: ResMut<Assets<ChunkMaterial>>,
  qry: Query<(&LevelChunk, Entity), Without<Mesh2d>>,
) {
  for (chunk, entity) in qry {
    // TODO: generate create resource to track meshes and materials used by chunks and
    // unload them when no longer needed
    cmd.entity(entity).insert((
      NoFrustumCulling,
      Mesh2d(meshes.add(Rectangle::from_size(Vec2::splat(chunk.size)))),
      MeshMaterial2d(materials.add(ChunkMaterial {
        id: IVec2::new(chunk.id.x(), chunk.id.y()),
        chunk_size: chunk.size,
        player_pos: Vec2::default(),
      })),
    ));
  }
}

pub fn update_chunk_player_pos(
  mut materials: ResMut<Assets<ChunkMaterial>>,
  qry: Query<&Transform, With<Player>>,
) {
  let Ok(player_transform) = qry.single() else {
    return;
  };

  for mat in materials.iter_mut() {
    mat.1.player_pos = player_transform.translation.xy();
  }
}

const SHADER_ASSET_PATH: &str = "shaders/chunk.wgsl";

#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
pub struct ChunkMaterial {
  #[uniform(0)]
  id: IVec2,
  #[uniform(1)]
  chunk_size: f32,
  #[uniform(2)]
  player_pos: Vec2,
}

impl Material2d for ChunkMaterial {
  fn vertex_shader() -> ShaderRef {
    SHADER_ASSET_PATH.into()
  }
  fn fragment_shader() -> ShaderRef {
    SHADER_ASSET_PATH.into()
  }

  fn alpha_mode(&self) -> AlphaMode2d {
    AlphaMode2d::Blend
  }
}
