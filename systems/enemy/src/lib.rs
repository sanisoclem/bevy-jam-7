use std::marker::PhantomData;

use bevy::{prelude::*, sprite::Anchor, time::Stopwatch};
use sys_animation::{AtlasAnimation, SysAnimationPlugin};
use sys_candy::Shadow;
use sys_combat::{Combatant, KillCounter};
use sys_magic::{SpellBook, SpellBookGenerator, SpellBookState};
use sys_move::{IsoWorldCoords, MoveDirection, MoveState, Moveable, Placeable};
use sys_procgen::ProceduralLevel;
use utils::{
  self,
  diff::{
    self, TEAM_ENEMY, get_effective_dps_from_offense_score,
    get_effective_range_from_rangeness_score, get_enemy_size_from_toughness, get_enemy_tint,
    get_max_hp_from_toughness_score, get_mobility_from_rangeness, get_power_budget_from_kills,
  },
};

use crate::asset::{EnemyDescriptor, EnemyDescriptorAssetLoader, TextureAtlasLayoutAssetLoader};

mod asset;

pub struct SysEnemyPlugin;

impl Plugin for SysEnemyPlugin {
  fn build(&self, app: &mut App) {
    app
      .add_plugins(SysAnimationPlugin::<EnemyAnimationState>::default())
      .init_asset::<EnemyDescriptor>()
      .init_resource::<EnemyRegistry>()
      .register_asset_loader(EnemyDescriptorAssetLoader)
      .register_asset_loader(TextureAtlasLayoutAssetLoader)
      .add_systems(Update, update_animation_state)
      .add_systems(
        FixedUpdate,
        (spawn_enemies, despawn_enemies, update_enemy_objectives),
      );
  }
}

#[derive(Component, Clone, Debug)]
pub struct Enemy {
  pub spawned_by: Entity,
  pub mobility: f32,
  pub desired_range: f32,
  pub spawned_at: IsoWorldCoords,
}

#[derive(Component, Clone, Debug)]
pub struct EnemyState {
  pub objective: Option<IsoWorldCoords>,
  pub idle_timer: Timer,
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
  pub disabled: bool,
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
  descriptors: Vec<Handle<EnemyDescriptor>>,
}

