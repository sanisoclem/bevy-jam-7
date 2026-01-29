#![recursion_limit = "256"]

use brains::PredictiveOrganism;
use burn::prelude::*;

fn main() {
  type Backend = burn::backend::NdArray;

  let device = Default::default();
  let mut organism = PredictiveOrganism::<Backend>::new(&device);

  for episode in 0..10 {
    println!("Episode {}", episode);

    for timestep in 0..100 {
      let input =
        Tensor::<Backend, 1>::random([32], burn::tensor::Distribution::Uniform(0.0, 1.0), &device);

      let (state_acts, action_acts, pred_acts) = organism.forward(input);

      if timestep == 50 {
        organism.enter_reward_state(1.0);
      }

      if timestep % 10 == 0 {
        let state_count: f32 = state_acts.clone().sum().into_scalar();
        let action_count: f32 = action_acts.clone().sum().into_scalar();
        let pred_count: f32 = pred_acts.clone().sum().into_scalar();

        println!(
          "  t={}: state={}, actions={}, predictive={}",
          timestep, state_count, action_count, pred_count
        );
      }
    }

    organism.end_episode();
  }
}
