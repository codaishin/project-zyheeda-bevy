pub mod resources;
pub mod systems;
pub mod traits;

pub(crate) mod asset_loader;
pub(crate) mod folder_asset_loader;

use bevy::prelude::*;
use common::{
	states::{game_state::GameState, load_state::LoadState},
	traits::{init_resource::InitResource, remove_resource::RemoveResource},
};
use resources::track::Track;
use traits::progress::{AssetsProgress, DependenciesProgress};

pub struct LoadingPlugin;

impl Plugin for LoadingPlugin {
	fn build(&self, app: &mut App) {
		let load_assets = GameState::Loading(LoadState::Assets);
		let load_deps = GameState::Loading(LoadState::Dependencies);

		app.add_systems(OnEnter(load_assets), Track::<AssetsProgress>::init)
			.add_systems(OnExit(load_assets), Track::<AssetsProgress>::remove)
			.add_systems(OnEnter(load_deps), Track::<DependenciesProgress>::init)
			.add_systems(OnExit(load_deps), Track::<DependenciesProgress>::remove);
	}
}
