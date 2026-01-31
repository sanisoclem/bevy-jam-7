use burn::{prelude::*, tensor::backend::Backend};
use core::f32;
use ring::*;
use std::ops::Add;

mod ring;

const NUM_INPUTS: usize = 32;
const NUM_STATE_NEURONS: usize = 64;
const NUM_ACTION_NEURONS: usize = 12;
const NUM_PREDICTIVE_NEURONS: usize = 128;
const NUM_PREDICTIVE_INPUTS: usize = NUM_STATE_NEURONS + NUM_ACTION_NEURONS;

#[derive(Clone, Debug)]
struct TimestepRecord<B: Backend> {
  state_activations: Tensor<B, 1>,
  focused_activations: Tensor<B, 1>,
  action_activations: Tensor<B, 1>,
  stack_count: usize,
}
impl<B: Backend> SampleData for TimestepRecord<B> {}
impl<B: Backend> Add for TimestepRecord<B> {
  type Output = Self;

  fn add(self, rhs: Self) -> Self::Output {
    TimestepRecord {
      state_activations: self.state_activations + rhs.state_activations,
      focused_activations: self.focused_activations + rhs.focused_activations,
      action_activations: self.action_activations + rhs.action_activations,
      stack_count: self.stack_count + rhs.stack_count,
    }
  }
}

#[derive(Debug)]
struct RewardLayer {
  label: String,
  // reservoir of rewards, if this runs out, no rewards can be given
  reservoir: f32,
  // how fast reservoir repleanishes
  recovery_rate: f32,
  // accumulates when current dominant reward layer
  // negatively affects effective score, if fatigue accumulates and
  // layer loses dominance
  fatigue: f32,
  fatigue_rate: f32,
  fatigue_punishment_rate: f32,
}

// TODO:
//  - need to add fatigue to neuron activation, prolonged activation should cause dulled response.
//  Also prolonged action exertion should also trigger a punishment response. These will prevent
//  getting stuck in paths and sound biologically plausible because neuron activations come at a
//  cost that is replenished at a finite pace.
//  - do we need to diminish weights for state neurons that didn't fire when rewarding action
//  neurons?
//  - need to penalize actions that are so ingrained do not create novelty (very high base weights)
//  and no novelty rewards generated in the current episode.
//  - do we need more state overlays aside from reward/punishment. I'm thinking things like fatigue
//  could have different effects on neurons too. Emotions might be good too: fight or flight, fear,
//  happyness etc. these states are feedback from the organism body or environment and should have
//  a cooldown, decay, max values etc.

#[derive(Debug)]
pub struct PredictiveOrganism<B: Backend> {
  // state neurons
  state_weights_base: Tensor<B, 2>,
  state_weights_overlay: Tensor<B, 2>,
  state_weights_effective: Tensor<B, 2>,
  state_bias: Tensor<B, 1>,
  state_threshold: f32,

  // action neurons
  action_weights_base: Tensor<B, 2>,
  action_weights_overlay: Tensor<B, 2>,
  action_weights_effective: Tensor<B, 2>,
  action_bias: Tensor<B, 1>,
  action_threshold: f32,
  action_inhibition_weights: Tensor<B, 2>,

  // prediction neurons
  predictive_weights_base: Tensor<B, 2>,
  predictive_weights_overlay: Tensor<B, 2>,
  predictive_bias: Tensor<B, 1>,
  predictive_threshold: f32,

  // how quickly prediction weight rewards decay over time
  prediction_decay_rate: f32,

  // prediction performance matrix
  predictions: Tensor<B, 2>,
  prediction_matrix_base: Tensor<B, 2>,
  prediction_matrix_overlay: Tensor<B, 2>,

  // track magnitude of weight changes for scaling learning rate
  // useful neurons change less, useless neurons change waaay more
  // and the more a neurons weight change, the faster it's previously
  // earned prediction performance decays
  predictive_weight_change_magnitude: Tensor<B, 1>,
  // how strongly influenced an overlay is
  // typically dependent how much the underlying neuron has changed
  // for Predictive Neurons, this ramps up the prediction performance score
  // if the neuron has changed a lot in the episode, otherwise, attenuates
  // any change
  volatility_amplification_rate: f32,
  // max consolidated base predictive weights, we have to cap
  // these values to ensure we can unlearn old behaviors and learn new ones
  prediction_max_negative_weight: f32,
  prediction_max_positive_weight: f32,
  // track reward score for each state neuron
  reward_score_base: Tensor<B, 2>,
  reward_score_overlay: Tensor<B, 2>,
  reward_layers: Vec<RewardLayer>,

