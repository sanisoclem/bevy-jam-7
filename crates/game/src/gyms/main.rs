use bevy::{prelude::*, sprite::Anchor, time::Stopwatch};
use jam7::{
  level::{
    Level, LevelCommand, LevelResourcesLoaded, asset::LevelAsset, render::ChunkMeshGenerator,
  },
  player::{
    Player, PlayerAnimationState, create_player_animations, create_player_controls,
    ui::{DespawnStatsUI, SpawnStatsUI},
  },
};
use sys_audio::{GameAudioChannels, GameAudioCommand, GameAudioLibrary};
use sys_cam::CameraTarget;
use sys_candy::Shadow;
use sys_chonker::ChunkGenerator;
use sys_combat::{Combatant, DeathBehavior, HitTestableShape, KillCounter};
use sys_enemy::{EnemySpawner, EnemySpawnerState};
use sys_magic::{SpellBook, SpellBookState};
use sys_move::{IsoMovementStage, IsoWorldCoords, Moveable, Placeable};
use sys_procgen::ProceduralLevel;
use sys_prog::{
  Progger,
  death::ShowDeathUi,
  levelup::LevelUp,
  spells::ui::{DespawnSpellBarUI, SpawnSpellBarUI},
};
use utils::diff::TEAM_PLAYER;

pub struct MainGymPlugin;

impl Plugin for MainGymPlugin {
  fn build(&self, app: &mut App) {
    app
      .init_state::<GameState>()
      .add_systems(OnEnter(GameState::Playing), setup)
      .add_plugins((splash::splash_plugin, menu::menu_plugin))
      .add_observer(on_level_loaded)
      .add_observer(on_death_ui);
  }
}

const TEXT_COLOR: Color = Color::srgb(0.9, 0.9, 0.9);
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Default, States)]
pub enum GameState {
  #[default]
  Splash,
  Menu,
  Playing,
  Done,
}

mod splash {
  use bevy::prelude::*;
  use sys_asset::{AssetLoaded, LoadAssets};
  use sys_audio::{AudioLibraryResource, GameAudioChannels, GameAudioLibrary};
  use sys_combat::CombatAssets;

  use super::GameState;

  pub fn splash_plugin(app: &mut App) {
    app
      .add_systems(OnEnter(GameState::Splash), splash_setup)
      .add_systems(Update, countdown.run_if(in_state(GameState::Splash)))
      .add_observer(on_combat_loaded)
      .add_observer(on_audio_loaded);
  }

  #[derive(Component)]
  struct OnSplashScreen {
    audio_loaded: bool,
    combat_loaded: bool,
    timer: Timer,
  }

  fn splash_setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    let icon = asset_server.load("branding/icon.png");

    commands.trigger(LoadAssets::<
      AudioLibraryResource<GameAudioLibrary, GameAudioChannels>,
    >::default());
    commands.trigger(LoadAssets::<CombatAssets>::default());
    commands.spawn((
      DespawnOnExit(GameState::Splash),
      Node {
        align_items: AlignItems::Center,
        justify_content: JustifyContent::Center,
        width: percent(100),
        height: percent(100),
        ..default()
      },
      OnSplashScreen {
        timer: Timer::from_seconds(5.0, TimerMode::Once),
        combat_loaded: false,
        audio_loaded: false,
      },
      children![(
        ImageNode::new(icon),
        Node {
          width: px(200),
          ..default()
        },
      )],
    ));
  }

  fn countdown(
    qry: Query<&mut OnSplashScreen>,
    time: Res<Time>,
    mut game_state: ResMut<NextState<GameState>>,
  ) {
    for mut splash in qry {
      if !splash.timer.is_finished() {
        splash.timer.tick(time.delta());
      }

      if splash.timer.is_finished() && splash.audio_loaded && splash.combat_loaded {
        game_state.set(GameState::Menu);
      }
    }
  }
  fn on_combat_loaded(_: On<AssetLoaded<CombatAssets>>, qry: Query<&mut OnSplashScreen>) {
    for mut splash in qry {
      splash.combat_loaded = true;
    }
  }
  fn on_audio_loaded(
    _: On<AssetLoaded<AudioLibraryResource<GameAudioLibrary, GameAudioChannels>>>,
    qry: Query<&mut OnSplashScreen>,
  ) {
    for mut splash in qry {
      splash.audio_loaded = true;
    }
  }
}

