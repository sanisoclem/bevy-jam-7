use bevy::prelude::*;
use std::{
  ops::Drop,
  sync::{
    Arc,
    atomic::{AtomicU32, Ordering},
  },
};
use thiserror::Error;

#[derive(Debug, Resource, Deref)]
pub struct AssetBarrier(Arc<AssetBarrierInner>);

#[derive(Debug, Deref)]
pub struct AssetBarrierGuard(Arc<AssetBarrierInner>);

#[derive(Debug, Resource)]
pub struct AssetBarrierInner {
  count: AtomicU32,
}
impl AssetBarrier {
  pub fn new() -> (AssetBarrier, AssetBarrierGuard) {
    let inner = Arc::new(AssetBarrierInner {
      count: AtomicU32::new(1),
    });
    (AssetBarrier(inner.clone()), AssetBarrierGuard(inner))
  }
  pub fn is_ready(&self) -> bool {
    self.count.load(Ordering::Acquire) == 0
  }
}

impl Clone for AssetBarrierGuard {
  fn clone(&self) -> Self {
    self.count.fetch_add(1, Ordering::AcqRel);
    AssetBarrierGuard(self.0.clone())
  }
}

impl Drop for AssetBarrierGuard {
  fn drop(&mut self) {
    self.count.fetch_sub(1, Ordering::AcqRel);
  }
}

#[non_exhaustive]
#[derive(Debug, Error)]
pub enum CustomRonAssetLoaderError {
  #[error("Could not load asset: {0}")]
  Io(#[from] std::io::Error),
  #[error("Could not parse RON: {0}")]
  RonSpannedError(#[from] ron::error::SpannedError),
}
