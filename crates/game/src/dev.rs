use bevy::{
  dev_tools::fps_overlay::{FpsOverlayConfig, FpsOverlayPlugin, FrameTimeGraphConfig},
  prelude::*,
  text::FontSmoothing,
};

pub struct DevGamePlugin;

impl Plugin for DevGamePlugin {
  fn build(&self, app: &mut App) {
    app.add_plugins(FpsOverlayPlugin {
      config: FpsOverlayConfig {
        text_config: TextFont {
          // Here we define size of our overlay
          font_size: 32.0,
          // If we want, we can use a custom font
          font: default(),
          // We could also disable font smoothing,
          font_smoothing: FontSmoothing::default(),
          ..default()
        },
        // We can also change color of the overlay
        text_color: Color::srgb(1.0, 0.0, 0.0),
        // We can also set the refresh interval for the FPS counter
        refresh_interval: core::time::Duration::from_millis(100),
        enabled: true,
        frame_time_graph_config: FrameTimeGraphConfig {
          enabled: true,
          // The minimum acceptable fps
          min_fps: 30.0,
          // The target fps
          target_fps: 144.0,
        },
      },
    });
  }
}