mod menu {
  use bevy::{app::AppExit, color::palettes::css::WHITE, prelude::*};

  use super::{GameState, TEXT_COLOR};

  pub fn menu_plugin(app: &mut App) {
    app
      .add_systems(OnEnter(GameState::Menu), main_menu_setup)
      .add_systems(
        Update,
        (menu_action, button_system).run_if(in_state(GameState::Menu)),
      );
  }

  #[derive(Component)]
  struct OnMainMenuScreen;

  const NORMAL_BUTTON: Color = Color::srgb(0.15, 0.15, 0.15);
  const HOVERED_BUTTON: Color = Color::srgb(0.25, 0.25, 0.25);
  const HOVERED_PRESSED_BUTTON: Color = Color::srgb(0.25, 0.65, 0.25);
  const PRESSED_BUTTON: Color = Color::srgb(0.35, 0.75, 0.35);

  #[derive(Component)]
  struct SelectedOption;

  #[derive(Component)]
  enum MenuButtonAction {
    Play,
    Quit,
  }

  fn button_system(
    mut interaction_query: Query<
      (&Interaction, &mut BackgroundColor, Option<&SelectedOption>),
      (Changed<Interaction>, With<Button>),
    >,
  ) {
    for (interaction, mut background_color, selected) in &mut interaction_query {
      *background_color = match (*interaction, selected) {
        (Interaction::Pressed, _) | (Interaction::None, Some(_)) => PRESSED_BUTTON.into(),
        (Interaction::Hovered, Some(_)) => HOVERED_PRESSED_BUTTON.into(),
        (Interaction::Hovered, None) => HOVERED_BUTTON.into(),
        (Interaction::None, None) => NORMAL_BUTTON.into(),
      }
    }
  }

  fn main_menu_setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    // Common style for all buttons on the screen
    let button_node = Node {
      width: px(300),
      height: px(65),
      margin: UiRect::all(px(20)),
      justify_content: JustifyContent::Center,
      align_items: AlignItems::Center,
      ..default()
    };
    let button_text_font = TextFont {
      font_size: 33.0,
      ..default()
    };

    let texture_handle = asset_server.load("ui/bg.png");
    commands.spawn((
      DespawnOnExit(GameState::Menu),
      Node {
        width: percent(100),
        height: percent(100),
        align_items: AlignItems::Center,
        justify_content: JustifyContent::Center,
        ..default()
      },
      BackgroundColor(WHITE.into()),
      ImageNode::from_atlas_image(texture_handle.clone(), TextureAtlas::default()),
      OnMainMenuScreen,
      children![(
        Node {
          flex_direction: FlexDirection::Column,
          align_items: AlignItems::Center,
          margin: UiRect::top(Val::Px(200.)),
          ..default()
        },
        children![
          (
            Text::new(""),
            TextFont {
              font_size: 90.0,
              ..default()
            },
            TextColor(TEXT_COLOR),
            Node {
              margin: UiRect::all(px(50)),
              ..default()
            },
          ),
          (
            Button,
            button_node.clone(),
            BackgroundColor(NORMAL_BUTTON),
            MenuButtonAction::Play,
            children![(
              Text::new("Play"),
              button_text_font.clone(),
              TextColor(TEXT_COLOR),
            ),]
          ),
          (
            Button,
            button_node,
            BackgroundColor(NORMAL_BUTTON),
            MenuButtonAction::Quit,
            children![(Text::new("Quit"), button_text_font, TextColor(TEXT_COLOR),),]
          ),
        ]
      )],
    ));
  }

  fn menu_action(
    interaction_query: Query<
      (&Interaction, &MenuButtonAction),
      (Changed<Interaction>, With<Button>),
    >,
    mut app_exit_writer: MessageWriter<AppExit>,
    mut game_state: ResMut<NextState<GameState>>,
  ) {
    for (interaction, menu_button_action) in &interaction_query {
      if *interaction == Interaction::Pressed {
        match menu_button_action {
          MenuButtonAction::Quit => {
            app_exit_writer.write(AppExit::Success);
          }
          MenuButtonAction::Play => {
            game_state.set(GameState::Playing);
          }
        }
      }
    }
  }
}

