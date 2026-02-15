use bevy::{ecs::relationship::Relationship, prelude::*};
use sys_candy::{FrozenOrb, FrozenOrbShard, Shadow};
use sys_combat::{
  ApplyCombatEffect, CombatAreaEffect, CombatEffect, CombatEffectBlueprint, Combatant,
  CombatantRadar, DetonatePayload, DetonationTrigger, HitTestableShape, Projectile,
  ProjectileMovement,
};
use sys_move::{ApplyImpulse, IsoWorldCoords, Moveable, Placeable};
use utils::{
  diff::{MAX_PROJECTILE_TRAVEL, TEAM_OTHER},
  vecstuff::subdivide_circle,
};

use crate::{SpellBookState, SpellDownside, SpellReady};

#[derive(Component, Debug, Clone)]
pub struct FrozenorbProjectile {
  pub num_shards: usize,
  pub shard_lifetime: f32,
  pub shard_speed: f32,
  pub shard_damage: u32,
  pub caster: Entity,
  pub team: u8,
}
#[derive(Component, Debug, Clone)]
pub struct FrozenorbShard;

#[derive(Clone, Debug, Reflect)]
pub struct FrozenorbSpellGenerator {
  pub speed: f32,
  pub orb_size: f32,
  pub base_damage: f32,
  pub shard_frequency: f32,
  pub shard_lifetime: f32,
  pub shard_count: f32,
}

impl FrozenorbSpellGenerator {
  pub fn cast(
    &self,
    cmd: &mut Commands,
    caster: (Entity, &Placeable),
    caster_team: u8,
    spawn_parent: Entity,
    downside: &Option<SpellDownside>,
    direction: Vec2,
    target: Entity,
    sfx: Handle<AudioSource>,
  ) {
    let team = if let Some(SpellDownside::FriendFire) = downside {
      TEAM_OTHER
    } else {
      caster_team
    };
    let max_lifetime = 2.0f32.min(MAX_PROJECTILE_TRAVEL / self.speed);

    cmd.entity(spawn_parent).with_children(|x| {
      x.spawn((
        Visibility::default(),
        Transform::default(),
        FrozenorbProjectile {
          num_shards: self.shard_count.floor() as usize,
          shard_lifetime: self.shard_lifetime,
          shard_speed: self.speed * 1.5,
          shard_damage: (self.base_damage / self.shard_count) as u32,
          caster: caster.0,
          team,
        },
        Placeable {
          layer: 5,
          location: caster.1.location + (IsoWorldCoords::from(direction * self.orb_size * 1.1)),
        },
        Moveable {
          damping: 1.0,
          net_forces: Vec2::ZERO,
          impulses: Vec::new(),
        },
        Projectile {
          lifetime: Timer::from_seconds(max_lifetime, TimerMode::Once),
          detonate_trigger: DetonationTrigger::Pulse(Timer::from_seconds(
            1.0 / self.shard_frequency,
            TimerMode::Repeating,
          )),
          movement: ProjectileMovement::Seek {
            target,
            speed: self.speed,
            max_angular_velocity: 1.0,
          },
        },
        CombatAreaEffect {
          owner: caster.0,
          team,
          shape: HitTestableShape::Circle {
            radius: self.orb_size,
          },
          effects: vec![CombatEffectBlueprint::Damage(self.base_damage as u32)],
          effect_tick: Some(Timer::from_seconds(0.1, TimerMode::Repeating)),
          hit: false,
        },
        AudioPlayer::new(sfx),
        PlaybackSettings::default()
          .with_spatial(true)
          .with_speed(fastrand::f32() * 0.1 + 0.9),
      ))
      .with_children(|x2| {
        x2.spawn((
          Shadow {
            radius: self.orb_size * 0.9,
          },
          Transform::default().with_translation(-Vec3::Z),
          Visibility::default(),
        ));
        x2.spawn((
          FrozenOrb {
            radius: self.orb_size,
            team,
            intensity: self.base_damage,
          },
          Transform::default().with_translation(Vec3::new(0.0, 16. * 3., 10.)),
          Visibility::default(),
        ));
      });
    });
  }
}

