use bevy::{
  asset::RenderAssetUsages,
  image::{ImageSampler, ToExtents as _},
  prelude::*,
  render::render_resource::{
    AsBindGroup, TextureDataOrder, TextureDescriptor, TextureDimension, TextureFormat,
    TextureUsages,
  },
  shader::ShaderRef,
  sprite_render::{AlphaMode2d, Material2d},
};
use bytemuck::{Pod, Zeroable};

use crate::level::procgen::TileData;

const SHADER_ASSET_PATH: &str = "shaders/chunk2.wgsl";

#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
pub struct ChunkMaterial {
  pub alpha_mode: AlphaMode2d,
  #[texture(0, sample_type = "u_int")]
  pub tile_data: Handle<Image>,
}

impl Material2d for ChunkMaterial {
  fn fragment_shader() -> ShaderRef {
    SHADER_ASSET_PATH.into()
  }

  fn alpha_mode(&self) -> AlphaMode2d {
    self.alpha_mode
  }
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct PackedTileData {
  pub data: [u8; 2],  // red channel
  pub color: [u8; 4], // green and blue channels
  pub flags: u16,     // alpha channel
}

impl From<TileData> for PackedTileData {
  fn from(data: TileData) -> Self {
    Self {
      data: [
        (data.moisture * u8::MAX as f32) as u8,
        (data.biopresence * u8::MAX as f32) as u8,
      ],
      color: [0, 0, 0, 0],
      flags: 0,
    }
  }
}

pub fn make_chunk_tile_data_image(size: &UVec2, data: &[PackedTileData]) -> Image {
  Image {
    data: Some(bytemuck::cast_slice(data).to_vec()),
    data_order: TextureDataOrder::default(),
    texture_descriptor: TextureDescriptor {
      size: size.to_extents(),
      dimension: TextureDimension::D2,
      format: TextureFormat::Rgba16Uint,
      label: None,
      mip_level_count: 1,
      sample_count: 1,
      usage: TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST,
      view_formats: &[],
    },
    sampler: ImageSampler::nearest(),
    texture_view_descriptor: None,
    asset_usage: RenderAssetUsages::RENDER_WORLD | RenderAssetUsages::MAIN_WORLD,
    copy_on_resize: false,
  }
}
