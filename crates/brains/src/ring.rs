use core::f32;
use std::{collections::VecDeque, ops::Add};

pub trait SampleData: Add<Output = Self> + Sized + Clone {}

#[derive(Debug)]
pub struct HierarchicalBuffer<D> {
  // each tier stores samples at different temporal resolutions
  tiers: Vec<RingBufferTier<D>>,
  temporal_weights: Vec<f32>,
}

#[derive(Debug)]
pub struct RingBufferTier<D> {
  buffer: VecDeque<D>,
  capacity: usize,
  total_samples: usize,
}

impl<D> HierarchicalBuffer<D>
where
  D: SampleData + Clone,
{
  // note: UB when stacking_size is 0 or 1
  pub fn new(tier_configs: Vec<usize>, recency_bias: f32) -> Self {
    let tiers = tier_configs
      .clone()
      .into_iter()
      .map(|capacity| RingBufferTier::new(capacity))
      .collect();
    // k + k/s + k/s^2  ... + k/s^n
    let k =
      (recency_bias - 1.0) / (recency_bias - (1. / recency_bias.powi(tier_configs.len() as i32)));
    let temporal_weights = (0..tier_configs.len())
      .map(|i| k / recency_bias.powi(i as i32))
      .collect();
    Self {
      tiers,
      temporal_weights,
    }
  }

  pub fn push(&mut self, data: D) {
    let tier0 = self
      .tiers
      .first_mut()
      .expect("there should always be a tier 0");
    tier0.total_samples += 1;
    tier0.push(data);
    self.cascade_downsample();
  }

  fn cascade_downsample(&mut self) {
    for tier_idx in 0..self.tiers.len() - 1 {
      let Some(summed) = ({
        let tier = self.tiers.get(tier_idx).expect("tier should exist!");

        if tier.should_downsample() {
          let summed = tier
            .sum_all()
            .expect("should have items to sum when downsampling");
          Some(summed)
        } else {
          None
        }
      }) else {
        continue;
      };
      let Some(next_tier) = self.tiers.get_mut(tier_idx + 1) else {
        // skip downsampling if no next tier
        continue;
      };

      next_tier.push(summed);
    }
  }

  pub fn get_tier(&self, tier: usize) -> Option<(f32, &RingBufferTier<D>)> {
    let mult = self.temporal_weights.get(tier).copied();
    let data = self.tiers.get(tier);
    mult.zip(data)
  }
  pub fn len(&self) -> usize {
    self.tiers.len()
  }
}

impl<D> RingBufferTier<D>
where
  D: SampleData + Clone,
{
  fn new(capacity: usize) -> Self {
    Self {
      buffer: VecDeque::with_capacity(capacity),
      capacity,
      total_samples: 0,
    }
  }

  fn push(&mut self, data: D) {
    self.buffer.push_front(data);
    self.total_samples += 1;
    if self.buffer.len() > self.capacity {
      self.buffer.pop_back();
    }
  }
  pub fn should_downsample(&self) -> bool {
    self.total_samples.is_multiple_of(self.capacity)
  }

  pub fn sum_all(&self) -> Option<D> {
    // Sum all tensors in the buffer
    self.buffer.iter().cloned().reduce(|acc, t| acc + t)
  }
}