pub fn on_frozenorb_shard_detonate(
  evt: On<DetonatePayload>,
  mut cmd: Commands,
  qry: Query<&FrozenorbShard>,
) {
  let Some(_) = qry.get(evt.target).ok() else {
    return;
  };
  cmd.entity(evt.target).despawn();
}
pub fn on_frozenorb_detonate(
  evt: On<DetonatePayload>,
  mut cmd: Commands,
  qry: Query<(&ChildOf, &FrozenorbProjectile, &Moveable)>,
  asset_server: Res<AssetServer>,
  mut sfx: Local<Option<Handle<AudioSource>>>,
) {
  let Some((parent, fp, mov)) = qry.get(evt.target).ok() else {
    return;
  };
  let sfx_handle = if sfx.is_none() {
    let new_value = asset_server.load("audio/POWERUP-7C63F39C804399D.ogg");
    *sfx = Some(new_value.clone());
    new_value
  } else {
    sfx.as_ref().unwrap().clone()
  };

  subdivide_circle(mov.net_forces.normalize_or(Vec2::Y), fp.num_shards)
    .into_iter()
    .for_each(|direction| {
      cmd.entity(parent.get()).with_children(|x| {
        x.spawn((
          FrozenorbShard,
          Projectile {
            lifetime: Timer::from_seconds(fp.shard_lifetime, TimerMode::Once),
            detonate_trigger: DetonationTrigger::None,
            movement: ProjectileMovement::Straight(direction * fp.shard_speed),
          },
          Transform::default(),
          Visibility::default(),
          Placeable {
            layer: 5,
            location: evt.location,
          },
          Moveable {
            damping: 1.0,
            net_forces: Vec2::ZERO,
            impulses: Vec::new(),
          },
          CombatAreaEffect {
            owner: fp.caster,
            team: fp.team,
            shape: HitTestableShape::Circle { radius: 15. },
            effects: vec![CombatEffectBlueprint::Damage(fp.shard_damage)],
            effect_tick: Some(Timer::from_seconds(0.1, TimerMode::Repeating)),
            hit: false,
          },
          AudioPlayer::new(sfx_handle.clone()),
          PlaybackSettings::default()
            .with_spatial(true)
            .with_speed(fastrand::f32() * 0.1 + 0.9),
        ))
        .with_children(|x2| {
          x2.spawn((
            Shadow { radius: 5. * 0.9 },
            Transform::default().with_translation(-Vec3::Z),
            Visibility::default(),
          ));
          x2.spawn((
            FrozenOrbShard {
              radius: 15.0,
              team: fp.team,
              intensity: fp.shard_damage as f32,
              direction: direction.to_angle(),
            },
            Transform::default().with_translation(Vec3::new(0.0, 16. * 3.0, 10.)),
            Visibility::default(),
          ));
        });
      });
    });
}
pub fn cast_frozenorb(
  evt: On<SpellReady<FrozenorbSpellGenerator>>,
  mut cmd: Commands,
  mut qry: Query<(
    &Combatant,
    &CombatantRadar,
    &Placeable,
    &ChildOf,
    &mut SpellBookState,
  )>,
  asset_server: Res<AssetServer>,
  mut sfx: Local<Option<Handle<AudioSource>>>,
) {
  let Some((c, radar, pos, parent, mut sbs)) = qry.get_mut(evt.caster).ok() else {
    return;
  };
  let Some((nearest_entity, nearest)) = radar.nearest else {
    return;
  };
  let Some(ss) = sbs.spells_states.get_mut(evt.spell_slot) else {
    return;
  };

  debug!("Casting frozenorb");
  let sfx_handle = if sfx.is_none() {
    let new_value = asset_server.load("audio/EXPLOSION-3069A7E3E2A80A33.ogg");
    *sfx = Some(new_value.clone());
    new_value
  } else {
    sfx.as_ref().unwrap().clone()
  };

  ss.cooldown = Some(evt.cooldown.clone());

  // TODO: centralize downside application
  let direction = (nearest - pos.location).normalize_or(Vec2::Y);
  for downside in evt.downside.iter() {
    if let SpellDownside::HpDrain { strength } = downside {
      cmd.trigger(ApplyCombatEffect {
        target: evt.caster,
        effects: vec![CombatEffect::Damage(
          (c.max_hp as f32 * (*strength / 100.)) as u32,
        )],
        source: evt.caster,
      });
    }
    if let SpellDownside::ForceMovement { strength, duration } = downside {
      cmd.trigger(ApplyImpulse {
        target: evt.caster,
        force: -direction * strength,
        timer: Timer::from_seconds(*duration, TimerMode::Once),
      });
    }
  }

  evt.generator.cast(
    &mut cmd,
    (evt.caster, pos),
    c.team,
    parent.0,
    &evt
      .downside
      .iter()
      .find(|f| matches!(f, SpellDownside::FriendFire))
      .cloned(),
    direction,
    nearest_entity,
    sfx_handle,
  );
}
