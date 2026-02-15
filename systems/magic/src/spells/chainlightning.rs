use bevy::{ecs::relationship::Relationship, prelude::*};
use sys_candy::LightningShard;
use sys_combat::{
  ApplyCombatEffect, CombatAreaEffect, CombatEffect, CombatEffectBlueprint, Combatant,
  CombatantGuages, CombatantRadar, DetonatePayload, DetonationTrigger, HitTestableShape,
  Projectile, ProjectileMovement,
};
use sys_move::{ApplyImpulse, IsoWorldCoords, Moveable, Placeable};
use utils::diff::{TEAM_OTHER, get_max_projectile_lifetime};

use crate::{SpellBookState, SpellDownside, SpellReady};

#[derive(Component, Debug, Clone)]
pub struct ChainlightningProjectile {
  pub base_damage: f32,
  pub bounces: u32,
  pub bounces_left: u32,
  pub bounce_children: usize,
  pub bounce_mult: f32,
  pub speed: f32,
  pub caster: Entity,
  pub team: u8,
  pub spawn_point: Option<IsoWorldCoords>,
}

#[derive(Clone, Debug, Reflect)]
pub struct ChainlightningSpellGenerator {
  pub speed: f32,
  pub base_damage: f32,
  pub bounce_children: f32,
  pub max_bounce: f32,
  pub first_hit_damage: f32,
  pub bounce_mult: f32,
}

const SPARK_SIZE: f32 = 30.;

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
    let speed = 240. + self.speed;
    let max_lifetime = get_max_projectile_lifetime(speed);

    cmd.entity(spawn_parent).with_children(|x| {
      x.spawn((
        Visibility::default(),
        Transform::default(),
        ChainlightningProjectile {
          base_damage: self.base_damage,
          bounces: 1,
          bounce_children: self.bounce_children.floor() as usize,
          speed: self.speed,
          caster: caster.0,
          team,
          bounce_mult: self.bounce_mult,
          spawn_point: None,
          bounces_left: self.max_bounce.floor() as u32,
        },
        Placeable::mid(caster.1.location + (IsoWorldCoords::from(direction * 90.))),
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
            half_extents: Vec2::new(SPARK_SIZE, SPARK_SIZE / 5.),
            rotation: direction.to_angle(),
          },
          effects: vec![CombatEffectBlueprint::Damage(
            (self.base_damage + self.first_hit_damage) as u32,
          )],
          effect_tick: None,
          hit: false,
        },
      ))
      .with_children(|x2| {
        x2.spawn((
          LightningShard {
            size: Vec2::new(SPARK_SIZE, SPARK_SIZE),
            team,
            intensity: 1.0,
            direction: direction.to_angle(),
          },
          Transform::default().with_translation(Vec3::new(0.0, 16., -1.)),
          Visibility::default(),
        ));
      });
    });
  }
}

pub fn on_detonate_chainlightning(
  evt: On<DetonatePayload>,
  mut cmd: Commands,
  qry: Query<(&ChildOf, &ChainlightningProjectile)>,
  qry_combatants: Query<(Entity, &Combatant, &CombatantGuages, &Placeable)>,
) {
  let Some((parent, proj)) = qry.get(evt.target).ok() else {
    return;
  };
  let Some((_, hit_c, cs, hit_pos)) = evt.hit.as_ref().and_then(|x| qry_combatants.get(*x).ok())
  else {
    return;
  };

  let max_range: f32 = 600.0f32.powi(2);
  cmd.entity(evt.target).despawn();

  if proj.bounces_left == 0 {
    return;
  }

  let detonate_origin = hit_pos.location;
  let detonate_offset = hit_c.hitbox.bounding_radius() * 2.;
  let firt_detonate_point = proj.spawn_point.unwrap_or(evt.location);

  qry_combatants
    .iter()
    .filter(|(e, c, cs, pos)| {
      let dist_squared = firt_detonate_point.distance_squared(pos.location);
      cs.current_hp > 0
        && c.team != proj.team
        && dist_squared <= max_range
        && evt.hit.is_none_or(|x| x != *e)
    })
    .take(proj.bounce_children)
    .for_each(|(_, _, _, pos)| {
      cmd.entity(parent.get()).with_children(|x| {
        let direction = (pos.location - detonate_origin).normalize();
        x.spawn((
          Visibility::default(),
          Transform::default(),
          ChainlightningProjectile {
            base_damage: proj.base_damage,
            bounces: proj.bounces + 1,
            bounces_left: proj.bounces_left - 1,
            bounce_children: proj.bounce_children,
            speed: proj.speed,
            caster: proj.caster,
            team: proj.team,
            bounce_mult: proj.bounce_mult,
            spawn_point: Some(firt_detonate_point),
          },
          Placeable::mid(detonate_origin + IsoWorldCoords::from(direction * detonate_offset * 1.1)),
          Moveable::default(),
          Projectile {
            lifetime: Timer::from_seconds(0.5, TimerMode::Once),
            detonate_trigger: DetonationTrigger::Contact,
            movement: ProjectileMovement::Straight(proj.speed * direction),
          },
          CombatAreaEffect {
            owner: proj.caster,
            team: proj.team,
            shape: HitTestableShape::Obb {
              half_extents: Vec2::new(SPARK_SIZE, SPARK_SIZE / 5.),
              rotation: direction.to_angle(),
            },
            effects: vec![CombatEffectBlueprint::Damage(
              proj.base_damage as u32 * (proj.bounces + 1),
            )],
            effect_tick: None,
            hit: false,
          },
        ))
        .with_children(|x2| {
          x2.spawn((
            LightningShard {
              size: Vec2::new(SPARK_SIZE, SPARK_SIZE),
              team: proj.team,
              intensity: proj.bounces as f32,
              direction: direction.to_angle(),
            },
            Transform::default(),
            Visibility::default(),
          ));
        });
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
  );
}
