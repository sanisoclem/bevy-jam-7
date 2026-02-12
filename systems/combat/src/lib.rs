use bevy::prelude::*;
use sys_move::{IsoWorldCoords, Placeable};

mod hittest;
mod projectile;

pub use hittest::HitTestableShape;
pub use projectile::{DetonatePayload, DetonationTrigger, Projectile, ProjectileMovement};

pub struct SysCombatPlugin;

impl Plugin for SysCombatPlugin {
  fn build(&self, app: &mut App) {
    app
      .add_message::<DamageTaken>()
      .add_message::<CombatantKilled>()
      .add_systems(Update, update_kill_counters)
      .add_systems(
        FixedUpdate,
        (
          create_combat_guages,
          test_hitboxes,
          tick_guages,
          despawn_respawn_dead,
          sync_combat_state,
          sync_combat_radar,
        )
          .chain(),
      )
      .add_systems(
        FixedUpdate,
        (
          projectile::update_movement_forces,
          projectile::despawn_expired_projectiles,
          projectile::pulse_projectiles,
        ),
      )
      .add_observer(apply_combat_effects);
  }
}

#[derive(Component, Reflect, Debug, Clone)]
pub struct Combatant {
  pub team: u8,
  pub max_hp: u32,
  pub regen: u32,
  pub regen_delay: u32,
  pub hitbox: HitTestableShape,
  pub death_behavior: DeathBehavior,
}

#[derive(Component, Debug, Clone, Default)]
pub struct KillCounter {
  pub kills: u32,
}

#[derive(Reflect, Debug, Clone)]
pub enum DeathBehavior {
  Respawn(Timer, Timer),
  Despawn(Timer),
}

#[derive(Component, Reflect, Debug, Clone, Default)]
pub struct CombatantRadar {
  pub nearest: Option<(Entity, IsoWorldCoords)>,
  pub strongest: Option<(Entity, IsoWorldCoords)>,
  pub densest: Option<(Entity, IsoWorldCoords)>,
}

#[derive(Component, Reflect, Debug, Clone)]
pub struct CombatantGuages {
  pub reeling_timer: Option<Timer>,
  pub invulnerability_timer: Option<Timer>,
  pub stun_timer: Option<Timer>,
  pub death_timer: Option<Timer>,
  pub current_hp: u32,
}

#[derive(Component, Reflect, Debug, Clone, Default)]
pub struct CombatantState {
  pub reeling: bool,
  pub stunned: bool,
  pub invulnerable: bool,
  pub dead: bool,
}

#[derive(Component, Reflect, Debug, Clone)]
pub struct CombatAreaEffect {
  pub owner: Entity,
  pub team: u8,
  pub shape: HitTestableShape,
  pub effect_tick: Option<Timer>,
  pub effects: Vec<CombatEffectBlueprint>,
  pub hit: bool,
}

#[derive(Reflect, Debug, Clone)]
pub enum CombatEffectBlueprint {
  Damage(u32),
  Stun(f32),
  Reeling(f32),
  SnipeDamage(u32, f32, IsoWorldCoords),
  // more damage the farther it goes
  // Knockback(f32, f32), // magnitude and falloff
  // Gravity(f32, f32),
  // Infection { delay_seconds: f32, lifetime: u8 },
}

#[derive(Reflect, Debug, Clone)]
pub enum CombatEffect {
  Damage(u32),
  Stun(f32),
  Reeling(f32),
}

#[derive(Reflect, Debug, Clone, Event)]
pub struct ApplyCombatEffect {
  pub effects: Vec<CombatEffect>,
  pub target: Entity,
  pub source: Entity,
}

#[derive(Reflect, Debug, Clone, Message)]
pub struct DamageTaken {
  pub amount: u32,
  pub target: Entity,
  pub source: Entity,
}

#[derive(Reflect, Debug, Clone, Message)]
pub struct CombatantKilled {
  pub killer: Entity,
  pub victim: Entity,
}

fn create_combat_guages(
  mut cmd: Commands,
  qry: Query<(Entity, &Combatant), Without<CombatantGuages>>,
) {
  for (entity, c) in qry {
    let Ok(mut ecmd) = cmd.get_entity(entity) else {
      continue;
    };

    ecmd.insert((
      CombatantGuages {
        current_hp: c.max_hp,
        reeling_timer: None,
        invulnerability_timer: None,
        stun_timer: None,
        death_timer: None,
      },
      CombatantState {
        reeling: false,
        stunned: false,
        invulnerable: false,
        dead: false,
      },
      CombatantRadar::default(),
    ));
  }
}

