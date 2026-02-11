mod get_mount_point;

use crate::components::{fix_points::MountPointsDefinition, mount_points::MountPoints};
use bevy::{ecs::system::SystemParam, prelude::*};
use common::traits::thread_safe::ThreadSafe;
use std::hash::Hash;

#[derive(SystemParam, Debug)]
pub(crate) struct MountPointsLookup<'w, 's, T>
where
	T: ThreadSafe + Eq + Hash,
{
	mount_points: Query<
		'w,
		's,
		(
			&'static MountPointsDefinition<T>,
			&'static mut MountPoints<T>,
		),
	>,
	children: Query<'w, 's, &'static Children>,
	names: Query<'w, 's, &'static Name>,
}
