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

const SHADER_ASSET_PATH: &str = "shaders/tile.wgsl";

#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
pub struct ChunkMaterial {
  pub alpha_mode: AlphaMode2d,
  #[texture(0, dimension = "2d_array")]
  #[sampler(1)]
  pub tileset: Handle<Image>,
  #[texture(2, sample_type = "u_int")]
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
  pub tileset_index: u16, // red channel
  pub color: [u8; 4],     // green and blue channels
  pub flags: u16,         // alpha channel
}

impl PackedTileData {
  fn empty() -> Self {
    Self {
      tileset_index: u16::MAX,
      color: [0, 0, 0, 0],
      flags: 0,
    }
  }
}

#[derive(Clone, Copy, Debug, Reflect)]
#[reflect(Clone, Debug, Default)]
pub struct TileData {
  /// The index of the tile in the corresponding tileset array texture.
  pub tileset_index: u16,
  /// The color tint of the tile. White leaves the sampled texture color unchanged.
  pub color: Color,
  /// The visibility of the tile.
  pub visible: bool,
}

impl TileData {
  /// Creates a new `TileData` with the given tileset index and default values.
  pub fn from_tileset_index(tileset_index: u16) -> Self {
    Self {
      tileset_index,
      ..default()
    }
  }
}

impl Default for TileData {
  fn default() -> Self {
    Self {
      tileset_index: 0,
      color: Color::WHITE,
      visible: true,
    }
  }
}
impl From<TileData> for PackedTileData {
  fn from(
    TileData {
      tileset_index,
      color,
      visible,
    }: TileData,
  ) -> Self {
    Self {
      tileset_index,
      color: color.to_srgba().to_u8_array(),
      flags: visible as u16,
    }
  }
}

impl From<Option<TileData>> for PackedTileData {
  fn from(maybe_tile_data: Option<TileData>) -> Self {
    maybe_tile_data
      .map(Into::into)
      .unwrap_or(PackedTileData::empty())
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
