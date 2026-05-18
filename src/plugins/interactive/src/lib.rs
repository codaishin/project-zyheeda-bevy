mod assets;
mod components;
mod system_params;

use crate::{assets::door_meta::DoorMeta, system_params::interactive::InteractiveMut};
use bevy::prelude::*;
use common::{
	states::game_state::LoadingEssentialAssets,
	traits::{
		handles_custom_assets::HandlesCustomFolderAssets,
		handles_interactive::HandlesInteractive,
		thread_safe::ThreadSafe,
	},
};
use std::marker::PhantomData;

pub struct InteractivePlugin<TDependencies>(PhantomData<TDependencies>);

impl<TLoading> InteractivePlugin<TLoading>
where
	TLoading: ThreadSafe + HandlesCustomFolderAssets,
{
	pub fn from_plugin(_: &TLoading) -> Self {
		Self(PhantomData)
	}
}

impl<TLoading> Plugin for InteractivePlugin<TLoading>
where
	TLoading: ThreadSafe + HandlesCustomFolderAssets,
{
	fn build(&self, app: &mut App) {
		TLoading::register_custom_folder_assets::<DoorMeta, DoorMeta, LoadingEssentialAssets>(app);

		app.init_asset::<DoorMeta>();
	}
}

impl<TDependencies> HandlesInteractive for InteractivePlugin<TDependencies> {
	type TInteractiveMut = InteractiveMut<'static, 'static>;
}