pub fn setup(mut level_cmd: MessageWriter<LevelCommand>) {
  level_cmd.write(LevelCommand::LoadLevel("alpha".to_owned()));
}

fn on_level_loaded(
  evt: On<LevelResourcesLoaded>,
  mut cmd: Commands,
  qry: Query<&Level>,
  mut layouts: ResMut<Assets<TextureAtlasLayout>>,
  asset_server: Res<AssetServer>,
  levels: Res<Assets<LevelAsset>>,
) {
  let Some(level) = qry.get(evt.0).ok() else {
    return;
  };
  let level_descriptor = levels
    .get(&level.descriptor)
    .expect("level asset should be loaded");

  let tile_size_world = UVec2::new(
    level_descriptor.tileset.tile_width_world,
    level_descriptor.tileset.tile_height_world,
  );
  let chunk_size_world = (level_descriptor.tiles_per_chunk * tile_size_world).as_vec2();
  let spawned_level = cmd
    .spawn((
      Transform::default(),
      Visibility::default(),
      ProceduralLevel::from(level_descriptor),
      IsoMovementStage::from(level_descriptor),
    ))
    .id();

  let player = cmd
    .spawn(create_player(&asset_server, &mut layouts, spawned_level))
    .with_children(|x| {
      x.spawn((
        Shadow { radius: 150. },
        Transform::default().with_translation(-Vec3::Z),
        Visibility::default(),
      ));
    })
    .id();
  cmd.spawn((
    ChunkGenerator::from_player(player, chunk_size_world),
    ChunkMeshGenerator::from(level_descriptor),
    Transform::default(),
    Visibility::default(),
    ChildOf(spawned_level),
  ));
  cmd
    .entity(evt.0)
    .despawn_children()
    .replace_children(&[spawned_level]);

  cmd.trigger(SpawnStatsUI);
  cmd.trigger(SpawnSpellBarUI {
    player_entity: player,
  });
  cmd.trigger(LevelUp { target: player });
  cmd.trigger(GameAudioCommand::ReplaceAllAndFadeInto(
    GameAudioLibrary::Lofi,
    GameAudioChannels::Music,
  ));
}

fn on_death_ui(_trigger: On<ShowDeathUi>, mut cmd: Commands) {
  cmd.trigger(DespawnStatsUI);
  cmd.trigger(DespawnSpellBarUI);
}

pub fn create_player(
  asset_server: &AssetServer,
  layouts: &mut Assets<TextureAtlasLayout>,
  spawn_parent: Entity,
) -> impl Bundle {
  (
    (
      Player,
      ChildOf(spawn_parent),
      KillCounter { kills: 0 },
      CameraTarget,
      Transform::default().with_scale(Vec3::splat(0.3)),
      Visibility::default(),
      Moveable::default(),
      Placeable::mid(IsoWorldCoords::default()),
      Progger {
        hp_gain: 100,
        level: 0,
        bosses_spawned: 0,
        bosses_killed: 0,
      },
      SpatialListener::new(400.),
    ),
    (SpellBook::default(), SpellBookState::default()),
    (
      EnemySpawner {
        spawn_parent,
        despawn_radius: 2500,
        no_spawn_radius: 1200,
        spawn_radius: 1000,
        initial_cooldown: 1.,
        cooldown_decay_rate: 1.5,
        disabled: false,
      },
      EnemySpawnerState {
        stopwatch: Stopwatch::new(),
        cooldown: Timer::from_seconds(0.5, TimerMode::Once),
      },
    ),
    (Combatant {
      max_hp: 1,
      hitbox: HitTestableShape::Circle { radius: 21.0 },
      team: TEAM_PLAYER,
      regen: 0,
      regen_delay: 0,
      death_behavior: DeathBehavior::Despawn(Timer::from_seconds(5.0, TimerMode::Once)),
    }),
    (
      create_player_animations(asset_server, layouts),
      Anchor(Vec2::new(0., -0.42)),
      PlayerAnimationState::default(),
    ),
    create_player_controls(),
  )
}
