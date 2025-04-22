pub mod resources;

mod assets;
mod systems;

use assets::ftl::{Ftl, loader::FtlLoader};
use bevy::prelude::*;
use common::traits::{
	handles_load_tracking::HandlesLoadTracking,
	load_asset::Path,
	thread_safe::ThreadSafe,
};
use resources::ftl_server::FtlServer;
use std::marker::PhantomData;
use systems::init_ftl_server::InitFtlServer;
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
			.add_systems(
				Startup,
				FtlServer::init_with(langid!("en-US"), Path::from("locale")),
			);
	}
}
