use bevy::prelude::*;
use sys_combat::{
  CombatAreaEffect, CombatEffectBlueprint, DetonationTrigger, Projectile, ProjectileMovement,
};
use sys_move::{Moveable, Placeable};

use crate::{CastSpell, SpellInstance};

pub fn cast_fireball(mut cmd: Commands, mut msg_reader: MessageReader<CastSpell>) {
  for msg in msg_reader.read() {
    let SpellInstance::Fireball {
      source,
      direction,
      shape,
      base_damage,
      lifetime,
      speed,
      target,
    } = &msg.spell;

    info!("Casting fireball");

    cmd.entity(msg.spawn_parent).with_child((
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
        payload: None,
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
