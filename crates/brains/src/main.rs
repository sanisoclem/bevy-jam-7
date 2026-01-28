#![recursion_limit = "256"]

use burn::backend::Wgpu;
use burn::{
  nn::{
    Dropout, DropoutConfig, Linear, LinearConfig, Relu,
    conv::{Conv2d, Conv2dConfig},
    pool::{AdaptiveAvgPool2d, AdaptiveAvgPool2dConfig},
  },
  prelude::*,
};

fn main() {
  // let device = Default::default();
  // // Creation of two tensors, the first with explicit values and the second one with ones, with the same shape as the first
  // let tensor_1 = Tensor::<Backend, 2>::from_data([[2., 3.], [4., 5.]], &device);
  // let tensor_2 = Tensor::ones_like(&tensor_1);
  //
  // // Print the element-wise addition (done with the WGPU backend) of the two tensors.
  // println!("{}", tensor_1 + tensor_2);
  //

  let device = Default::default();
  let model = ModelConfig::new(10, 512).init::<Wgpu<f32, i32>>(&device);

  println!("{model}");
}

#[derive(Module, Debug)]
pub struct Model<B: Backend> {
  conv1: Conv2d<B>,
  conv2: Conv2d<B>,
  pool: AdaptiveAvgPool2d,
  dropout: Dropout,
  linear1: Linear<B>,
  linear2: Linear<B>,
  activation: Relu,
}

#[derive(Config, Debug)]
pub struct ModelConfig {
  num_classes: usize,
  hidden_size: usize,
  #[config(default = "0.5")]
  dropout: f64,
}

impl ModelConfig {
  /// Returns the initialized model.
  pub fn init<B: Backend>(&self, device: &B::Device) -> Model<B> {
    Model {
      conv1: Conv2dConfig::new([1, 8], [3, 3]).init(device),
      conv2: Conv2dConfig::new([8, 16], [3, 3]).init(device),
      pool: AdaptiveAvgPool2dConfig::new([8, 8]).init(),
      activation: Relu::new(),
      linear1: LinearConfig::new(16 * 8 * 8, self.hidden_size).init(device),
      linear2: LinearConfig::new(self.hidden_size, self.num_classes).init(device),
      dropout: DropoutConfig::new(self.dropout).init(),
    }
  }
}
