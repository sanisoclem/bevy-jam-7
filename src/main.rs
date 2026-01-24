use bevy::prelude::*;
use game::GamePlugin;

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

  app.run();
}
