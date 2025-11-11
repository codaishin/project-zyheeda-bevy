pub mod resources;

mod assets;
mod systems;
mod tools;
mod traits;

use crate::systems::remove_empty_folder_handle::RemoveEmptyFolderHandle;
use assets::ftl::{Ftl, loader::FtlLoader};
use bevy::prelude::*;
use common::{
	states::game_state::LoadingEssentialAssets,
	systems::log::OnError,
	tools::path::Path,
	traits::{
		handles_load_tracking::{AssetsProgress, HandlesLoadTracking, LoadTrackingInApp},
		handles_localization::HandlesLocalization,
		thread_safe::ThreadSafe,
	},
};
use resources::ftl_server::FtlServer;
use std::marker::PhantomData;
use systems::{
	init_ftl_server::InitFtlServer,
	load_requested_asset_file::LoadRequestedAssetFile,
	load_requested_asset_folder::LoadRequestedAssetFolder,
	remove_failed_asset_handles::RemoveFailedAssetHandles,
	update_ftl_bundle::UpdateFtlBundle,
};
use unic_langid::langid;

pub struct LocalizationPlugin<TLoading>(PhantomData<TLoading>);

impl<TLoading> LocalizationPlugin<TLoading>
where
	TLoading: HandlesLoadTracking + ThreadSafe,
{
	pub fn from_plugin(_: &TLoading) -> Self {
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
					FtlServer::load_requested_asset_file(Path::from("locale")),
					FtlServer::load_requested_asset_folder(Path::from("locale")),
					FtlServer::remove_failed_asset_handles,
					FtlServer::remove_empty_folder_handle,
					FtlServer::update_ftl_bundle.pipe(OnError::log),
				),
			);
	}
}

impl<TDependencies> HandlesLocalization for LocalizationPlugin<TDependencies> {
	type TLocalizationServer = FtlServer;
}
