use bevy::prelude::*;

pub(crate) trait GetMountPoint<T> {
	type TError;

	fn get_mount_point(&mut self, root: Entity, key: T) -> Result<Entity, Self::TError>;
}