fn tick_guages(qry: Query<&mut CombatantGuages>, time: Res<Time>) {
  for mut c in qry {
    if let Some(invuln_timer) = c.invulnerability_timer.as_mut() {
      invuln_timer.tick(time.delta());
      if invuln_timer.just_finished() {
        c.invulnerability_timer = None;
      }
    }
    if let Some(reeling_timer) = c.reeling_timer.as_mut() {
      reeling_timer.tick(time.delta());
      if reeling_timer.just_finished() {
        c.reeling_timer = None;
      }
    }

    if let Some(stun_timer) = c.stun_timer.as_mut() {
      stun_timer.tick(time.delta());
      if stun_timer.just_finished() {
        c.stun_timer = None;
      }
    }
  }
}

fn test_hitboxes(
  mut cmd: Commands,
  qry_hitboxes: Query<(Entity, &Combatant, &CombatantGuages, &Placeable)>,
  qry_effects: Query<(
    Entity,
    &mut CombatAreaEffect,
    &Placeable,
    Option<&Projectile>,
  )>,
  time: Res<Time>,
) {
  for (effect_entity, mut eb, eb_pos, maybe_projectile) in qry_effects {
    if let Some(effect_timer) = eb.effect_tick.as_mut() {
      effect_timer.tick(time.delta());
      if !effect_timer.just_finished() {
        continue;
      }
    }

    for (hb_entity, hb, hbg, hb_pos) in qry_hitboxes {
      if hbg.current_hp == 0
        || hb.team == eb.team
        || hbg
          .invulnerability_timer
          .as_ref()
          .is_some_and(|x| !x.is_finished())
      {
        continue;
      }
      if !hb
        .hitbox
        .hit_test(&hb_pos.location, &eb.shape, &eb_pos.location)
      {
        continue;
      }

      let effects: Vec<CombatEffect> = eb
        .effects
        .iter()
        .map(|e| match e {
          CombatEffectBlueprint::Damage(base_dmg) => CombatEffect::Damage(*base_dmg),
          CombatEffectBlueprint::SnipeDamage(base_dmg, distance_multiplier, source) => {
            let dist = source.distance(hb_pos.location);
            CombatEffect::Damage(*base_dmg + (*base_dmg as f32 * dist * distance_multiplier) as u32)
          }
          CombatEffectBlueprint::Stun(duration) => CombatEffect::Stun(*duration),
          CombatEffectBlueprint::Reeling(duration) => CombatEffect::Reeling(*duration),
        })
        .collect();

      cmd.trigger(ApplyCombatEffect {
        effects,
        target: hb_entity,
        source: eb.owner,
      });

      if !eb.hit {
        eb.hit = true;
        if let Some(DetonationTrigger::Contact) =
          maybe_projectile.as_ref().map(|x| &x.detonate_trigger)
        {
          cmd.trigger(DetonatePayload {
            target: effect_entity,
            location: eb_pos.location,
            hit: Some(hb_entity),
          });
        }
      }
    }
  }
}

fn apply_combat_effects(
  msg: On<ApplyCombatEffect>,
  mut qry: Query<(&Combatant, &mut CombatantGuages, &CombatantState)>,
  mut msg_writer: MessageWriter<DamageTaken>,
  mut kill_writer: MessageWriter<CombatantKilled>,
) {
  let Ok((c, mut g, s)) = qry.get_mut(msg.target) else {
    return;
  };
  if s.dead || s.invulnerable {
    return;
  }

  for eff in msg.effects.iter() {
    match eff {
      CombatEffect::Damage(amount) => {
        if g.current_hp == 0 {
          continue;
        }

        let dealt = if *amount > g.current_hp {
          g.current_hp
        } else {
          *amount
        };

        g.current_hp -= dealt;

        msg_writer.write(DamageTaken {
          target: msg.target,
          source: msg.source,
          amount: dealt,
        });

        if g.current_hp == 0 {
          g.death_timer = match &c.death_behavior {
            DeathBehavior::Respawn(t, _) => t.clone(),
            DeathBehavior::Despawn(t) => t.clone(),
          }
          .into();

          kill_writer.write(CombatantKilled {
            killer: msg.source,
            victim: msg.target,
          });
        }
      }
      CombatEffect::Reeling(duration) => {
        if g
          .reeling_timer
          .as_ref()
          .is_some_and(|x| x.remaining_secs() > *duration)
        {
          return;
        }
        g.reeling_timer = Some(Timer::from_seconds(*duration, TimerMode::Once))
      }
      CombatEffect::Stun(duration) => {
        if g
          .stun_timer
          .as_ref()
          .is_some_and(|x| x.remaining_secs() > *duration)
        {
          return;
        }
        g.stun_timer = Some(Timer::from_seconds(*duration, TimerMode::Once))
      }
    }
  }
}

