use asset::{SpellBuilderConfig, SpellBuilderConfigLoader};
use bevy::prelude::*;

use crate::{
  levelup::ui::{levelup_ui_interaction, on_levelup_ui},
  spells::SpellBuilder,
};

mod asset;
pub mod levelup;
pub mod spells;

pub struct SysProgPlugin;

impl Plugin for SysProgPlugin {
  fn build(&self, app: &mut App) {
    app
      .init_asset::<SpellBuilderConfig>()
      .init_asset_loader::<SpellBuilderConfigLoader>()
      .init_resource::<LongTermProgger>()
      .add_systems(Update, (sync_spell_builders,))
      .add_systems(Update, (levelup_ui_interaction,))
      .add_observer(on_levelup_ui)
      .add_observer(levelup::on_levelup)
      .add_observer(levelup::on_apply_levelup);
  }
}

#[derive(Component)]
pub struct Progger {
  pub level: u32,
  pub base_hp: u32,
  pub hp_gain: u32,
}

pub enum ProggerTrophy {
  Deal1000DPS,
  Die5Times,
  DefeatFinalBoss,
}

#[derive(Resource)]
pub struct LongTermProgger {
  pub max_spells: usize,
  pub num_perk_choices: usize,
  pub lucidty: u32,
  pub runs: u32,
  pub trophies: Vec<ProggerTrophy>,
  pub spell_builder_config: Handle<SpellBuilderConfig>,
  pub spell_builder: Option<SpellBuilder>,
}
impl FromWorld for LongTermProgger {
  fn from_world(world: &mut World) -> Self {
    let asset_server = world
      .get_resource::<AssetServer>()
      .expect("Should have AssetServer");
    let builder_config = asset_server.load("spells.config.ron");
    Self {
      max_spells: 3,
      num_perk_choices: 2,
      lucidty: 0,
      runs: 0,
      trophies: Vec::new(),
      spell_builder_config: builder_config,
      spell_builder: None,
    }
  }
}

fn sync_spell_builders(
  mut lprog: ResMut<LongTermProgger>,
  mut msgs: MessageReader<AssetEvent<SpellBuilderConfig>>,
  assets: Res<Assets<SpellBuilderConfig>>,
) {
  for msg in msgs.read() {
    if let AssetEvent::Modified { id } | AssetEvent::LoadedWithDependencies { id } = msg {
      if *id != lprog.spell_builder_config.id() {
        continue;
      }
      let Some(config) = assets.get(*id) else {
        continue;
      };
      lprog.spell_builder = Some(config.spellbuilder.clone());
    }
  }
}
