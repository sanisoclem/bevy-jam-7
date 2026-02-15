pub mod level;
pub mod player;
pub mod plugin;
pub mod ui;

pub mod prelude {
  pub use crate::plugin::Jam7Plugin;
}

#[cfg(feature = "dev")]
mod debug;
