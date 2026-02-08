use bevy::{
  color::palettes::css::{GREEN, RED},
  prelude::*,
};
pub use hittest::HitTestableShape;
use sys_move::{IsoMovementStage, IsoWorldCoords, Placeable};

mod hittest;

pub struct SysCombatPlugin;

impl Plugin for SysCombatPlugin {
  fn build(&self, app: &mut App) {
    app
      .add_message::<ApplyCombatEffect>()
      .add_message::<DamageTaken>()
      .add_systems(
        FixedUpdate,
        (
          create_combat_guages,
          test_hitboxes,
          apply_combat_effects,
          sync_combat_state,
          despawn_dead,
        )
          .chain(),
      );

    #[cfg(feature = "dev")]
    app.add_systems(Update, draw_gizmos);
  }
}

#[derive(Reflect, Debug, Clone)]
pub struct CombatantId(u32);

#[derive(Component, Reflect, Debug, Clone)]
pub struct Combatant {
  pub team: u8,
  pub max_hp: u32,
  pub regen: u32,
  pub regen_delay: u32,
  pub hitbox: HitTestableShape,
  pub despawn_delay_seconds: u32,
}

#[derive(Component, Reflect, Debug, Clone)]
pub struct CombatantGuages {
  pub reeling_timer: Option<Timer>,
  pub invulnerability_timer: Option<Timer>,
  pub stun_timer: Option<Timer>,
  pub despawn_timer: Option<Timer>,
  pub current_hp: u32,
}

#[derive(Component, Reflect, Debug, Clone)]
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
  pub effects: Vec<CombatEffectBlueprint>,
}

#[derive(Reflect, Debug, Clone)]
pub enum CombatEffectBlueprint {
  Damage(u32),
  // more damage the farther it goes
  SnipeDamage(u32, f32, IsoWorldCoords),
  // Knockback(f32, f32), // magnitude and falloff
  // Gravity(f32, f32),
  // Stun(f32),
  // Infection { delay_seconds: f32, lifetime: u8 },
}

#[derive(Reflect, Debug, Clone)]
pub enum CombatEffect {
  Damage(u32),
}

#[derive(Reflect, Debug, Clone, Message)]
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
        despawn_timer: None,
      },
      CombatantState {
        reeling: false,
        stunned: false,
        invulnerable: false,
        dead: false,
      },
    ));
  }
}
fn test_hitboxes(
  qry_hitboxes: Query<(Entity, &Combatant, &CombatantGuages, &Placeable)>,
  qry_effects: Query<(&CombatAreaEffect, &Placeable)>,
  mut msg_writer: MessageWriter<ApplyCombatEffect>,
) {
  for (hb_entity, hb, hbg, hb_pos) in qry_hitboxes {
    if hbg.current_hp == 0 {
      continue;
    }

    for (eb, eb_pos) in qry_effects {
      if !hb
        .hitbox
        .hit_test(&hb_pos.location, &eb.shape, &eb_pos.location)
      {
        continue;
      }
      if hb.team == eb.team {
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
        })
        .collect();
      msg_writer.write(ApplyCombatEffect {
        effects,
        target: hb_entity,
        source: eb.owner,
      });
    }
  }
}

fn apply_combat_effects(
  mut qry: Query<(&Combatant, &mut CombatantGuages, &CombatantState)>,
  mut msg_reader: MessageReader<ApplyCombatEffect>,
  mut msg_writer: MessageWriter<DamageTaken>,
) {
  for msg in msg_reader.read() {
    let Ok((c, mut g, s)) = qry.get_mut(msg.target) else {
      continue;
    };
    if s.dead || s.invulnerable {
      continue;
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
            g.despawn_timer = Some(Timer::from_seconds(
              c.despawn_delay_seconds as f32,
              TimerMode::Once,
            ));
          }
        }
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

fn despawn_dead(
  mut cmd: Commands,
  mut qry: Query<(Entity, &mut CombatantGuages)>,
  time: Res<Time>,
) {
  for (e, mut c) in qry {
    let Some(despawn_timer) = c.despawn_timer.as_mut() else {
      continue;
    };

    despawn_timer.tick(time.delta());
    if despawn_timer.just_finished() {
      cmd.entity(e).despawn();
    }
  }
}

fn draw_gizmos(
  mut gizmos: Gizmos,
  qry: Query<(&Combatant, &CombatantGuages, &CombatantState, &Placeable)>,
  qry_aoe: Query<(&CombatAreaEffect, &Placeable)>,
  qry_stage: Query<&IsoMovementStage>,
) {
  let Some(stage) = qry_stage.iter().next() else {
    return;
  };

  for (c, cg, cs, p) in qry {
    c.hitbox.draw_gizmo(
      &mut gizmos,
      p.location,
      stage.aspect_ratio,
      Color::from(GREEN),
    );
  }

  for (c, p) in qry_aoe {
    let Some(stage) = qry_stage.iter().next() else {
      continue;
    };
    c.shape.draw_gizmo(
      &mut gizmos,
      p.location,
      stage.aspect_ratio,
      Color::from(RED),
    );
  }
}
