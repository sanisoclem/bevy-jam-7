use bevy::prelude::*;
use sys_combat::{Combatant, CombatantRadar, CombatantState, HitTestableShape};
use sys_move::{IsoWorldCoords, Placeable};

mod fireball;

pub struct SysMagicPlugin;

impl Plugin for SysMagicPlugin {
  fn build(&self, app: &mut App) {
    app.add_message::<CastSpell>().add_systems(
      FixedUpdate,
      (update_cooldowns, cast_auto_spells, fireball::cast_fireball),
    );
  }
}

#[derive(Component, Debug, Reflect, Clone)]
pub struct SpellBook {
  pub spells: Vec<EquippedSpell>,
}

#[derive(Component, Debug, Reflect, Clone)]
pub struct SpellBookState {
  pub spells_states: Vec<EquippedSpellState>,
}

#[derive(Debug, Reflect, Clone)]
pub struct EquippedSpell {
  pub generator: SpellGenerator,
  pub cooldown: Timer,
  pub trigger: SpellTrigger,
  // pub item_upgrades: Vec<()>,
}

#[derive(Debug, Reflect, Clone, Default)]
pub struct EquippedSpellState {
  pub cooldown: Option<Timer>,
}

#[derive(Debug, Reflect, Clone)]
pub struct SpellGrowthModifiers {
  pub damage_mult: f32,
}

#[derive(Debug, Reflect, Clone)]
pub enum SpellGenerator {
  Fireball {
    radius: f32,
    base_damage: u32,
    lifetime: f32,
    speed: f32,
    // growth_factor: f32,
    // snipe_damage: f32,
  },
}

#[derive(Debug, Reflect, Clone, Eq, PartialEq)]
pub enum SpellTrigger {
  Auto,
  TakingDamage,
}

#[derive(Message, Clone, Debug, Reflect)]
pub struct CastSpell {
  pub caster: Entity,
  pub spawn_parent: Entity,
  pub team: u8,
  pub spell: SpellInstance,
}

#[derive(Debug, Reflect, Clone)]
pub enum SpellInstance {
  Fireball {
    source: IsoWorldCoords,
    target: Entity,
    direction: Vec2,
    shape: HitTestableShape,
    base_damage: u32,
    speed: f32,
    lifetime: Timer,
  },
}

fn update_cooldowns(qry: Query<&mut SpellBookState>, time: Res<Time>) {
  for mut sbs in qry {
    for spell_state in sbs.spells_states.iter_mut() {
      if let Some(cd) = spell_state.cooldown.as_mut() {
        cd.tick(time.delta());
        if cd.is_finished() {
          spell_state.cooldown = None;
        }
      }
    }
  }
}

fn cast_auto_spells(
  qry: Query<(
    Entity,
    &ChildOf,
    &Combatant,
    &CombatantRadar,
    &CombatantState,
    &SpellBook,
    &Placeable,
    &mut SpellBookState,
  )>,
  mut message_writer: MessageWriter<CastSpell>,
) {
  for (caster, parent, c, radar, cs, spellbook, p, mut sbs) in qry {
    for (spell, spell_state) in spellbook.spells.iter().zip(sbs.spells_states.iter_mut()) {
      if SpellTrigger::Auto != spell.trigger || spell_state.cooldown.is_some() {
        continue;
      }
      if cs.dead || cs.reeling || cs.stunned {
        continue;
      }

      let maybe_instance = match (&spell.generator, radar) {
        (
          SpellGenerator::Fireball {
            radius,
            base_damage,
            lifetime,
            speed,
          },
          CombatantRadar {
            nearest: Some((nearest_entity, nearest_coords)),
            ..
          },
        ) => Some(SpellInstance::Fireball {
          source: p.location,
          direction: *(*nearest_coords - p.location),
          shape: HitTestableShape::Circle { radius: *radius },
          base_damage: *base_damage,
          lifetime: Timer::from_seconds(*lifetime, TimerMode::Once),
          speed: *speed,
          target: *nearest_entity,
        }),
        _ => None,
      };

      let Some(instance) = maybe_instance else {
        continue;
      };

      spell_state.cooldown = Some(spell.cooldown.clone());
      message_writer.write(CastSpell {
        caster,
        spawn_parent: parent.0,
        team: c.team,
        spell: instance,
      });
    }
  }
}