fn sync_combat_state(qry: Query<(&CombatantGuages, &mut CombatantState)>) {
  for (cg, mut cs) in qry {
    cs.reeling = cg.reeling_timer.as_ref().is_some_and(|x| !x.is_finished());
    cs.stunned = cg.stun_timer.as_ref().is_some_and(|x| !x.is_finished());
    cs.invulnerable = cg
      .invulnerability_timer
      .as_ref()
      .is_some_and(|x| !x.is_finished());
    cs.dead = cg.current_hp == 0;
  }
}
fn sync_combat_radar(
  mut qry: Query<(
    Entity,
    &Combatant,
    &mut CombatantRadar,
    &CombatantGuages,
    &Placeable,
  )>,
) {
  let data: Vec<_> = qry
    .iter()
    .map(|(e, c, _, cg, p)| (e, c.team, cg.current_hp, p.location))
    .collect();

  for (_e1, c1, mut cr1, cg1, p1) in &mut qry {
    if cg1.current_hp == 0 {
      cr1.nearest = None;
      cr1.strongest = None;
      cr1.densest = None;
      continue;
    }

    let mut strongest_hp = 0u32;
    let mut strongest_pos = None;
    let mut nearest_dist = f32::MAX;
    let mut nearest_pos: Option<(Entity, IsoWorldCoords)> = None;

    let mut quadrant_counts = [0usize; 4];
    let mut quadrant_last: [Option<(Entity, IsoWorldCoords)>; 4] = [None, None, None, None];

    for (e2, team, hp, location) in &data {
      if *team == c1.team || *hp == 0 {
        continue;
      }

      let dist = p1.location.distance(*location);

      if dist < nearest_dist {
        nearest_dist = dist;
        nearest_pos = Some((*e2, *location));
      }
      if hp > &strongest_hp {
        strongest_hp = *hp;
        strongest_pos = Some((*e2, *location));
      }

      // determine quadrant relative to p1
      let delta = *location - p1.location;
      let quadrant = match (delta.x >= 0.0, delta.y >= 0.0) {
        (true, true) => 0,   // NE
        (false, true) => 1,  // NW
        (false, false) => 2, // SW
        (true, false) => 3,  // SE
      };

      quadrant_counts[quadrant] += 1;
      quadrant_last[quadrant] = Some((*e2, *location));
    }

    let densest_quadrant = quadrant_counts
      .iter()
      .enumerate()
      .max_by_key(|(_, count)| *count)
      .map(|(idx, _)| idx);

    let densest_pos = densest_quadrant.and_then(|idx| quadrant_last[idx]);

    cr1.nearest = nearest_pos;
    cr1.strongest = strongest_pos;
    cr1.densest = densest_pos;
  }
}

fn despawn_respawn_dead(
  mut cmd: Commands,
  qry: Query<(Entity, &Combatant, &mut CombatantGuages)>,
  time: Res<Time>,
) {
  for (e, c, mut cg) in qry {
    let Some(death_timer) = cg.death_timer.as_mut() else {
      continue;
    };

    death_timer.tick(time.delta());

    if death_timer.just_finished() {
      match &c.death_behavior {
        DeathBehavior::Respawn(_, invuln_timer) => {
          cg.current_hp = c.max_hp;
          cg.invulnerability_timer = invuln_timer.clone().into();
          cg.stun_timer = None;
          cg.reeling_timer = None;
          cg.death_timer = None;
        }
        DeathBehavior::Despawn(_) => {
          cmd.entity(e).despawn();
        }
      }
    }
  }
}

fn update_kill_counters(
  mut kill_reader: MessageReader<CombatantKilled>,
  mut qry_player: Query<&mut KillCounter>,
) {
  for msg in kill_reader.read() {
    let Some(mut killer) = qry_player.get_mut(msg.killer).ok() else {
      continue;
    };

    killer.kills += 1;
  }
}
