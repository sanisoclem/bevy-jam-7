use std::sync::Arc;

use bevy::prelude::*;
use sys_combat::{
  ApplyCombatEffect, CombatAreaEffect, CombatEffect, CombatEffectBlueprint, Combatant,
  CombatantRadar, DetonationTrigger, HitTestableShape, Projectile, ProjectileMovement,
  ProjectilePayload,
};
use sys_move::{ApplyImpulse, IsoWorldCoords, Moveable, Placeable};
use utils::diff::TEAM_OTHER;

use crate::{SpellBookState, SpellDownside, SpellReady};

#[derive(Component, Debug, Clone)]
pub struct FireballProjectile;

#[derive(Component, Debug, Clone)]
pub struct FireballExplosion;

#[derive(Clone, Debug, Reflect)]
pub struct FireballSpellGenerator {
  pub radius: f32,
  pub base_damage: u32,
  pub lifetime: f32,
  pub speed: f32,
  pub explosion_radius: f32,
  pub explosion_lifetime: f32,
  pub explosion_damage_multiplier: f32,
}

impl FireballSpellGenerator {
  pub fn cast(
    &self,
    cmd: &mut Commands,
    caster: (Entity, &Placeable),
    caster_team: u8,
    spawn_parent: Entity,
    downside: &Option<SpellDownside>,
    direction: Vec2,
  ) {
    let team = if let Some(SpellDownside::FriendFire) = downside {
      TEAM_OTHER
    } else {
      info!("friendly fire not {}", caster_team);
      caster_team
    };
    let payload_damage =
      (self.base_damage as f32 * self.explosion_damage_multiplier).floor() as u32;
    let explosion_lifetime = Timer::from_seconds(self.explosion_lifetime, TimerMode::Once);
    let explosion_shape = HitTestableShape::Circle {
      radius: self.explosion_radius,
    };

    cmd.entity(spawn_parent).with_child((
      FireballProjectile,
      Placeable {
        layer: 5,
        location: caster.1.location + direction.into(),
      },
      Moveable {
        damping: 1.0,
        net_forces: Vec2::ZERO,
        impulses: Vec::new(),
      },
      Projectile {
        lifetime: Timer::from_seconds(self.lifetime, TimerMode::Once),
        detonate_trigger: DetonationTrigger::Contact,
        payload: Some(ProjectilePayload::SpawnEntities(
          spawn_parent,
          Arc::new(
            move |cmd: &mut Commands, detonate_location: &IsoWorldCoords| {
              vec![
                cmd
                  .spawn((
                    FireballExplosion,
                    Projectile {
                      lifetime: explosion_lifetime.clone(),
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
                      impulses: Vec::new(),
                    },
                    CombatAreaEffect {
                      owner: caster.0,
                      team,
                      shape: explosion_shape.clone(),
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
        movement: ProjectileMovement::Straight(direction * self.speed),
      },
      CombatAreaEffect {
        owner: caster.0,
        team,
        shape: HitTestableShape::Circle {
          radius: self.radius,
        },
        effects: vec![CombatEffectBlueprint::Damage(self.base_damage)],
        effect_tick: None,
        hit: false,
      },
    ));
  }
}

pub fn cast_fireball(
  evt: On<SpellReady<FireballSpellGenerator>>,
  mut cmd: Commands,
  mut qry: Query<(
    &Combatant,
    &CombatantRadar,
    &Placeable,
    &ChildOf,
    &mut SpellBookState,
  )>,
) {
  let Some((c, radar, pos, parent, mut sbs)) = qry.get_mut(evt.caster).ok() else {
    return;
  };
  let Some((_, nearest)) = radar.nearest else {
    return;
  };
  let Some(ss) = sbs.spells_states.get_mut(evt.spell_slot) else {
    return;
  };

  debug!("Casting fireball");

  ss.cooldown = Some(evt.cooldown.clone());

  let direction = (nearest - pos.location).normalize_or(Vec2::Y);

  if let Some(SpellDownside::HpDrain { strength }) = &evt.downside {
    cmd.trigger(ApplyCombatEffect {
      target: evt.caster,
      effects: vec![CombatEffect::Damage(
        (evt.generator.base_damage as f32 * *strength) as u32,
      )],
      source: evt.caster,
    });
  }
  if let Some(SpellDownside::ForceMovement { strength, duration }) = &evt.downside {
    cmd.trigger(ApplyImpulse {
      target: evt.caster,
      force: -direction * strength,
      timer: Timer::from_seconds(*duration, TimerMode::Once),
    });
  }

  evt.generator.cast(
    &mut cmd,
    (evt.caster, pos),
    c.team,
    parent.0,
    &evt.downside,
    direction,
  );
}
