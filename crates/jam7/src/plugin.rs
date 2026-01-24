use bevy::prelude::*;
pub struct Jam7Plugin;

impl Plugin for Jam7Plugin {
  fn build(&self, app: &mut App) {
    #[cfg(feature = "debug")]
    app.add_plugins((
            // FpsOverlayPlugin
      // EguiPlugin,
      // utils::fps::ScreenDiagsTextPlugin,
      // WorldInspectorPlugin::default(),
    ));
  }
}
