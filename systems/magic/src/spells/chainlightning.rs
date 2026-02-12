use bevy::{ecs::relationship::Relationship, prelude::*};
use sys_combat::{
  ApplyCombatEffect, CombatAreaEffect, CombatEffect, CombatEffectBlueprint, Combatant,
  CombatantRadar, DetonatePayload, DetonationTrigger, HitTestableShape, Projectile,
  ProjectileMovement,
};
use sys_move::{ApplyImpulse, IsoWorldCoords, Moveable, Placeable};
use utils::diff::{TEAM_OTHER, get_max_projectile_lifetime};

use crate::{SpellBookState, SpellDownside, SpellReady};

#[derive(Component, Debug, Clone)]
pub struct ChainlightningProjectile {
  pub bounce_range: f32,
  pub base_damage: f32,
  pub bounces: u32,
  pub bounce_children: usize,
  pub bounce_mult: f32,
  pub speed: f32,
  pub caster: Entity,
  pub team: u8,
}

#[derive(Clone, Debug, Reflect)]
pub struct ChainlightningSpellGenerator {
  pub speed: f32,
  pub base_damage: f32,
  pub bounce_children: f32,
  pub bounce_range: f32,
  pub bounce_mult: f32,
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
    let max_lifetime = get_max_projectile_lifetime(speed);

    cmd.entity(spawn_parent).with_child((
      ChainlightningProjectile {
        base_damage: self.base_damage,
        bounce_range: self.bounce_range,
        bounces: 1,
        bounce_children: self.bounce_children.floor() as usize,
        speed: self.speed,
        caster: caster.0,
        team,
        bounce_mult: self.bounce_mult,
      },
      Placeable::mid(caster.1.location + (IsoWorldCoords::from(direction * 30.))),
      Moveable::default(),
      Projectile {
        lifetime: Timer::from_seconds(max_lifetime, TimerMode::Once),
        detonate_trigger: DetonationTrigger::Contact,
        movement: ProjectileMovement::Straight(speed * direction),
      },
      CombatAreaEffect {
        owner: caster.0,
        team,
        shape: HitTestableShape::Obb {
          half_extents: Vec2::new(10., 2.),
          rotation: direction.to_angle(),
        },
        effects: vec![CombatEffectBlueprint::Damage(self.base_damage as u32)],
        effect_tick: None,
        hit: false,
      },
    ));
  }
}

pub fn on_detonate_chainlightning(
  evt: On<DetonatePayload>,
  mut cmd: Commands,
  qry: Query<(&ChildOf, &ChainlightningProjectile)>,
  qry_combatants: Query<(&Combatant, &Placeable)>,
) {
  let Some((parent, proj)) = qry.get(evt.target).ok() else {
    return;
  };
  let max_lifetime = get_max_projectile_lifetime(proj.speed);
  let max_range: f32 = proj.bounce_range.powi(2);

  cmd.entity(evt.target).despawn();

  if proj.bounces > 5 {
    return;
  }
  qry_combatants
    .iter()
    .filter(|(c, pos)| {
      let dist_squared = evt.location.distance_squared(pos.location);
      c.team != proj.team && dist_squared > 2500. && dist_squared <= max_range
    })
    .take(proj.bounce_children)
    .for_each(|(_, pos)| {
      cmd.entity(parent.get()).with_children(|x| {
        let direction = (pos.location - evt.location).normalize();
        x.spawn((
          ChainlightningProjectile {
            base_damage: proj.base_damage,
            bounce_range: proj.bounce_range,
            bounces: proj.bounces + 1,
            bounce_children: proj.bounce_children,
            speed: proj.speed,
            caster: proj.caster,
            team: proj.team,
            bounce_mult: proj.bounce_mult,
          },
          Placeable::mid(evt.location),
          Moveable::default(),
          Projectile {
            lifetime: Timer::from_seconds(max_lifetime, TimerMode::Once),
            detonate_trigger: DetonationTrigger::Contact,
            movement: ProjectileMovement::Straight(proj.speed * direction),
          },
          CombatAreaEffect {
            owner: proj.caster,
            team: proj.team,
            shape: HitTestableShape::Obb {
              half_extents: Vec2::new(10., 2.),
              rotation: direction.to_angle(),
            },
            effects: vec![CombatEffectBlueprint::Damage(
              proj.base_damage as u32 * (proj.bounces + 1),
            )],
            effect_tick: None,
            hit: false,
          },
        ));
      });
    });
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
