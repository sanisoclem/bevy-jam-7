use bevy::prelude::*;
use game::GamePlugin;

#[cfg(feature = "dev")]
use bevy_egui::EguiPlugin;
#[cfg(feature = "dev")]
use bevy_inspector_egui::quick::WorldInspectorPlugin;

// #[cfg(feature = "debug")]
// use bevy_egui::EguiPlugin;
// #[cfg(feature = "debug")]
// use bevy_inspector_egui::quick::WorldInspectorPlugin;

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn main_wasm() {
  main();
}

fn main() {
  let mut app = App::new();

  app.add_plugins(GamePlugin);

  #[cfg(feature = "dev")]
  app.add_plugins((EguiPlugin::default(), WorldInspectorPlugin::default()));

  app.run();
}
