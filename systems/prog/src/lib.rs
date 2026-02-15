use std::mem::discriminant;

use asset::{SpellBuilderConfig, SpellBuilderConfigLoader};
use bevy::prelude::*;
use serde::Deserialize;
use sys_combat::KillCounter;
use utils::colors::get_kills_needed_for_next;

use crate::{
  asset::{LongTermProgConfig, LongTermProgConfigLoader},
  boss::{
    spawn_boss, spawn_boss_kill_text, update_animation_state, update_boss_kill_text,
    update_boss_objectives, wait_for_boss_kills,
  },
  levelup::{
    LevelUp, PendingLevelUp,
    ui::{levelup_ui_interaction, on_levelup_ui, reroll_interactions},
  },
  spells::SpellBuilder,
};

mod asset;
pub mod boss;
pub mod death;
pub mod levelup;
pub mod spells;

pub struct SysProgPlugin;

impl Plugin for SysProgPlugin {
  fn build(&self, app: &mut App) {
    app
      .init_asset::<SpellBuilderConfig>()
      .init_asset_loader::<SpellBuilderConfigLoader>()
      .init_asset::<LongTermProgConfig>()
      .init_asset_loader::<LongTermProgConfigLoader>()
      .init_resource::<LongTermProgger>()
      .add_systems(Update, (sync_spell_builders, sync_lprog_config))
      .add_systems(
        Update,
        (
          levelup_ui_interaction,
          death::death_ui_interaction,
          reroll_interactions,
          spawn_boss,
          update_animation_state,
          update_boss_objectives,
          update_boss_kill_text,
          wait_for_boss_kills,
          boss::ui::update_boss_health_bar,
        ),
      )
      .add_systems(FixedUpdate, (levelup,))
      .add_observer(on_levelup_ui)
      .add_observer(death::spawn_death_ui)
      .add_observer(levelup::on_levelup)
      .add_observer(levelup::on_apply_levelup)
      .add_observer(levelup::on_game_restart)
      .add_observer(spawn_boss_kill_text)
      .add_observer(boss::ui::on_hide_boss_health_bar)
      .add_observer(boss::ui::on_show_boss_health_bar);
  }
}

#[derive(Component)]
pub struct Progger {
  pub level: u32,
  pub hp_gain: u32,
  pub bosses_spawned: u32,
  pub bosses_killed: u32,
}

pub enum ProggerTrophy {
  Deal1000DPS,
  Die5Times,
  DefeatFinalBoss,
}

#[derive(Deserialize, Debug, Clone)]
pub enum LongTermProgFeature {
  BetterSpellUpgrades,
  MoreChoices,
  SeeInSingle,
  SpellInfo,
  MoarHp,
  CheaperRerolls,
}

#[derive(Deserialize, Debug, Clone)]
pub struct LongTermProgDescriptor {
  pub feature: LongTermProgFeature,
  pub description: String,
  pub cost: u32,
}

#[derive(Resource)]
pub struct LongTermProgger {
  pub max_spells: usize,
  pub num_perk_choices: usize,
  pub lucidty: u32,
  pub reroll_cost: u32,
  pub runs: u32,
  pub trophies: Vec<ProggerTrophy>,
  pub active_lprog_features: Vec<LongTermProgDescriptor>,
  pub used_lucidty: u32,
  pub lprog_config_handle: Handle<LongTermProgConfig>,
  pub lprog_config: Option<LongTermProgConfig>,
  pub spell_builder_config: Handle<SpellBuilderConfig>,
  pub spell_builder: Option<SpellBuilder>,
}
impl LongTermProgger {
  pub fn has_upgrade(&self, upgrade: LongTermProgFeature) -> bool {
    self
      .active_lprog_features
      .iter()
      .any(|x| discriminant(&x.feature) == discriminant(&upgrade))
  }
  pub fn reroll_cost(&self) -> u32 {
    if self.has_upgrade(crate::LongTermProgFeature::CheaperRerolls) {
      self.reroll_cost / 2
    } else {
      self.reroll_cost
    }
  }
}
impl FromWorld for LongTermProgger {
  fn from_world(world: &mut World) -> Self {
    let asset_server = world
      .get_resource::<AssetServer>()
      .expect("Should have AssetServer");
    let builder_config = asset_server.load("spells.config.ron");
    let lprog_config = asset_server.load("prog.config.ron");

    Self {
      max_spells: 1,
      num_perk_choices: 3,
      lucidty: 0,
      runs: 0,
      trophies: Vec::new(),
      spell_builder_config: builder_config,
      spell_builder: None,
      lprog_config: None,
      lprog_config_handle: lprog_config,
      active_lprog_features: vec![],
      used_lucidty: 0,
      reroll_cost: 0,
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
fn sync_lprog_config(
  mut lprog: ResMut<LongTermProgger>,
  mut msgs: MessageReader<AssetEvent<LongTermProgConfig>>,
  assets: Res<Assets<LongTermProgConfig>>,
) {
  for msg in msgs.read() {
    if let AssetEvent::Modified { id } | AssetEvent::LoadedWithDependencies { id } = msg {
      if *id != lprog.lprog_config_handle.id() {
        continue;
      }
      let Some(config) = assets.get(*id) else {
        continue;
      };
      lprog.lprog_config = Some(config.clone());
      lprog.reroll_cost = config.reroll_cost;
    }
  }
}

fn levelup(
  mut cmd: Commands,
  qry: Query<(Entity, &KillCounter, &Progger), Without<PendingLevelUp>>,
) {
  for (e, kc, prog) in qry {
    if get_kills_needed_for_next(prog.level, kc.kills) > 0 {
      continue;
    };

    info!("levelup!");
    cmd.trigger(LevelUp { target: e });
  }
}
