use burn::prelude::*;
use burn::tensor::backend::Backend;

#[derive(Clone, Debug)]
struct FiringEvent {
  neuron_idx: usize,
  timestep: usize,
  neuron_type: NeuronType,
}

#[derive(Clone, Debug, PartialEq)]
enum NeuronType {
  State,
  Action,
  Predictive,
}

#[derive(Debug)]
struct PredictiveOrganism<B: Backend> {
  state_weights_base: Tensor<B, 2>,
  state_weights_overlay: Tensor<B, 2>,
  state_bias: Tensor<B, 1>,
  state_threshold: f32,

  action_weights_base: Tensor<B, 2>,
  action_weights_overlay: Tensor<B, 2>,
  action_bias: Tensor<B, 1>,
  action_threshold: f32,

  predictive_weights_base: Tensor<B, 2>,
  predictive_weights_overlay: Tensor<B, 2>,
  predictive_bias: Tensor<B, 1>,
  predictive_threshold: f32,

  prediction_matrix_base: Tensor<B, 2>,
  prediction_matrix_overlay: Tensor<B, 2>,

  predictive_weight_change_magnitude: Vec<f32>,

  firing_history: Vec<FiringEvent>,
  current_timestep: usize,

  reward_magnitude: f32,
  reward_duration: usize,
  punishment_magnitude: f32,
  punishment_duration: usize,

  prediction_learning_rate: f32,
  action_learning_rate: f32,
  novelty_threshold: f32,
  temporal_window: usize,
  stable_link_threshold: f32,
  base_decay_rate: f32,
}

impl<B: Backend> PredictiveOrganism<B> {
  pub fn new(device: &B::Device) -> Self {
    let state_weights_base = Tensor::random(
      [32, 64],
      burn::tensor::Distribution::Uniform(-1.0, 1.0),
      device,
    );
    let state_weights_overlay = Tensor::zeros([32, 64], device);
    let state_bias = Tensor::zeros([64], device);

    let action_weights_base = Tensor::random(
      [64, 12],
      burn::tensor::Distribution::Uniform(-1.0, 1.0),
      device,
    );
    let action_weights_overlay = Tensor::zeros([64, 12], device);
    let action_bias = Tensor::zeros([12], device);

    let predictive_weights_base = Tensor::random(
      [76, 128],
      burn::tensor::Distribution::Uniform(-1.0, 1.0),
      device,
    );
    let predictive_weights_overlay = Tensor::zeros([76, 128], device);
    let predictive_bias = Tensor::zeros([128], device);

    let prediction_matrix_base = Tensor::zeros([128, 64], device);
    let prediction_matrix_overlay = Tensor::zeros([128, 64], device);

    Self {
      state_weights_base,
      state_weights_overlay,
      state_bias,
      state_threshold: 0.5,

      action_weights_base,
      action_weights_overlay,
      action_bias,
      action_threshold: 0.5,

      predictive_weights_base,
      predictive_weights_overlay,
      predictive_bias,
      predictive_threshold: 0.5,

      prediction_matrix_base,
      prediction_matrix_overlay,
      predictive_weight_change_magnitude: vec![0.0; 128],

      firing_history: Vec::new(),
      current_timestep: 0,

      reward_magnitude: 0.0,
      reward_duration: 0,
      punishment_magnitude: 0.0,
      punishment_duration: 0,

      prediction_learning_rate: 0.01,
      action_learning_rate: 0.01,
      novelty_threshold: 5.0,
      temporal_window: 10,
      stable_link_threshold: 0.3,
      base_decay_rate: 0.001,
    }
  }

