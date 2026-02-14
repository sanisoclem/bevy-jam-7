use bevy::{ecs::relationship::Relationship, prelude::*};
use sys_candy::{FireballBody, FireballExplosionBody, Shadow};
use sys_combat::{
  ApplyCombatEffect, CombatAreaEffect, CombatEffect, CombatEffectBlueprint, Combatant,
  CombatantRadar, DetonatePayload, DetonationTrigger, HitTestableShape, Projectile,
  ProjectileMovement,
};
use sys_move::{ApplyImpulse, IsoWorldCoords, Moveable, Placeable};
use utils::diff::TEAM_OTHER;

use crate::{SpellBookState, SpellDownside, SpellReady};

#[derive(Component, Debug, Clone)]
pub struct FireballProjectile {
  pub explosion_radius: f32,
  pub explosion_lifetime: f32,
  pub explosion_damage: u32,
  pub team: u8,
  pub caster: Entity,
}

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
      caster_team
    };
    let payload_damage =
      (self.base_damage as f32 * self.explosion_damage_multiplier).floor() as u32;

    let cast_location = caster.1.location + (IsoWorldCoords::from(direction * self.radius * 2.));

    cmd.entity(spawn_parent).with_children(|x| {
      x.spawn((
        Visibility::default(),
        Transform::default(),
        FireballProjectile {
          explosion_radius: self.explosion_radius,
          explosion_lifetime: self.explosion_lifetime,
          explosion_damage: payload_damage,
          caster: caster.0,
          team,
        },
        Placeable {
          layer: 5,
          location: cast_location,
        },
        Moveable {
          damping: 1.0,
          net_forces: Vec2::ZERO,
          impulses: Vec::new(),
        },
        Projectile {
          lifetime: Timer::from_seconds(self.lifetime, TimerMode::Once),
          detonate_trigger: DetonationTrigger::Contact,
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
      ))
      .with_children(|x2| {
        x2.spawn((
          Shadow {
            radius: self.radius * 0.8,
          },
          Transform::default().with_translation(-Vec3::Z),
          Visibility::default(),
        ));
        x2.spawn((
          FireballBody {
            radius: self.radius,
            intensity: self.base_damage as f32,
            team,
          },
          Transform::default().with_translation(Vec3::new(0.0, 16. * 3., 0.)),
          Visibility::default(),
        ));
      });
    });
  }
}

pub fn on_fireball_detonate(
  evt: On<DetonatePayload>,
  mut cmd: Commands,
  qry: Query<(&ChildOf, &FireballProjectile)>,
) {
  let Some((parent, fb)) = qry.get(evt.target).ok() else {
    return;
  };
  cmd.entity(evt.target).despawn();
  cmd.entity(parent.get()).with_children(|x| {
    x.spawn((
      FireballExplosion,
      Projectile {
        lifetime: Timer::from_seconds(fb.explosion_lifetime, TimerMode::Once),
        detonate_trigger: DetonationTrigger::None,
        movement: ProjectileMovement::Static,
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
        owner: fb.caster,
        team: fb.team,
        shape: HitTestableShape::Circle {
          radius: fb.explosion_radius,
        },
        effects: vec![CombatEffectBlueprint::Damage(fb.explosion_damage)],
        effect_tick: Some(Timer::from_seconds(0.5, TimerMode::Repeating)),
        hit: false,
      },
    ))
    .with_children(|x2| {
      x2.spawn(FireballExplosionBody {
        radius: fb.explosion_radius,
        intensity: fb.explosion_damage as f32,
        lifetime: Timer::from_seconds(fb.explosion_lifetime, TimerMode::Once),
        team: fb.team,
      });
    });
  });
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

  let dist = nearest - pos.location;
  if (evt.generator.lifetime * evt.generator.speed * 1.1).powi(2) <= dist.length_squared() {
    return;
  }

  debug!("Casting fireball");

  ss.cooldown = Some(evt.cooldown.clone());
  let direction = dist.normalize_or(Vec2::Y);
  for downside in evt.downside.iter() {
    if let SpellDownside::HpDrain { strength } = downside {
      cmd.trigger(ApplyCombatEffect {
        target: evt.caster,
        effects: vec![CombatEffect::Damage(
          (evt.generator.base_damage as f32 * *strength) as u32,
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
