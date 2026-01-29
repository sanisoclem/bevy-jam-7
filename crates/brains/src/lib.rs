use burn::prelude::*;
use burn::tensor::backend::Backend;
use core::f32;

#[derive(Clone, Debug)]
pub struct FiringEvent {
  neuron_idx: usize,
  timestep: usize,
  neuron_type: NeuronType,
}

#[derive(Clone, Debug, PartialEq)]
pub enum NeuronType {
  State,
  Action,
  Predictive,
}

#[derive(Debug)]
pub struct PredictiveOrganism<B: Backend> {
  // State Neurons
  state_weights_base: Tensor<B, 2>,
  state_weights_overlay: Tensor<B, 2>,
  state_bias: Tensor<B, 1>,
  state_threshold: f32,

  // Action Neurons
  action_weights_base: Tensor<B, 2>,
  action_weights_overlay: Tensor<B, 2>,
  action_bias: Tensor<B, 1>,
  action_threshold: f32,

  // Prediction Neurons
  predictive_weights_base: Tensor<B, 2>,
  predictive_weights_overlay: Tensor<B, 2>,
  predictive_bias: Tensor<B, 1>,
  predictive_threshold: f32,

  // Prediction performance matrix
  prediction_matrix_base: Tensor<B, 2>,
  prediction_matrix_overlay: Tensor<B, 2>,
  predictive_weight_change_magnitude: Tensor<B, 1>,

  // save firings for reward/punishment
  // NOTE: we prob want to use some type of circular buffer
  // to establish an upper bound on space used
  firing_history: Vec<FiringEvent>,

  // are overflows significant?
  current_timestep: usize,

  reward_magnitude: f32,
  punishment_magnitude: f32,
  temporal_scaling_k: f32,

  action_learning_rate: f32,
  novelty_threshold: f32,
  temporal_window: usize,
  stable_link_threshold: f32,

  prediction_learning_rate: f32,
  prediction_max_negative_weight: f32,
  prediction_max_positive_weight: f32,
}

