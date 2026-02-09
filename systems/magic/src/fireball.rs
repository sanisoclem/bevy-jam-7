use std::sync::Arc;

use bevy::prelude::*;
use sys_combat::{
  CombatAreaEffect, CombatEffectBlueprint, DetonationTrigger, Projectile, ProjectileMovement,
  ProjectilePayload,
};
use sys_move::{IsoWorldCoords, Moveable, Placeable};

use crate::{CastSpell, SpellInstance};

#[derive(Component, Debug, Clone)]
pub struct FireballProjectile;

#[derive(Component, Debug, Clone)]
pub struct FireballExplosion;

pub fn cast_fireball(mut cmd: Commands, mut msg_reader: MessageReader<CastSpell>) {
  for msg in msg_reader.read() {
    let SpellInstance::Fireball {
      source,
      direction: _direction,
      shape,
      base_damage,
      lifetime,
      speed,
      target,
      explosion_damage_multiplier,
      explosion_lifetime,
      explosion_shape,
    } = &msg.spell;

    info!("Casting fireball");

    let payload_lifetime = explosion_lifetime.clone();
    let payload_damage = (*base_damage as f32 * explosion_damage_multiplier).floor() as u32;
    let payload_shape = explosion_shape.clone();
    let payload_owner = msg.caster;
    let payload_team = msg.team;

    cmd.entity(msg.spawn_parent).with_child((
      FireballProjectile,
      Placeable {
        layer: 5,
        location: *source,
      },
      Moveable {
        damping: 1.0,
        net_forces: Vec2::ZERO,
      },
      Projectile {
        lifetime: lifetime.clone(),
        detonate_trigger: DetonationTrigger::Contact,
        payload: Some(ProjectilePayload::SpawnEntities(
          msg.spawn_parent,
          Arc::new(
            move |cmd: &mut Commands, detonate_location: &IsoWorldCoords| {
              vec![
                cmd
                  .spawn((
                    FireballExplosion,
                    Projectile {
                      lifetime: payload_lifetime.clone(),
                      detonate_trigger: DetonationTrigger::Expiry,
                      payload: None,
                      movement: ProjectileMovement::Static,
                    },
                    Transform::default(),
                    Visibility::default(),
                    Placeable {
                      layer: 5,
                      location: *detonate_location,
                    },
                    Moveable {
                      damping: 1.0,
                      net_forces: Vec2::ZERO,
                    },
                    CombatAreaEffect {
                      owner: payload_owner,
                      team: payload_team,
                      shape: payload_shape.clone(),
                      effects: vec![CombatEffectBlueprint::Damage(payload_damage)],
                      effect_tick: Some(Timer::from_seconds(0.5, TimerMode::Repeating)),
                      hit: false,
                    },
                  ))
                  .id(),
              ]
            },
          ),
        )),
        movement: ProjectileMovement::Seek {
          target: *target,
          speed: *speed,
          max_angular_velocity: 1.0,
        },
      },
      CombatAreaEffect {
        owner: msg.caster,
        team: msg.team,
        shape: shape.clone(),
        effects: vec![CombatEffectBlueprint::Damage(*base_damage)],
        effect_tick: None,
        hit: false,
      },
    ));
  }
}