impl FromWorld for EnemyRegistry {
  fn from_world(world: &mut World) -> Self {
    let asset_server = world.resource::<AssetServer>();
    // let descriptor_folder = asset_server.load_folder("enemies");
    // NOTE: NO SUPPORT FOR LOADING FOLDERS IN WASM!!!!!!

    let e1 = asset_server.load("enemies/test1.enemy.ron");
    let e2 = asset_server.load("enemies/test2.enemy.ron");

    Self {
      sb_generator: SpellBookGenerator,
      descriptors: vec![e1, e2],
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
    let power_budget = get_power_budget_from_kills(total_kills as f32);

    let [
      density_score,
      rangeness_score,
      toughness_score,
      offense_score,
    ] = utils::diff::normalize_scores(
      power_budget,
      [0, 1, 2, 3].map(|layer| procgen.sample(location, layer)),
    );

    // info!(
    //   "spawning, density: {}, range: {}, tough: {}, offense: {}",
    //   density_score, rangeness_score, toughness_score, offense_score
    // );

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
        .create_spellbook(1, effective_range, effective_dps, power_budget);

    let descriptor = self.get_enemy_descriptor(descriptors, location)?;

    let scale = descriptor.scale * get_enemy_size_from_toughness(toughness_score);
    let tint = get_enemy_tint(toughness_score, rangeness_score, offense_score);
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
      offense_score,
      toughness_score,
      rangeness_score,
      power_budget,
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
  pub toughness_score: f32,
  pub rangeness_score: f32,
  pub offense_score: f32,
  pub power_budget: f32,
}

// pub fn load_enemy_registry(
//   mut ev_asset: MessageReader<AssetEvent<LoadedFolder>>,
//   mut enemy_registry: ResMut<EnemyRegistry>,
//   loaded_folders: Res<Assets<LoadedFolder>>,
// ) {
//   for ev in ev_asset.read() {
//     if !ev.is_loaded_with_dependencies(&enemy_registry.descriptor_folder) {
//       continue;
//     }
//
//     let loaded_folder = loaded_folders
//       .get(&enemy_registry.descriptor_folder)
//       .expect("folder should be loaded");
//
//     enemy_registry.descriptors = loaded_folder
//       .handles
//       .iter()
//       .cloned()
//       .filter_map(|h| h.try_typed::<EnemyDescriptor>().ok())
//       .collect();
//   }
// }

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
      if spawner_child_of.0 != procgen_entity || spawner.disabled {
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

      cmd.entity(spawner.spawn_parent).with_children(|x| {
        x.spawn((
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
            impulses: Vec::new(),
          },
          Enemy {
            spawned_by: spawner_entity,
            mobility: get_mobility_from_rangeness(enemy.power_budget, enemy.rangeness_score),
            desired_range: get_effective_range_from_rangeness_score(enemy.rangeness_score),
            spawned_at: location,
          },
          EnemyState {
            objective: None,
            idle_timer: Timer::from_seconds(2.0, TimerMode::Once),
          },
          EnemyAnimationState {
            facing: MoveDirection::Southeast,
            moving: false,
          },
        ))
        .with_children(|x2| {
          x2.spawn((
            Shadow {
              radius: enemy.combatant.hitbox.bounding_radius() * (1. / enemy.scale.x) * 0.7,
            },
            Transform::default().with_translation(-Vec3::Z),
            Visibility::default(),
          ));
        });
      });
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
fn update_enemy_objectives(
  qry_enemies: Query<(&Enemy, &mut EnemyState, &Placeable, &mut Moveable)>,
  player: Query<&Placeable, With<EnemySpawner>>,
  time: Res<Time>,
) {
  let Some(player_pos) = player.iter().next().map(|p| p.location) else {
    return;
  };

  for (enemy, mut state, placeable, mut mov) in qry_enemies {
    let dist_to_player = placeable.location.distance(player_pos);
    let player_dist_from_spawn = enemy.spawned_at.distance(player_pos);
    let is_player_nearby = dist_to_player <= enemy.desired_range * 1.2;

    if is_player_nearby {
      if state.objective.is_some() {
        state.objective = None;
        state.idle_timer.reset();
      }

      let direction = (player_pos - placeable.location).normalize_or_zero();
      let distance_error = dist_to_player - enemy.desired_range;

      // give up if player is too far (2x effective_range from spawn point)
      mov.net_forces =
        if distance_error.abs() < 5.0 || player_dist_from_spawn > enemy.desired_range * 2. {
          Vec2::ZERO
        } else if distance_error > 0.0 {
          direction
        } else {
          -direction
        } * enemy.mobility;

      continue;
    }

    if let Some(objective) = state.objective {
      let dist_to_objective = placeable.location.distance_squared(objective);
      if dist_to_objective < 25.0 {
        state.objective = None;
        mov.net_forces = Vec2::ZERO;
        state.idle_timer.reset();
      }
      continue;
    }

    if !state.idle_timer.is_finished() {
      state.idle_timer.tick(time.delta());
      continue;
    }

    let max_travel_dist = enemy.mobility * 2.0;
    let random_angle = fastrand::f32() * std::f32::consts::TAU;
    let random_dist = fastrand::f32() * max_travel_dist;
    let random_offset = IsoWorldCoords::new(
      random_angle.cos() * random_dist,
      random_angle.sin() * random_dist,
    );
    state.objective = Some(placeable.location + random_offset);
    mov.net_forces = (*random_offset).normalize() * enemy.mobility;
  }
}
fn update_animation_state(qry: Query<(&mut EnemyAnimationState, &MoveState), With<Enemy>>) {
  for (mut anim, mov) in qry {
    anim.moving = mov.is_moving_voluntary;
    anim.facing = mov.direction.clone();
  }
}
