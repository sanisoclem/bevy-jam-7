use bevy::{
  prelude::*,
  render::render_resource::AsBindGroup,
  shader::ShaderRef,
  sprite_render::{AlphaMode2d, Material2d},
};

const SHADER_ASSET_PATH: &str = "shaders/chunk.wgsl";

#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
pub struct ChunkMaterial {
  #[uniform(0)]
  pub id: IVec4,
  #[uniform(1)]
  pub player_pos: Vec4,
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
