pub mod resources;
pub mod systems;
pub mod traits;

pub(crate) mod folder_asset_loader;

use bevy::{prelude::*, state::state::FreelyMutableState};
use common::traits::{init_resource::InitResource, remove_resource::RemoveResource};
use resources::track::Track;
use traits::progress::{AssetLoadProgress, DependencyResolveProgress};

pub struct LoadingPlugin<TState> {
	pub load_assets: TState,
	pub resolve_dependencies: TState,
}

impl<TState> Plugin for LoadingPlugin<TState>
where
	TState: FreelyMutableState + Copy,
{
	fn build(&self, app: &mut App) {
		let load = self.load_assets;
		let resolve = self.resolve_dependencies;

		app.add_systems(OnEnter(load), Track::<AssetLoadProgress>::init)
			.add_systems(OnExit(load), Track::<AssetLoadProgress>::remove)
			.add_systems(OnEnter(resolve), Track::<DependencyResolveProgress>::init)
			.add_systems(OnExit(resolve), Track::<DependencyResolveProgress>::remove);
	}
}
