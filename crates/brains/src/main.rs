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

      organism.forward(input);

      if timestep == 50 {
        organism.apply_reward(2, 10.0);
      }

      if timestep % 10 == 0 {
        let metrics = organism.metrics();

        println!("t={},{:?}", timestep, metrics);
      }
    }

    organism.end_episode();
  }
}
