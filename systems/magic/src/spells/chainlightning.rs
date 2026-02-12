use bevy::prelude::*;
use sys_combat::{
  ApplyCombatEffect, CombatAreaEffect, CombatEffect, CombatEffectBlueprint, Combatant,
  CombatantRadar, DetonationTrigger, HitTestableShape, Projectile, ProjectileMovement,
};
use sys_move::{ApplyImpulse, IsoWorldCoords, Moveable, Placeable};
use utils::diff::{MAX_PROJECTILE_TRAVEL, TEAM_OTHER};

use crate::{SpellBookState, SpellDownside, SpellReady};

#[derive(Component, Debug, Clone)]
pub struct ChainlightningProjectile;

#[derive(Clone, Debug, Reflect)]
pub struct ChainlightningSpellGenerator {
  pub speed: f32,
  pub base_damage: f32,
  pub num_chains: f32,
  pub bounce_mult: f32,
  pub bounce_range: f32,
}

impl ChainlightningSpellGenerator {
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
      caster_team
    };
    let speed = 80. + self.speed;
    let max_lifetime = 2.0f32.min(MAX_PROJECTILE_TRAVEL / speed);

    cmd.entity(spawn_parent).with_child((
      ChainlightningProjectile,
      Placeable {
        layer: 5,
        location: caster.1.location + (IsoWorldCoords::from(direction * 100.)),
      },
      Moveable {
        damping: 1.0,
        net_forces: Vec2::ZERO,
        impulses: Vec::new(),
      },
      Projectile {
        lifetime: Timer::from_seconds(max_lifetime, TimerMode::Once),
        detonate_trigger: DetonationTrigger::None,
        movement: ProjectileMovement::Straight(speed * direction),
      },
      CombatAreaEffect {
        owner: caster.0,
        team,
        shape: HitTestableShape::Obb {
          half_extents: Vec2::new(100., 50.),
          rotation: direction.to_angle(),
        },
        effects: vec![CombatEffectBlueprint::Damage(self.base_damage as u32)],
        effect_tick: None,
        hit: false,
      },
    ));
  }
}

pub fn cast_chainlightning(
  evt: On<SpellReady<ChainlightningSpellGenerator>>,
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

  debug!("Casting chainlightning");

  ss.cooldown = Some(evt.cooldown.clone());

  // TODO: centralize downside application
  let direction = (nearest - pos.location).normalize_or(Vec2::Y);
  for downside in evt.downside.iter() {
    if let SpellDownside::HpDrain { strength } = downside {
      cmd.trigger(ApplyCombatEffect {
        target: evt.caster,
        effects: vec![CombatEffect::Damage(
          (evt.generator.base_damage * *strength) as u32,
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
  );
}
