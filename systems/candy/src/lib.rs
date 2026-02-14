use bevy::{prelude::*, sprite_render::Material2dPlugin};

mod fireball;
mod fireball_explode;
mod lightning;
mod orb;
mod orb_shard;
mod shadow;

pub use fireball::FireballBody;
pub use fireball_explode::FireballExplosionBody;
pub use lightning::LightningShard;
pub use orb::FrozenOrb;
pub use orb_shard::FrozenOrbShard;
pub use shadow::Shadow;

use crate::{
  fireball::FireballMaterial,
  fireball_explode::{FireballExplosionMaterial, update_fireball_explosion},
  lightning::LightningShardMaterial,
  orb::FrozenOrbMaterial,
  orb_shard::FrozenOrbShardMaterial,
  shadow::{ShadowMaterial, sync_shadow_materials},
};

pub struct SysCandyPlugin;

impl Plugin for SysCandyPlugin {
  fn build(&self, app: &mut App) {
    app
      .add_plugins(Material2dPlugin::<FireballExplosionMaterial>::default())
      .add_plugins(Material2dPlugin::<LightningShardMaterial>::default())
      .add_plugins(Material2dPlugin::<FrozenOrbMaterial>::default())
      .add_plugins(Material2dPlugin::<FrozenOrbShardMaterial>::default())
      .add_plugins(Material2dPlugin::<FireballMaterial>::default())
      .add_plugins(Material2dPlugin::<ShadowMaterial>::default())
      .add_systems(Update, (sync_shadow_materials, update_fireball_explosion));
  }
}

