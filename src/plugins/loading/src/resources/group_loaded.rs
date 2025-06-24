use bevy::prelude::*;
use common::traits::thread_safe::ThreadSafe;
use std::marker::PhantomData;

/// When present, the load group is considered to be loaded. This allows
/// after-load-system to run.
///
/// Used instead of a [`State`] because of the 1-2 frames delay when setting new states
/// from within a system. Especially relevant for resetting the [`LoadPlugin`](`crate::LoadingPlugin`)
/// for a specific load group. Such resets should not be delayed in order to prevent premature
/// running of after-load-system.
#[derive(Resource)]
pub(crate) struct GroupLoaded<TLoadGroup>(PhantomData<TLoadGroup>);

impl<TLoadGroup> Default for GroupLoaded<TLoadGroup> {
	fn default() -> Self {
		Self(PhantomData)
	}
}

impl<TLoadGroup> GroupLoaded<TLoadGroup>
where
	TLoadGroup: ThreadSafe,
{
	pub(crate) fn insert(mut commands: Commands) {
		commands.init_resource::<Self>();
	}

	pub(crate) fn remove(mut commands: Commands) {
		commands.remove_resource::<Self>();
	}
}
