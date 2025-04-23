pub mod resources;

mod assets;
mod systems;
mod traits;

use assets::ftl::{Ftl, loader::FtlLoader};
use bevy::prelude::*;
use common::{
	states::game_state::LoadingEssentialAssets,
	systems::log::log_many,
	traits::{
		handles_load_tracking::{AssetsProgress, HandlesLoadTracking, LoadTrackingInApp},
		handles_localization::HandlesLocalization,
		load_asset::Path,
		thread_safe::ThreadSafe,
	},
};
use resources::ftl_server::FtlServer;
use std::marker::PhantomData;
use systems::{
	drain_errors::DrainErrors,
	init_ftl_server::InitFtlServer,
	remove_failed_asset_handles::RemoveFailedAssetHandles,
	set_requested_language::LoadRequestedAssets,
	update_ftl_bundle::UpdateFtlBundle,
};
use unic_langid::langid;

pub struct LocalizationPlugin<TLoading>(PhantomData<TLoading>);

impl<TLoading> LocalizationPlugin<TLoading> {
	pub fn depends_on(_: &TLoading) -> Self {
		Self(PhantomData)
	}
}

impl<TLoading> Plugin for LocalizationPlugin<TLoading>
where
	TLoading: HandlesLoadTracking + ThreadSafe,
{
	fn build(&self, app: &mut App) {
		TLoading::register_load_tracking::<FtlServer, LoadingEssentialAssets, AssetsProgress>()
			.in_app(app, FtlServer::all_fallback_files_loaded);

		app.init_asset::<Ftl>()
			.register_asset_loader(FtlLoader)
			.add_systems(Startup, FtlServer::init_with(langid!("en-US")))
			.add_systems(
				Update,
				(
					FtlServer::load_requested_assets(Path::from("locale")),
					FtlServer::remove_failed_asset_handles,
					FtlServer::update_ftl_bundle.pipe(log_many),
					FtlServer::drain_errors.pipe(log_many),
				),
			);
	}
}

impl<TDependencies> HandlesLocalization for LocalizationPlugin<TDependencies> {
	type TLocalizationServer = FtlServer;
}
