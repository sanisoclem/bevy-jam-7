use std::marker::PhantomData;

use bevy::{asset::LoadedFolder, prelude::*, sprite::Anchor, time::Stopwatch};
use sys_animation::{AtlasAnimation, SysAnimationPlugin};
use sys_combat::{Combatant, KillCounter};
use sys_magic::{SpellBook, SpellBookGenerator, SpellBookState};
use sys_move::{IsoWorldCoords, MoveDirection, Moveable, Placeable};
use sys_procgen::ProceduralLevel;
use utils::{
  self,
  diff::{
    self, TEAM_ENEMY, get_effective_dps_from_offense_score,
    get_effective_range_from_rangeness_score, get_enemy_size_from_toughness, get_enemy_tint,
    get_max_hp_from_toughness_score, get_power_budget_from_kills,
  },
};

use crate::asset::{EnemyDescriptor, EnemyDescriptorAssetLoader, TextureAtlasLayoutAssetLoader};

mod asset;

pub struct SysEnemyPlugin;

impl Plugin for SysEnemyPlugin {
  fn build(&self, app: &mut App) {
    app
      .add_plugins(SysAnimationPlugin::<EnemyAnimationState>::default())
      .init_resource::<EnemyRegistry>()
      .init_asset::<EnemyDescriptor>()
      .register_asset_loader(EnemyDescriptorAssetLoader)
      .register_asset_loader(TextureAtlasLayoutAssetLoader)
      .add_systems(Update, load_enemy_registry)
      .add_systems(FixedUpdate, (spawn_enemies, despawn_enemies));
  }
}

#[derive(Component, Clone, Debug)]
pub struct Enemy {
  spawned_by: Entity,
}

// will be attached to the player
#[derive(Component, Clone, Debug)]
pub struct EnemySpawner {
  pub spawn_parent: Entity,
  pub no_spawn_radius: u32,
  pub spawn_radius: u32,
  pub despawn_radius: u32,
  pub initial_cooldown: f32,
  pub cooldown_decay_rate: f32,
}

#[derive(Component, Clone, Debug)]
pub struct EnemySpawnerState {
  pub stopwatch: Stopwatch,
  pub cooldown: Timer,
  // pub cached_bp: Option<EnemyBlueprint>,
}

#[derive(Resource)]
pub struct EnemyRegistry {
  sb_generator: SpellBookGenerator,
  descriptor_folder: Handle<LoadedFolder>,
  descriptors: Vec<Handle<EnemyDescriptor>>,
}

impl FromWorld for EnemyRegistry {
  fn from_world(world: &mut World) -> Self {
    let asset_server = world.resource::<AssetServer>();
    let descriptor_folder = asset_server.load_folder("enemies");

    Self {
      sb_generator: SpellBookGenerator,
      descriptor_folder,
      descriptors: Vec::new(),
    }
  }
}

impl EnemyRegistry {
  pub fn get_enemy_descriptor<'a>(
    &self,
    descriptors: &'a Assets<EnemyDescriptor>,
    location: &IsoWorldCoords,
  ) -> Option<&'a EnemyDescriptor> {
    let all: Vec<_> = self
      .descriptors
      .iter()
      .cloned()
      .filter_map(|x| descriptors.get(&x))
      .collect();
    if all.is_empty() {
      return None;
    }
    let total_prevalence: f32 = all.iter().map(|e| e.prevalence).sum();
    let divisions = Vec2::splat(total_prevalence);
    let chunk = (**location / divisions).floor();

    let hash = (chunk.x as i32).wrapping_mul(73856093) ^ (chunk.y as i32).wrapping_mul(19349663);
    let normalized = (hash.abs() as f32 / i32::MAX as f32) * total_prevalence;

    let mut cumulative = 0.0;
    for enemy in &all {
      cumulative += enemy.prevalence;
      if normalized < cumulative {
        return Some(enemy);
      }
    }
    None
  }
  pub fn get_enemy(
    &self,
    current_density: f32,
    total_kills: u32,
    location: &IsoWorldCoords,
    descriptors: &Assets<EnemyDescriptor>,
    procgen: &ProceduralLevel,
  ) -> Option<EnemyBlueprint> {
    let power_budget = get_power_budget_from_kills(total_kills as f32 + 100000.);

    let [
      density_score,
      rangeness_score,
      toughness_score,
      offense_score,
    ] = utils::diff::normalize_scores(
      power_budget,
      [0, 1, 2, 3].map(|layer| procgen.sample(location, layer)),
    );

    info!(
      "spawning, density: {}, range: {}, tough: {}, offense: {}",
      density_score, rangeness_score, toughness_score, offense_score
    );

    let max_density = diff::get_density_ceiling_from_score(density_score);
    if current_density > max_density {
      return None;
    }
    debug!(
      "current enemy density {} vs max {}",
      current_density * 10000.,
      max_density * 10000.
    );

    let hp = get_max_hp_from_toughness_score(toughness_score);
    let effective_range = get_effective_range_from_rangeness_score(rangeness_score);
    let effective_dps = get_effective_dps_from_offense_score(offense_score);

    let (spellbook, sb_state) =
      self
        .sb_generator
        .create_spellbook(1, effective_range, effective_dps);

    let descriptor = self.get_enemy_descriptor(descriptors, location)?;

    let scale = descriptor.scale * get_enemy_size_from_toughness(toughness_score);
    let tint = get_enemy_tint(0., rangeness_score, offense_score);
    let animation = AtlasAnimation {
      tint: Some(tint),
      phantom: PhantomData,
      animations: descriptor.animations.clone(),
      default_animation: descriptor
        .animations
        .iter()
        .map(|(_x, y)| y)
        .next()
        .cloned()
        .expect("must have at least one animation"),
    };

    Some(EnemyBlueprint {
      spell_book: spellbook,
      spell_book_state: sb_state,
      combatant: Combatant {
        team: TEAM_ENEMY,
        max_hp: hp,
        hitbox: descriptor.hitbox.clone(),
        death_behavior: sys_combat::DeathBehavior::Despawn(Timer::from_seconds(
          1.0,
          TimerMode::Once,
        )),
        regen: 0,
        regen_delay: 0,
      },
      anchor: Some(descriptor.anchor),
      scale,
      animation,
    })
  }
}