impl<B: Backend> PredictiveOrganism<B>
where
  B: Backend<FloatElem = f32>,
{
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
      predictive_weight_change_magnitude: Tensor::zeros([128], device),

      firing_history: Vec::new(),
      current_timestep: 0,

      reward_magnitude: 0.0,
      punishment_magnitude: 0.0,
      temporal_scaling_k: 1.0,

      prediction_learning_rate: 0.01,
      action_learning_rate: 0.01,
      novelty_threshold: 5.0,
      temporal_window: 10,
      stable_link_threshold: 0.3,
      prediction_max_negative_weight: -100.,
      prediction_max_positive_weight: 1000.,
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
      .squeeze()
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
      .squeeze()
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
        .squeeze()
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
    let acts: Vec<f32> = activations.clone().into_data().to_vec::<f32>().unwrap();

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
    // make into single column matrix (a vector!) and multiply both
    let pred_fired = predictive_activations.clone().unsqueeze_dim(1);
    let state_fired = state_activations.clone().unsqueeze();
    let both_fired = pred_fired.matmul(state_fired);

    // create a matrix of same size but will be one only if P neuron fired
    let pred_fired_2d = predictive_activations
      .clone()
      .unsqueeze_dim(1)
      .repeat(&[1, 64]);

    // calculate base deltas
    let positive_base = both_fired.clone() * self.prediction_learning_rate;
    let pred_fired_state_not = pred_fired_2d.clone() - both_fired;
    let negative_base = pred_fired_state_not * (-2.0 * self.prediction_learning_rate);

    // scale learning rate by weight change magnitude (since episode start)
    let scaling_tensor = (self.predictive_weight_change_magnitude.clone() + 1.0)
      .unsqueeze_dim(1) // [128, 1]
      .repeat(&[1, 64]); // [128, 64]

    let delta = (positive_base + negative_base) * scaling_tensor;

    self.prediction_matrix_overlay = self.prediction_matrix_overlay.clone() + delta;
  }

  fn check_novelty(&mut self, predictive_activations: &Tensor<B, 1>) {
    // create matrix that has val 1 if fired
    let pred_fired_mask = predictive_activations
      .clone()
      .unsqueeze_dim(1)
      .repeat(&[1, 64]);

    // overlay > threshold
    let overlay_high = self
      .prediction_matrix_overlay
      .clone()
      .greater_elem(self.stable_link_threshold)
      .float();

    // base <= 0
    let base_low = self
      .prediction_matrix_base
      .clone()
      .lower_equal_elem(0.0)
      .float();

    // combine all, will be 1.0 for P neurons that fired that have high overlay and low base
    let new_links_mask = pred_fired_mask * overlay_high * base_low;

    let new_stable_links = new_links_mask.sum().into_scalar();

    if new_stable_links > self.novelty_threshold {
      let magnitude = (new_stable_links / self.novelty_threshold) * 0.5;
      self.enter_reward_state(magnitude);
    }
  }

  pub fn enter_reward_state(&mut self, magnitude: f32) {
    self.reward_magnitude += magnitude;
  }
  pub fn enter_punishment_state(&mut self, magnitude: f32) {
    self.punishment_magnitude += magnitude;
  }
  fn decay_reward_punishment(&mut self) {
    self.reward_magnitude *= 0.9; // Exponential decay
    self.punishment_magnitude *= 0.9;
  }
  fn modulate_action_weights(&mut self) {
    if self.reward_magnitude == 0.0 && self.punishment_magnitude == 0.0 {
      return;
    }

    let device = self.action_weights_base.device();

    // static temporal mask, using linear scaling for now
    let temporal_mask: Vec<f32> = (0..self.temporal_window)
      .map(|i| i as f32 / (self.temporal_window - 1) as f32)
      .collect();

    let mut updates = Vec::new();

    for (t, &mask_val) in temporal_mask.iter().enumerate() {
      let steps_back = self.temporal_window - 1 - t; // most recent = highest mask

      let state_fired = self.get_previous_activations(NeuronType::State, steps_back);
      let action_fired = self.get_previous_activations(NeuronType::Action, steps_back);

      let state_2d = state_fired.unsqueeze_dim(1);
      let action_2d = action_fired.unsqueeze();
      let firing_mask = state_2d.matmul(action_2d);

      // scale temporal mask by magnitude
      let net_magnitude = self.reward_magnitude - self.punishment_magnitude;
      let effective_weight = (mask_val * net_magnitude * self.temporal_scaling_k).max(0.0);

      if effective_weight > 0.0 {
        let delta = firing_mask * effective_weight * self.action_learning_rate;
        updates.push(delta);
      }
    }

    let total_update = if updates.is_empty() {
      Tensor::zeros([64, 12], &device)
    } else {
      updates
        .into_iter()
        .fold(Tensor::zeros([64, 12], &device), |acc, update| acc + update)
    };

    self.action_weights_overlay = self.action_weights_overlay.clone() + total_update;
  }

  pub fn end_episode(&mut self) {
    let device = self.state_weights_base.device();

    self.prediction_matrix_base =
      self.prediction_matrix_base.clone() + self.prediction_matrix_overlay.clone();
    self.prediction_matrix_base = self.prediction_matrix_base.clone().clamp(
      self.prediction_max_negative_weight,
      self.prediction_max_positive_weight,
    );
    self.prediction_matrix_overlay = Tensor::zeros([128, 64], &device);

    self.update_predictive_weight_change_magnitude();
    let state_utilization = self.calculate_state_utilization();
    let predictive_utilization = self.calculate_predictive_utilization();

    consolidate_and_randomize_weights(
      &mut self.state_weights_base,
      &mut self.state_weights_overlay,
      &state_utilization,
      [32, 64],
      &device,
    );

    consolidate_and_randomize_weights(
      &mut self.predictive_weights_base,
      &mut self.predictive_weights_overlay,
      &predictive_utilization,
      [76, 128],
      &device,
    );

    self.action_weights_base =
      self.action_weights_base.clone() + self.action_weights_overlay.clone();
    self.action_weights_overlay = Tensor::zeros([64, 12], &device);

    self.firing_history.clear();
    self.current_timestep = 0;

    self.reward_magnitude = 0.0;
    self.punishment_magnitude = 0.0;
  }

  fn calculate_state_utilization(&self) -> Tensor<B, 1> {
    let pred_matrix = self.prediction_matrix_base.clone() + self.prediction_matrix_overlay.clone();

    let predictive_weights =
      self.predictive_weights_base.clone() + self.predictive_weights_overlay.clone();

    let state_to_pred = predictive_weights.slice([0..64, 0..128]);

    let pred_matrix_positive = pred_matrix.clone().greater_elem(0.0).float() * pred_matrix;

    let pred_transposed = pred_matrix_positive.transpose(); // [64, 128]

    (pred_transposed * state_to_pred.abs()).sum_dim(1).squeeze()
  }
  fn calculate_predictive_utilization(&self) -> Tensor<B, 1> {
    let pred_matrix = self.prediction_matrix_base.clone() + self.prediction_matrix_overlay.clone();

    let pred_matrix_positive = pred_matrix.clone().greater_elem(0.0).float() * pred_matrix;

    pred_matrix_positive.sum_dim(1).squeeze()
  }
  fn update_predictive_weight_change_magnitude(&mut self) {
    let overlay = self.predictive_weights_overlay.clone();

    // sum absolute values across inputs  for each predictive neuron
    self.predictive_weight_change_magnitude = overlay.abs().sum_dim(0).squeeze();
  }
}

fn consolidate_and_randomize_weights<B: Backend>(
  base: &mut Tensor<B, 2>,
  overlay: &mut Tensor<B, 2>,
  utilization: &Tensor<B, 1>,
  shape: [usize; 2],
  device: &B::Device,
) {
  *base = base.clone() + overlay.clone();

  let base_magnitude = base.clone().abs();
  let util_2d = utilization.clone().unsqueeze_dim(0).repeat(&[shape[0], 1]);
  let overlay_magnitude = base_magnitude / (util_2d + 1.0);
  let random_vals = Tensor::random(
    shape,
    burn::tensor::Distribution::Uniform(-1.0, 1.0),
    device,
  );

  *overlay = random_vals * overlay_magnitude;
}