  pub fn forward(&mut self, input: Tensor<B, 1>) -> (Tensor<B, 1>, Tensor<B, 1>, Tensor<B, 1>) {
    let device = input.device();

    // overlay + base
    let state_weights = self.state_weights_base.clone() + self.state_weights_overlay.clone();

    // get result for state layer
    let state_logits = input
      .clone()
      .unsqueeze_dim(0)
      .matmul(state_weights)
      .squeeze(0)
      + self.state_bias.clone();

    let state_activations = self.apply_threshold(state_logits.clone(), self.state_threshold);

    // record the neurons that fired in case of ambient state
    self.record_firings(&state_activations, NeuronType::State);

    // get actions to execute based on state
    let action_weights = self.action_weights_base.clone() + self.action_weights_overlay.clone();
    let action_logits = state_activations
      .clone()
      .unsqueeze_dim(0)
      .matmul(action_weights)
      .squeeze(0)
      + self.action_bias.clone();
    let action_activations = self.apply_threshold(action_logits, self.action_threshold);

    // record actions in state of ambient state
    self.record_firings(&action_activations, NeuronType::Action);

    let predictive_activations = if self.current_timestep > 0 {
      let prev_state = self.get_previous_activations(NeuronType::State, 1);
      let prev_actions = self.get_previous_activations(NeuronType::Action, 1);

      let predictive_input = Tensor::cat(vec![prev_state, prev_actions], 0);

      let predictive_weights =
        self.predictive_weights_base.clone() + self.predictive_weights_overlay.clone();
      let predictive_logits = predictive_input
        .unsqueeze_dim(0)
        .matmul(predictive_weights)
        .squeeze(0)
        + self.predictive_bias.clone();

      let activations = self.apply_threshold(predictive_logits, self.predictive_threshold);

      // our predictions
      self.record_firings(&activations, NeuronType::Predictive);

      self.update_prediction_weights(&activations, &state_activations);

      self.check_novelty(&activations);

      activations
    } else {
      Tensor::zeros([128], &device)
    };

    self.decay_reward_punishment();

    self.modulate_action_weights();

    self.current_timestep += 1;

    (
      state_activations,
      action_activations,
      predictive_activations,
    )
  }

  fn apply_threshold(&self, tensor: Tensor<B, 1>, threshold: f32) -> Tensor<B, 1> {
    let mask = tensor.greater_elem(threshold);
    mask.float()
  }

  fn record_firings(&mut self, activations: &Tensor<B, 1>, neuron_type: NeuronType) {
    let acts: Vec<f32> = activations.clone().into_data().convert().value;

    for (idx, &value) in acts.iter().enumerate() {
      if value > 0.5 {
        self.firing_history.push(FiringEvent {
          neuron_idx: idx,
          timestep: self.current_timestep,
          neuron_type: neuron_type.clone(),
        });
      }
    }
  }

  fn get_previous_activations(&self, neuron_type: NeuronType, steps_back: usize) -> Tensor<B, 1> {
    let target_timestep = self.current_timestep.saturating_sub(steps_back);

    let size = match neuron_type {
      NeuronType::State => 64,
      NeuronType::Action => 12,
      NeuronType::Predictive => 128,
    };

    let device = self.state_weights_base.device();
    let mut activations = vec![0.0; size];

    for event in &self.firing_history {
      if event.timestep == target_timestep && event.neuron_type == neuron_type {
        activations[event.neuron_idx] = 1.0;
      }
    }

    Tensor::from_floats(activations.as_slice(), &device)
  }

  fn update_prediction_weights(
    &mut self,
    predictive_activations: &Tensor<B, 1>,
    state_activations: &Tensor<B, 1>,
  ) {
    let pred_acts: Vec<f32> = predictive_activations.clone().into_data().convert().value;
    let state_acts: Vec<f32> = state_activations.clone().into_data().convert().value;

    let device = self.prediction_matrix_base.device();
    let pred_matrix_base: Vec<f32> = self
      .prediction_matrix_base
      .clone()
      .into_data()
      .convert()
      .value;
    let pred_matrix_overlay: Vec<f32> = self
      .prediction_matrix_overlay
      .clone()
      .into_data()
      .convert()
      .value;

    let mut new_overlay = pred_matrix_overlay.clone();

    // check predictions for all state neurons
    // maybe we should limit this spatially, but since the neuron count is low,
    // its prob fine like this for now
    for p in 0..128 {
      if pred_acts[p] > 0.5 {
        for s in 0..64 {
          // TODO: no need to evaluate overly negative predictive wegihts
          let idx = p * 64 + s;
          let base_val = pred_matrix_base[idx];
          let overlay_val = pred_matrix_overlay[idx];
          let total_val = base_val + overlay_val;

          let delta = if state_acts[s] > 0.5 {
            self.prediction_learning_rate
          } else {
            -2.0 * self.prediction_learning_rate
          };

          new_overlay[idx] = overlay_val + delta;

          if total_val + delta < 0.0 {
            let weight_change = self.predictive_weight_change_magnitude[p];
            let decay_rate = self.base_decay_rate * (1.0 + weight_change);
            new_overlay[idx] += decay_rate;
          }
        }
      }
    }

    self.prediction_matrix_overlay =
      Tensor::from_floats(new_overlay.as_slice(), &device).reshape([128, 64]);
  }