#[derive(Component, Clone, Debug, Eq, PartialEq, Hash)]
pub struct EnemyAnimationState {
  pub facing: sys_move::MoveDirection,
  // pub dead: bool,
  // pub stunned: bool,
  // pub reeling: bool,
  pub moving: bool,
}

#[derive(Clone, Debug)]
pub struct EnemyBlueprint {
  pub spell_book: SpellBook,
  pub spell_book_state: SpellBookState,
  pub combatant: Combatant,
  pub anchor: Option<Anchor>,
  pub scale: Vec2,
  pub animation: AtlasAnimation<EnemyAnimationState>,
}

pub fn load_enemy_registry(
  mut ev_asset: MessageReader<AssetEvent<LoadedFolder>>,
  mut enemy_registry: ResMut<EnemyRegistry>,
  loaded_folders: Res<Assets<LoadedFolder>>,
) {
  for ev in ev_asset.read() {
    if !ev.is_loaded_with_dependencies(&enemy_registry.descriptor_folder) {
      continue;
    }

    let loaded_folder = loaded_folders
      .get(&enemy_registry.descriptor_folder)
      .expect("folder should be loaded");

    enemy_registry.descriptors = loaded_folder
      .handles
      .iter()
      .cloned()
      .filter_map(|h| h.try_typed::<EnemyDescriptor>().ok())
      .collect();
  }
}

fn spawn_enemies(
  mut cmd: Commands,
  registry: Res<EnemyRegistry>,
  descriptors: Res<Assets<EnemyDescriptor>>,
  mut qry: Query<(
    Entity,
    &EnemySpawner,
    &mut EnemySpawnerState,
    &Placeable,
    &ChildOf,
    &KillCounter,
  )>,
  qry_enemies: Query<(&Placeable, &Enemy)>,
  qry_procgen: Query<(Entity, &ProceduralLevel)>,
  time: Res<Time>,
) {
  for (procgen_entity, procgen_level) in qry_procgen {
    for (spawner_entity, spawner, mut spawner_state, spawner_pos, spawner_child_of, kills) in
      qry.iter_mut()
    {
      if spawner_child_of.0 != procgen_entity {
        continue;
      }

      spawner_state.stopwatch.tick(time.delta());
      spawner_state.cooldown.tick(time.delta());

      if !spawner_state.cooldown.just_finished() {
        continue;
      }

      // decay cooldown
      spawner_state.cooldown = Timer::from_seconds(
        spawner.initial_cooldown
          * (-spawner.cooldown_decay_rate
            * (spawner_state.stopwatch.elapsed().as_secs_f32() / 60.).floor())
          .exp(),
        TimerMode::Once,
      );

      // calculate enemy density
      let spawn_radius_sqd = (spawner.spawn_radius as f32).powi(2);

      let enemy_count = qry_enemies
        .iter()
        .filter(|(enemy_pos, enemy)| {
          enemy.spawned_by == spawner_entity
            && spawner_pos.location.distance_squared(enemy_pos.location) <= spawn_radius_sqd
        })
        .count();

      let current_enemy_density = enemy_count as f32 / (spawner.spawn_radius as f32 * 2.).powi(2);

      // find spawn location
      let angle = fastrand::f32() * std::f32::consts::TAU;
      let min_dist = spawner.no_spawn_radius as f32;
      let max_dist = spawner.spawn_radius as f32;
      let distance = min_dist + fastrand::f32() * (max_dist - min_dist);

      let offset = Vec2::from_angle(angle) * distance;
      let location = spawner_pos.location + IsoWorldCoords::from(offset);

      let Some(enemy) = registry.get_enemy(
        current_enemy_density,
        kills.kills,
        &location,
        &descriptors,
        procgen_level,
      ) else {
        continue;
      };

      cmd.entity(spawner.spawn_parent).with_child((
        Transform::default().with_scale(enemy.scale.extend(enemy.scale.x)),
        Visibility::default(),
        enemy.spell_book.clone(),
        enemy.spell_book_state.clone(),
        enemy.combatant.clone(),
        enemy.animation.clone(),
        enemy.anchor.unwrap_or_default(),
        Placeable { location, layer: 5 },
        Moveable {
          net_forces: Vec2::ZERO,
          damping: 1.0,
        },
        Enemy {
          spawned_by: spawner_entity,
        },
        EnemyAnimationState {
          facing: MoveDirection::Southeast,
          moving: false,
        },
      ));
    }
  }
}

fn despawn_enemies(
  mut cmd: Commands,
  qry_enemies: Query<(Entity, &Placeable, &Enemy)>,
  qry_spawner: Query<(Entity, &EnemySpawner, &Placeable)>,
) {
  for (spawner_entity, spawner, spawner_pos) in qry_spawner {
    for (enemy_entity, enemy_pos, enemy) in qry_enemies {
      if enemy.spawned_by != spawner_entity {
        continue;
      }

      let despawn_dist_sqd = (spawner.despawn_radius as f32).powi(2);
      let dist_sqd = spawner_pos.location.distance_squared(enemy_pos.location);

      if dist_sqd >= despawn_dist_sqd {
        cmd.entity(enemy_entity).despawn();
      }
    }
  }
}
