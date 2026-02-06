pub mod level;
pub mod player;
pub mod plugin;
pub mod ui;
pub mod prelude {
  pub use crate::{
    level::chunk::{ChunkSpawner, LevelChunk},
    plugin::Jam7Plugin,
  };
}