  fn check_novelty(&mut self, predictive_activations: &Tensor<B, 1>) {
    let pred_acts: Vec<f32> = predictive_activations.clone().into_data().convert().value;
    let pred_matrix_base: Vec<f32> = self
      .prediction_matrix_base
      .clone()
      .into_data()
      .convert()
      .value;
    let pred_matrix_overlay: Vec<f32> = self
      .prediction_matrix_overlay
      .clone()
      .into_data()
      .convert()
      .value;

    let mut new_stable_links = 0;

    for p in 0..128 {
      if pred_acts[p] > 0.5 {
        for s in 0..64 {
          let idx = p * 64 + s;
          let base_val = pred_matrix_base[idx];
          let overlay_val = pred_matrix_overlay[idx];

          if overlay_val > self.stable_link_threshold && base_val <= 0.0 {
            new_stable_links += 1;
          }
        }
      }
    }

    if new_stable_links as f32 > self.novelty_threshold {
      let magnitude = (new_stable_links as f32 / self.novelty_threshold) * 0.5;
      // TODO: variable reward state duration based on how many stable links have formed
      // how do we know which formed recently (not fired recently or does it matter?)
      self.enter_reward_state(magnitude, 5);
    }
  }

  pub fn enter_reward_state(&mut self, magnitude: f32, duration: usize) {
    self.reward_magnitude += magnitude;
    self.reward_duration = self.reward_duration.max(duration);
  }

  pub fn enter_punishment_state(&mut self, magnitude: f32, duration: usize) {
    self.punishment_magnitude += magnitude;
    self.punishment_duration = self.punishment_duration.max(duration);
  }

  fn decay_reward_punishment(&mut self) {
    if self.reward_duration > 0 {
      self.reward_duration -= 1;
      if self.reward_duration == 0 {
        self.reward_magnitude = 0.0;
      }
    }

    if self.punishment_duration > 0 {
      self.punishment_duration -= 1;
      if self.punishment_duration == 0 {
        self.punishment_magnitude = 0.0;
      }
    }
  }

  fn modulate_action_weights(&mut self) {
    if self.reward_magnitude == 0.0 && self.punishment_magnitude == 0.0 {
      return;
    }

    let device = self.action_weights_base.device();
    let mut overlay: Vec<f32> = self
      .action_weights_overlay
      .clone()
      .into_data()
      .convert()
      .value;

    for event in &self.firing_history {
      if event.neuron_type != NeuronType::Action {
        continue;
      }

      let time_distance = self.current_timestep.saturating_sub(event.timestep);
      if time_distance > self.temporal_window {
        continue;
      }

      let action_idx = event.neuron_idx;

      let state_activations = self.get_activations_at_timestep(NeuronType::State, event.timestep);

      let temporal_factor = 1.0 / (1.0 + time_distance as f32);

      for (state_idx, &was_active) in state_activations.iter().enumerate() {
        if was_active > 0.5 {
          let weight_idx = state_idx * 12 + action_idx;

          let reward_effect = self.reward_magnitude * temporal_factor * self.action_learning_rate;
          let punishment_effect =
            self.punishment_magnitude * temporal_factor * self.action_learning_rate;

          overlay[weight_idx] += reward_effect - punishment_effect;
        }
      }
    }

    self.action_weights_overlay =
      Tensor::from_floats(overlay.as_slice(), &device).reshape([64, 12]);
  }

  fn get_activations_at_timestep(&self, neuron_type: NeuronType, timestep: usize) -> Vec<f32> {
    let size = match neuron_type {
      NeuronType::State => 64,
      NeuronType::Action => 12,
      NeuronType::Predictive => 128,
    };

    let mut activations = vec![0.0; size];

    for event in &self.firing_history {
      if event.timestep == timestep && event.neuron_type == neuron_type {
        activations[event.neuron_idx] = 1.0;
      }
    }

    activations
  }