  // circular buffer for temporal credit assignment
  // TODO: this buffer should have higher resolution for recent events
  // and lower resolution for older ones, so we can look back lot and only attribute
  // learning for the most impactful neurons the farther we look back
  activation_history: HierarchicalBuffer<TimestepRecord<B>>,

  // are overflows significant?
  current_timestep: usize,

  // determines if a newly formed predictive link (base predictive weight <= 0)
  // is novel must be high enough but not too high that we have seen it too many times
  stable_link_threshold_min: f32,
  stable_link_threshold_max: f32,
  // the % of prediction neurons that are expected to be in flux
  // values exceeding this will have higher reward magnitudes and will
  // be similar to a traumatic event
  novelty_rate: f32,
}

#[derive(Debug)]
pub struct ModelMetrics {
  pub total_reward_amount: f32,
  pub total_predictive_power: f32,
  pub network_utilization: f32,
  pub predictive_utilization: f32,
  pub state_utilization: f32,
}

impl<B: Backend> PredictiveOrganism<B>
where
  B: Backend<FloatElem = f32>,
{
  pub fn new(device: &B::Device) -> Self {
    let state_weights_base = Tensor::random(
      [NUM_INPUTS, NUM_STATE_NEURONS],
      burn::tensor::Distribution::Uniform(-1.0, 1.0),
      device,
    );
    let state_weights_overlay = Tensor::zeros([NUM_INPUTS, NUM_STATE_NEURONS], device);
    let state_bias = Tensor::zeros([NUM_STATE_NEURONS], device);
    let state_weights_effective = state_weights_overlay.clone() + state_weights_base.clone();
    let action_weights_base = Tensor::random(
      [NUM_STATE_NEURONS, NUM_ACTION_NEURONS],
      burn::tensor::Distribution::Uniform(-1.0, 1.0),
      device,
    );
    let action_weights_overlay = Tensor::zeros([NUM_STATE_NEURONS, NUM_ACTION_NEURONS], device);
    let action_bias = Tensor::zeros([NUM_ACTION_NEURONS], device);
    let action_inhibition_weights = Self::create_action_inhibition_matrix(device);
    let action_weights_effective = action_weights_overlay.clone() + action_weights_base.clone();
    let predictive_weights_base = Tensor::random(
      [NUM_PREDICTIVE_INPUTS, NUM_PREDICTIVE_NEURONS],
      burn::tensor::Distribution::Uniform(-1.0, 1.0),
      device,
    );
    let predictive_weights_overlay =
      Tensor::zeros([NUM_PREDICTIVE_INPUTS, NUM_PREDICTIVE_NEURONS], device);
    let predictive_bias = Tensor::zeros([NUM_PREDICTIVE_NEURONS], device);
    let predictions = Tensor::zeros([NUM_PREDICTIVE_NEURONS, NUM_STATE_NEURONS], device);
    let prediction_matrix_base = Tensor::zeros([NUM_PREDICTIVE_NEURONS, NUM_STATE_NEURONS], device);
    let prediction_matrix_overlay =
      Tensor::zeros([NUM_PREDICTIVE_NEURONS, NUM_STATE_NEURONS], device);

    let reward_layers = vec![
      RewardLayer {
        label: "Novelty".to_string(),
        recovery_rate: 1.0,
        reservoir: 100.0,
        fatigue: 0.,
        fatigue_rate: 0.01,
        fatigue_punishment_rate: -1.0,
      },
      RewardLayer {
        label: "Food".to_string(),
        recovery_rate: 1.0,
        reservoir: 100.0,
        fatigue: 0.0,
        fatigue_rate: 0.01,
        fatigue_punishment_rate: -1.0,
      },
      RewardLayer {
        label: "Pain".to_string(),
        recovery_rate: 1.0,
        reservoir: 100.0,
        fatigue: 0.0,
        fatigue_rate: 0.01,
        fatigue_punishment_rate: 1.0,
      },
    ];

    let num_layers = reward_layers.len();
    let reward_score_base = Tensor::zeros([NUM_STATE_NEURONS, num_layers], device);
    let reward_score_overlay = Tensor::zeros([NUM_STATE_NEURONS, num_layers], device);

    Self {
      state_weights_base,
      state_weights_overlay,
      state_weights_effective,
      state_bias,
      state_threshold: 0.5,
      action_weights_base,
      action_weights_overlay,
      action_weights_effective,
      action_bias,
      action_threshold: 0.5,
      action_inhibition_weights,
      predictive_weights_base,
      predictive_weights_overlay,
      predictive_bias,
      predictive_threshold: 0.5,
      prediction_matrix_base,
      predictions,
      prediction_matrix_overlay,
      predictive_weight_change_magnitude: Tensor::zeros([NUM_PREDICTIVE_NEURONS], device),
      reward_score_base,
      reward_score_overlay,
      reward_layers,
      activation_history: HierarchicalBuffer::new(vec![30, 60, 60, 24], 2.),
      current_timestep: 0,
      stable_link_threshold_min: 5.0,
      stable_link_threshold_max: 20.0,
      prediction_max_negative_weight: -100.,
      prediction_max_positive_weight: 1000.,
      volatility_amplification_rate: 10.,
      prediction_decay_rate: 1.5,
      novelty_rate: 0.1,
    }
  }

  pub fn metrics(&self) -> ModelMetrics {
    let state_utilization = self
      .calculate_state_utilization()
      .greater_equal_elem(1.0)
      .float()
      .sum()
      .into_scalar()
      / NUM_STATE_NEURONS as f32;
    let predictive_utilization = self
      .calculate_predictive_utilization()
      .greater_equal_elem(1.0)
      .float()
      .sum()
      .into_scalar()
      / NUM_PREDICTIVE_NEURONS as f32;
    ModelMetrics {
      network_utilization: predictive_utilization * state_utilization,
      predictive_utilization,
      state_utilization,
      total_predictive_power: (self.prediction_matrix_overlay.clone()
        + self.prediction_matrix_base.clone())
      .sum()
      .into_scalar(),
      total_reward_amount: (self.reward_score_overlay.clone() + self.reward_score_base.clone())
        .sum()
        .into_scalar(),
    }
  }

  fn create_action_inhibition_matrix(device: &B::Device) -> Tensor<B, 2> {
    let mut inhibition = vec![0.0; NUM_ACTION_NEURONS * NUM_ACTION_NEURONS];

    let opposing_pairs = vec![
      (0, 1), // left right
      (2, 3), // up down
    ];

    for (a, b) in opposing_pairs {
      inhibition[a * NUM_ACTION_NEURONS + b] = -5.0;
      inhibition[b * NUM_ACTION_NEURONS + a] = -5.0;
    }

    Tensor::<B, 1>::from_floats(inhibition.as_slice(), device)
      .reshape([NUM_ACTION_NEURONS, NUM_ACTION_NEURONS])
  }

  pub fn forward(&mut self, input: Tensor<B, 1>) {
    // compute state neurons
    self.state_weights_effective =
      self.state_weights_base.clone() + self.state_weights_overlay.clone();
    let state_logits = input
      .unsqueeze_dim(0)
      .matmul(self.state_weights_effective.clone())
      .squeeze()
      + self.state_bias.clone();
    let state_activations = self.apply_threshold(state_logits, self.state_threshold);

    // compute focus based on reward scores and layer dominance
    let focused_activations = self.compute_focus(&state_activations);

    // compute actions from focused neurons
    self.action_weights_effective =
      self.action_weights_base.clone() + self.action_weights_overlay.clone();
    let action_logits = focused_activations
      .clone()
      .unsqueeze_dim(0)
      .matmul(self.action_weights_effective.clone())
      .squeeze()
      + self.action_bias.clone();

    // apply inhibition between opposing actions
    let action_inhibition = action_logits
      .clone()
      .unsqueeze_dim(0)
      .matmul(self.action_inhibition_weights.clone())
      .squeeze();
    let action_logits_final = action_logits + action_inhibition;

    let action_activations = self.apply_threshold(action_logits_final, self.action_threshold);

    // store in circular buffer for temporal credit assignment
    self.activation_history.push(TimestepRecord {
      state_activations: state_activations.clone(),
      focused_activations: focused_activations.clone(),
      action_activations: action_activations.clone(),
      stack_count: 1,
    });

    // calculate predictions for next timestep
    let correct_predictions =
      self.update_predictions(state_activations.clone(), action_activations.clone());

    // TODO: fire rewards
    self.check_novelty(correct_predictions.clone());

    // replenish rewards reservoir
    for layer in self.reward_layers.iter_mut() {
      layer.reservoir += layer.recovery_rate;
    }

    self.current_timestep += 1;
  }

  fn compute_focus(&mut self, state_activations: &Tensor<B, 1>) -> Tensor<B, 1> {
    let device = state_activations.device();
    let reward_scores = self.reward_score_base.clone() + self.reward_score_overlay.clone();

    let mut sum_per_layer: Vec<(usize, f32)> = reward_scores
      .clone()
      .sum_dim(0)
      .to_data()
      .into_vec()
      .expect("Can convert tensor to f32 vec")
      .into_iter()
      .enumerate()
      .map(|(idx, sum): (usize, f32)| (idx, sum * (1.0 - self.reward_layers[idx].fatigue)))
      .collect();

    sum_per_layer.sort_by(|(_idx, sum1), (_idx2, sum2)| {
      sum2
        .partial_cmp(sum1)
        .expect("sum comparison should succeed")
    });

    let mut idx_mult: Vec<(usize, f32)> = sum_per_layer
      .iter()
      .copied()
      .enumerate() // include ranks
      .map(|(rank, (idx, _sum))| (idx, 0.5f32.powi(rank as i32 + 1)))
      .collect();

    idx_mult.sort_by(|(idx1, _m1), (idx2, _m2)| {
      idx1
        .partial_cmp(idx2)
        .expect("idx comparison should succeed")
    });

    let multipliers: Vec<_> = idx_mult.into_iter().map(|(_idx, m)| m).collect();

    let multipliers_tensor =
      Tensor::<B, 1>::from_floats(multipliers.as_slice(), &device).unsqueeze_dim(1);

    // NOTE: currently the focus is determined only by:
    //  - state - only state neurons that fire can be focused
    //  - experience - neurons that have been associated to rewards
    //
    //  I think it might be also useful to some sort of momentum
    //  where it is harder to change the dominant reward layer.
    //
    //  Another idea is to drive this with another neural network for more
    //  flexible focusing rules, but there are a few questions:
    //   - what inputs into those neurons
    //   - how do those neurons learn - reward experience?

    // amplify rewards from layers that have more active rewarding neurons
    let scaled_reward_scores = reward_scores.clone() * multipliers_tensor.clone().transpose();

    // calculate total rewards across all layers per neuron
    let total_scaled_rewards_per_neuron: Tensor<B, 1> =
      scaled_reward_scores.sum_dim(1).flatten(0, 1);

    // normalize so the highest value is 1.0
    // depending on the score distribution, this might only focus on few neurons
    let min_val = total_scaled_rewards_per_neuron.clone().min().into_scalar();
    let max_val = total_scaled_rewards_per_neuron.clone().max().into_scalar();
    let range = max_val - min_val;
    let normalized = if range == 0.0 {
      Tensor::ones_like(&total_scaled_rewards_per_neuron)
    } else {
      (total_scaled_rewards_per_neuron - min_val) / range
    };

    // accumulate/consume fatigue
    for (rank, (layer_idx, _layer_sum)) in sum_per_layer.into_iter().enumerate() {
      if rank == 0 {
        // dominant layers get fatigue over time
        self.reward_layers[layer_idx].fatigue += self.reward_layers[layer_idx].fatigue_rate;
      } else if self.reward_layers[layer_idx].fatigue > 0. {
        // not dominant layers that have fatigue consolidate it

        self.apply_reward(
          layer_idx,
          self.reward_layers[layer_idx].fatigue
            * self.reward_layers[layer_idx].fatigue_punishment_rate,
        );
        self.reward_layers[layer_idx].fatigue = 0.;
      }
    }

    normalized * state_activations.clone()
  }

  fn update_predictions(
    &mut self,
    state_activations: Tensor<B, 1>,
    action_activations: Tensor<B, 1>,
  ) -> Tensor<B, 1> {
    let correct_predictions = self.predictions.clone() * state_activations.clone().unsqueeze_dim(0);

    // remove "claimed" predictions
    self.predictions = self.predictions.clone() - correct_predictions.clone();

    // make new predictions
    let predictive_input = Tensor::cat(
      vec![state_activations.clone(), action_activations.clone()],
      0,
    );
    let predictive_weights =
      self.predictive_weights_base.clone() + self.predictive_weights_overlay.clone();
    let predictive_logits = predictive_input
      .unsqueeze_dim(0)
      .matmul(predictive_weights)
      .squeeze()
      + self.predictive_bias.clone();
    let activations = self.apply_threshold(predictive_logits, self.predictive_threshold);

    // get nodes that have no predictions
    let no_predictions = self.predictions.clone().equal_elem(0.0).float();
    // get new prediction candidates
    let has_predictions = activations.clone().equal_elem(1.0).float().unsqueeze_dim(1);
    // get new predictions that have no existing predictions
    let new_predictions = no_predictions.clone() * has_predictions;
    // get existing (not new) predictions andcalc decayed value
    let should_decay = Tensor::ones_like(&self.predictions) - no_predictions;
    // need to mult by 0.95 to allow 1.0 values to decay
    let decayed = (self
      .predictions
      .clone()
      .powf_scalar(self.prediction_decay_rate)
      * 0.95)
      .clamp_min(0.0);
    let decayed_result = should_decay * decayed.clone();

    // predictions for next timestep
    self.predictions = new_predictions + decayed_result;

    // scale learning rate by weight change magnitude (faster learning for neurons with changing
    // weights)
    let scaling_tensor = ((self.predictive_weight_change_magnitude.clone()
      * self.volatility_amplification_rate.ln())
    .exp()
      * self.prediction_decay_rate)
      .unsqueeze_dim(1)
      .repeat(&[1, NUM_STATE_NEURONS]);

    // update prediction performance matrix
    self.prediction_matrix_overlay = self.prediction_matrix_overlay.clone()
      + (correct_predictions.clone() + decayed.equal_elem(0.0).float() * -2.0) * scaling_tensor;

    // return all P neurons that have at least one correct prediction
    correct_predictions
      .clone()
      .sum_dim(1)
      .squeeze_dim(1)
      .greater_equal_elem(1.0)
      .float()
  }

  fn apply_threshold(&self, tensor: Tensor<B, 1>, threshold: f32) -> Tensor<B, 1> {
    let mask = tensor.greater_elem(threshold);
    mask.float()
  }

  fn check_novelty(&mut self, correct_predictions: Tensor<B, 1>) {
    // novelty = new stable predictive links discovered
    let pred_fired_mask = correct_predictions
      .clone()
      .unsqueeze_dim(1)
      .repeat(&[1, NUM_STATE_NEURONS]);

    let overlay_low = self
      .prediction_matrix_overlay
      .clone()
      .greater_elem(self.stable_link_threshold_min)
      .float();
    let overlay_high = self
      .prediction_matrix_overlay
      .clone()
      .lower_elem(self.stable_link_threshold_max)
      .float();

    let base_low = self
      .prediction_matrix_base
      .clone()
      .lower_equal_elem(0.0)
      .float();

    let new_links_mask = pred_fired_mask * overlay_high * overlay_low * base_low;

    let reward =
      new_links_mask.sum().into_scalar() / (NUM_PREDICTIVE_INPUTS as f32 * self.novelty_rate);

    self.apply_reward(0, reward);
  }

  pub fn apply_reward(&mut self, layer_idx: usize, amount: f32) {
    let layer = self
      .reward_layers
      .get_mut(layer_idx)
      .expect("reward layer should exit");

    if amount > layer.reservoir {
      // unable to apply reward, no more in reservoir
      return;
    }

    layer.reservoir -= amount;

    // credit focused state neurons
    for idx in 0..self.activation_history.len() {
      // TODO: implement iterator
      let (mult, data) = self
        .activation_history
        .get_tier(idx)
        .expect("tier should exist");
      let Some(summed) = data.sum_all() else {
        // not enough data in timestep
        continue;
      };
      let reward = amount * mult;
      let state_rewards =
        summed.focused_activations.clone() * reward / summed.focused_activations.clone().sum();
      let layer_slice = self
        .reward_score_overlay
        .clone()
        .slice([0..NUM_STATE_NEURONS, layer_idx..layer_idx + 1]);
      let updated_rewards = layer_slice + state_rewards.unsqueeze_dim(1);

      self.reward_score_overlay = self.reward_score_overlay.clone().slice_assign(
        [0..NUM_STATE_NEURONS, layer_idx..layer_idx + 1],
        updated_rewards,
      );
    }
  }

  pub fn end_episode(&mut self) {
    let device = self.state_weights_base.device();

    // consolidate reward scores
    self.reward_score_base = self.reward_score_base.clone() + self.reward_score_overlay.clone();
    self.reward_score_overlay =
      Tensor::zeros([NUM_STATE_NEURONS, self.reward_layers.len()], &device);

    // consolidate prediction matrix
    self.prediction_matrix_base =
      self.prediction_matrix_base.clone() + self.prediction_matrix_overlay.clone();
    self.prediction_matrix_base = self.prediction_matrix_base.clone().clamp(
      self.prediction_max_negative_weight,
      self.prediction_max_positive_weight,
    );
    self.prediction_matrix_overlay =
      Tensor::zeros([NUM_PREDICTIVE_NEURONS, NUM_STATE_NEURONS], &device);

    // update weight change magnitude
    self.update_predictive_weight_change_magnitude();

    // calculate utilizations for consolidation
    let state_utilization = self.calculate_state_utilization();
    let predictive_utilization = self.calculate_predictive_utilization();

    // consolidate with utilization-based randomization
    consolidate_and_randomize_weights(
      &mut self.state_weights_base,
      &mut self.state_weights_overlay,
      &state_utilization,
      [NUM_INPUTS, NUM_STATE_NEURONS],
      &device,
    );

    consolidate_and_randomize_weights(
      &mut self.predictive_weights_base,
      &mut self.predictive_weights_overlay,
      &predictive_utilization,
      [NUM_PREDICTIVE_INPUTS, NUM_PREDICTIVE_NEURONS],
      &device,
    );

    // TODO: calculate how many predictions can be made per action
    // randomize weights???
    self.action_weights_base =
      self.action_weights_base.clone() + self.action_weights_overlay.clone();
    self.action_weights_overlay = Tensor::zeros([NUM_STATE_NEURONS, NUM_ACTION_NEURONS], &device);

    // reset for next episode
    self.current_timestep = 0;
    self.predictive_weight_change_magnitude = Tensor::zeros([NUM_PREDICTIVE_NEURONS], &device);

    // reset layer fatigue
    for layer in &mut self.reward_layers {
      layer.fatigue = 0.0;
    }
  }

  fn calculate_state_utilization(&self) -> Tensor<B, 1> {
    let pred_matrix = self.prediction_matrix_base.clone() + self.prediction_matrix_overlay.clone();

    let predictive_weights =
      self.predictive_weights_base.clone() + self.predictive_weights_overlay.clone();

    // extract state->predictive connections: first NUM_STATE_NEURONS rows
    let state_to_pred = predictive_weights.slice([0..NUM_STATE_NEURONS, 0..NUM_PREDICTIVE_NEURONS]);

    // only count predictions with positive performance
    let pred_matrix_positive = pred_matrix.clone().greater_elem(0.0).float() * pred_matrix;

    // transpose to [NUM_STATE_NEURONS, NUM_PREDICTIVE_NEURONS] for alignment
    let pred_transposed = pred_matrix_positive.transpose();

    // total prediction score * weights agg by state neuron
    (pred_transposed * state_to_pred.abs()).sum_dim(1).squeeze()
  }

  fn calculate_predictive_utilization(&self) -> Tensor<B, 1> {
    let pred_matrix = self.prediction_matrix_base.clone() + self.prediction_matrix_overlay.clone();

    // only count positive weights
    let pred_matrix_positive = pred_matrix.clone().greater_elem(0.0).float() * pred_matrix;

    // total prediction score agg by P neuron
    pred_matrix_positive.sum_dim(1).squeeze()
  }

  fn update_predictive_weight_change_magnitude(&mut self) {
    let overlay = self.predictive_weights_overlay.clone();

    // sum absolute values across input dimension for each predictive neuron
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
  // consolidate
  *base = base.clone() + overlay.clone();

  let base_magnitude = base.clone().abs();

  // broadcast utilization to 2D
  let util_2d = utilization.clone().unsqueeze_dim(0).repeat(&[shape[0], 1]);

  // overlay_magnitude = base_magnitude / (1 + util)
  let overlay_magnitude = base_magnitude / (util_2d + 1.0);

  // random values in [-1, 1]
  let random_vals = Tensor::random(
    shape,
    burn::tensor::Distribution::Uniform(-1.0, 1.0),
    device,
  );

  *overlay = random_vals * overlay_magnitude;
}
