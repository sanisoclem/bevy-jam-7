use bevy::{prelude::*, sprite::Anchor, time::Stopwatch};
use sys_animation::AtlasAnimation;
use sys_combat::Combatant;
use sys_magic::{SpellBook, SpellBookGenerator};
use sys_move::{IsoWorldCoords, Placeable};
use utils::{
  self,
  dps::{
    self, TEAM_ENEMY, get_effective_dps_from_offense_score,
    get_effective_range_from_rangeness_score, get_enemy_size_from_density,
    get_max_hp_from_toughness_score,
  },
};

pub struct SysEnemyPlugin;

impl Plugin for SysEnemyPlugin {
  fn build(&self, app: &mut App) {
    app.add_systems(FixedUpdate, (spawn_enemies, despawn_enemies));
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

pub struct EnemyArtRegistry;

impl EnemyArtRegistry {
  pub fn get_enemy_art(
    &self,
    size: f32,
    power_score: f32,
    toughness_score: f32,
    rangeness_score: f32,
  ) -> (Anchor, Vec2, AtlasAnimation<EnemyAnimationState>) {
    todo!()
  }
}

#[derive(Resource)]
pub struct EnemyRegistry {
  sb_generator: SpellBookGenerator,
  enemy_art_registry: EnemyArtRegistry,
}

impl FromWorld for EnemyRegistry {
  fn from_world(world: &mut World) -> Self {
    Self {
      sb_generator: SpellBookGenerator,
      enemy_art_registry: EnemyArtRegistry,
    }
  }
}

impl EnemyRegistry {
  pub fn get_enemy(
    &self,
    current_density: f32,
    time_seconds: f32,
    location: &IsoWorldCoords,
  ) -> Option<EnemyBlueprint> {
    let power_budget = 10.0;

    let [
      density_score,
      rangeness_score,
      toughness_score,
      offense_score,
    ] = utils::dps::normalize_scores(power_budget, [1.0, 1.0, 1.0, 1.0]);

    let max_density = dps::get_density_ceiling_from_score(density_score);
    if current_density > max_density {
      return None;
    }

    let hp = get_max_hp_from_toughness_score(toughness_score);
    let effective_range = get_effective_range_from_rangeness_score(rangeness_score);
    let effective_dps = get_effective_dps_from_offense_score(offense_score);

    let enemy_size = get_enemy_size_from_density(density_score);
    let spellbook = self
      .sb_generator
      .create_spellbook(1, effective_range, effective_dps);

    let (anchor, scale, animation) = self.enemy_art_registry.get_enemy_art(
      enemy_size,
      offense_score,
      toughness_score,
      rangeness_score,
    );

    Some(EnemyBlueprint {
      spell_book: spellbook,
      combatant: Combatant {
        team: TEAM_ENEMY,
        max_hp: hp,
        hitbox: sys_combat::HitTestableShape::Circle { radius: enemy_size },
        death_behavior: sys_combat::DeathBehavior::Despawn(Timer::from_seconds(
          1.0,
          TimerMode::Once,
        )),
        regen: 0,
        regen_delay: 0,
      },
      anchor: Some(anchor),
      scale,
      animation,
    })
  }
}

#[derive(Component, Clone, Debug)]
pub struct EnemyAnimationState {
  pub facing: sys_move::MoveDirection,
  pub dead: bool,
  pub stunned: bool,
  pub reeling: bool,
  pub moving: bool,
}

#[derive(Clone, Debug)]
pub struct EnemyBlueprint {
  pub spell_book: SpellBook,
  pub combatant: Combatant,
  pub anchor: Option<Anchor>,
  pub scale: Vec2,
  pub animation: AtlasAnimation<EnemyAnimationState>,
}

fn spawn_enemies(
  mut cmd: Commands,
  registry: Res<EnemyRegistry>,
  qry: Query<(Entity, &EnemySpawner, &mut EnemySpawnerState, &Placeable)>,
  qry_enemies: Query<(&Placeable, &Enemy)>,
  time: Res<Time>,
) {
  for (spawner_entity, spawner, mut spawner_state, spawner_pos) in qry {
    spawner_state.stopwatch.tick(time.delta());
    spawner_state.cooldown.tick(time.delta());

    if !spawner_state.cooldown.just_finished() {
      continue;
    }

    // decay cooldown
    spawner_state.cooldown = Timer::from_seconds(
      spawner.initial_cooldown
        * (-spawner.cooldown_decay_rate * spawner_state.stopwatch.elapsed().as_secs_f32()).exp(),
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
      spawner_state.stopwatch.elapsed().as_secs_f32(),
      &location,
    ) else {
      continue;
    };

    cmd.entity(spawner.spawn_parent).with_child((
      Transform::default().with_scale(enemy.scale.extend(1.0)),
      Visibility::default(),
      enemy.spell_book.clone(),
      enemy.combatant.clone(),
      enemy.animation.clone(),
      Enemy {
        spawned_by: spawner_entity,
      },
    ));
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
        // cmd.entity(enemy_entity).despawn();
      }
    }
  }
}
