use bevy::prelude::*;
use std::marker::PhantomData;
pub use utils::assets::AssetBarrier;

use bevy::asset::AssetServer;

pub trait IAssetBundle: Send + Sync + 'static {
  fn load_all(asset_server: &AssetServer) -> (AssetBarrier, Self);
}

pub struct SysAssetPlugin<T: IAssetBundle>(PhantomData<T>);

impl<T> Plugin for SysAssetPlugin<T>
where
  T: IAssetBundle,
{
  fn build(&self, app: &mut App) {
    app
      .init_resource::<AssetBundle<T>>()
      .add_systems(
        Update,
        process_assets::<T>.run_if(should_process_assets::<T>),
      )
      .add_observer(on_load_assets::<T>);
  }
}
impl<T: IAssetBundle> Default for SysAssetPlugin<T> {
  fn default() -> Self {
    Self(Default::default())
  }
}

#[derive(Resource, Default)]
pub enum AssetBundle<T> {
  #[default]
  NotLoaded,
  Loading(T, AssetBarrier),
  Loaded(T),
}

#[derive(Event, Debug)]
pub struct LoadAssets<T>(PhantomData<T>);

impl<T: IAssetBundle> Default for LoadAssets<T> {
  fn default() -> Self {
    Self(Default::default())
  }
}
#[derive(Event, Debug)]
pub struct AssetLoaded<T>(PhantomData<T>);

impl<T> Default for AssetLoaded<T> {
  fn default() -> Self {
    Self(Default::default())
  }
}

pub fn on_load_assets<T>(
  _evt: On<LoadAssets<T>>,
  asset_server: Res<AssetServer>,
  mut res: ResMut<AssetBundle<T>>,
) where
  T: IAssetBundle,
{
  if !matches!(*res, AssetBundle::<T>::NotLoaded) {
    return;
  }

  let (barrier, bundle) = T::load_all(&asset_server);
  *res = AssetBundle::Loading(bundle, barrier);
}

fn should_process_assets<T: IAssetBundle>(bundle: Res<AssetBundle<T>>) -> bool {
  if let AssetBundle::Loading(_b, barrier) = bundle.as_ref()
    && barrier.is_ready()
  {
    true
  } else {
    false
  }
}

fn process_assets<T: IAssetBundle>(mut bundle: ResMut<AssetBundle<T>>, mut cmd: Commands) {
  // should work, but a bit confusing to read
  let mut placeholder = AssetBundle::NotLoaded;
  std::mem::swap(&mut *bundle, &mut placeholder);
  let AssetBundle::Loading(inner, _) = placeholder else {
    return;
  };
  *bundle = AssetBundle::Loaded(inner);

  cmd.trigger(AssetLoaded::<T>::default());
}
