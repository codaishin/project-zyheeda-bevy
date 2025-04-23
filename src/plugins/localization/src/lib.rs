pub mod resources;

mod assets;
mod systems;
mod traits;

use assets::ftl::{Ftl, loader::FtlLoader};
use bevy::prelude::*;
use common::{
	systems::log::log_many,
	traits::{
		handles_load_tracking::HandlesLoadTracking,
		load_asset::Path,
		thread_safe::ThreadSafe,
	},
};
use resources::ftl_server::FtlServer;
use std::marker::PhantomData;
use systems::{
	init_ftl_server::InitFtlServer,
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
		app.init_asset::<Ftl>()
			.register_asset_loader(FtlLoader)
			.add_systems(Startup, FtlServer::init_with(langid!("en-US")))
			.add_systems(
				Update,
				(
					FtlServer::load_requested_assets(Path::from("locale")),
					FtlServer::update_ftl_bundle.pipe(log_many),
				),
			);
	}
}
