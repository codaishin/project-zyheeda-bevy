use crate::states::asset_load_phase::AssetLoadPhase;
use bevy::prelude::*;
use common::prelude::*;

/// When present, the load group is considered to be loaded. This allows
/// after-load-system to run.
///
/// Used instead of a [`State`] because of the 1-2 frames delay when setting new states
/// from within a system.
#[derive(Resource)]
pub(crate) struct GroupLoaded<TLoadGroup>(pub(crate) TLoadGroup);

impl<TLoadGroup> GroupLoaded<TLoadGroup>
where
	TLoadGroup: LoadGroup<AssetLoadPhase>,
{
	pub(crate) fn insert(load_group: TLoadGroup) -> impl Fn(ZyheedaCommands) {
		move |mut commands: ZyheedaCommands| commands.insert_resource(Self(load_group))
	}

	pub(crate) fn remove(mut commands: ZyheedaCommands) {
		commands.remove_resource::<Self>();
	}

	pub(crate) fn exists_as_resource(&self) -> fn(Option<Res<Self>>) -> bool {
		|res| res.is_some()
	}
}