  pub fn end_episode(&mut self) {
    let device = self.state_weights_base.device();

    self.prediction_matrix_base =
      self.prediction_matrix_base.clone() + self.prediction_matrix_overlay.clone();
    self.prediction_matrix_overlay = Tensor::zeros([128, 64], &device);

    let state_utilization = self.calculate_state_utilization();

    self.consolidate_and_randomize_weights(
      &mut self.state_weights_base,
      &mut self.state_weights_overlay,
      &state_utilization,
      [32, 64],
    );

    let predictive_utilization = self.calculate_predictive_utilization();

    self.update_predictive_weight_change_magnitude();

    self.consolidate_and_randomize_weights(
      &mut self.predictive_weights_base,
      &mut self.predictive_weights_overlay,
      &predictive_utilization,
      [76, 128],
    );

    self.action_weights_base =
      self.action_weights_base.clone() + self.action_weights_overlay.clone();
    self.action_weights_overlay = Tensor::zeros([64, 12], &device);

    self.firing_history.clear();
    self.current_timestep = 0;

    self.reward_magnitude = 0.0;
    self.reward_duration = 0;
    self.punishment_magnitude = 0.0;
    self.punishment_duration = 0;
  }

  fn calculate_state_utilization(&self) -> Vec<f32> {
    let pred_matrix: Vec<f32> = (self.prediction_matrix_base.clone()
      + self.prediction_matrix_overlay.clone())
    .into_data()
    .convert()
    .value;

    let predictive_weights: Vec<f32> = (self.predictive_weights_base.clone()
      + self.predictive_weights_overlay.clone())
    .into_data()
    .convert()
    .value;

    let mut utilization = vec![0.0; 64];

    for s in 0..64 {
      let mut total = 0.0;

      for p in 0..128 {
        let pred_weight = pred_matrix[p * 64 + s];

        if pred_weight > 0.0 {
          let state_to_pred_weight = predictive_weights[s * 128 + p];

          total += pred_weight * state_to_pred_weight.abs();
        }
      }

      utilization[s] = total;
    }

    utilization
  }

  fn calculate_predictive_utilization(&self) -> Vec<f32> {
    let pred_matrix: Vec<f32> = (self.prediction_matrix_base.clone()
      + self.prediction_matrix_overlay.clone())
    .into_data()
    .convert()
    .value;

    let mut utilization = vec![0.0; 128];

    for p in 0..128 {
      let mut total = 0.0;

      for s in 0..64 {
        let pred_weight = pred_matrix[p * 64 + s];

        if pred_weight > 0.0 {
          total += pred_weight;
        }
      }

      utilization[p] = total;
    }

    utilization
  }

  fn update_predictive_weight_change_magnitude(&mut self) {
    let overlay: Vec<f32> = self
      .predictive_weights_overlay
      .clone()
      .into_data()
      .convert()
      .value;

    for p in 0..128 {
      let mut total_change = 0.0;

      for i in 0..76 {
        let weight_idx = i * 128 + p;
        total_change += overlay[weight_idx].abs();
      }

      self.predictive_weight_change_magnitude[p] = total_change;
    }
  }

  fn consolidate_and_randomize_weights(
    &mut self,
    base: &mut Tensor<B, 2>,
    overlay: &mut Tensor<B, 2>,
    utilization: &[f32],
    shape: [usize; 2],
  ) {
    let device = base.device();

    *base = base.clone() + overlay.clone();

    let base_data: Vec<f32> = base.clone().into_data().convert().value;
    let mut new_overlay = Vec::new();

    for i in 0..shape[0] {
      for j in 0..shape[1] {
        let idx = i * shape[1] + j;

        let util = utilization[j];
        let base_magnitude = base_data[idx].abs();

        let overlay_magnitude = base_magnitude / (1.0 + util);

        let random_val = (rand::random::<f32>() * 2.0 - 1.0) * overlay_magnitude;
        new_overlay.push(random_val);
      }
    }

    *overlay = Tensor::from_floats(new_overlay.as_slice(), &device).reshape(shape);
  }
}

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
        organism.enter_reward_state(1.0, 5);
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
